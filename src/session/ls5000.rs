//! The LS-5000 ED behind a [`Session`](super::Session)
//!
//! The roll feeder senses where the frames are and reports a transport table, so
//! [`Placement::Sensed`] is the accurate way to place them and an even pitch is the fallback.

use super::{Driver, Error, FocusMode};
use crate::adapter::Adapter;
use crate::capability::Capabilities;
use crate::capability::resolve::{
    ResolvedExposure, ResolvedFrame, ResolvedPlacement, ResolvedPrepare,
};
use crate::capability::unsupported::{Feature, Unsupported};
use crate::decode::Image;
use crate::devices::Model;
use crate::scanners::{
    FilmHolder, Focus, ProgressFn, Scanner,
    ls5000::{
        Ls5000,
        boundaries::FrameBoundaries,
        calibration::DEFAULT_GAIN,
        geometry::{Dpi, Samples, ScanSettings, native_dots},
    },
    nikon::{ChannelExposures, metering::Metering},
};
use crate::scsi::Transport;
use tracing::*;

pub(super) struct Ls5000Driver {
    scanner: Ls5000<Box<dyn Transport + Send>>,
    frames: FrameBoundaries,
    gain: ChannelExposures,
    /// Set once the gain is settled, by a fixed exposure or by the one metering pass
    fixed: bool,
    lock_white_balance: bool,
}

impl Ls5000Driver {
    pub(super) fn open(transport: Box<dyn Transport + Send>, model: Model) -> Result<Self, Error> {
        let mut scanner = Ls5000::new(transport)?;
        super::confirm_model(scanner.identify()?, model)?;
        debug!(adapter = %scanner.adapter()?, name = ?scanner.adapter_name(), "Adapter");
        Ok(Self {
            scanner,
            frames: FrameBoundaries(Vec::new()),
            gain: DEFAULT_GAIN,
            fixed: false,
            lock_white_balance: false,
        })
    }

    fn dpi(&self, requested: u16) -> Result<Dpi, Error> {
        super::resolve_dpi(
            requested,
            &Dpi::ALL,
            self.scanner.capabilities().x_resolution,
            Dpi::to_dpi,
        )
    }

    /// Frames at an even pitch, for an adapter that senses none
    fn place_by_hand(&mut self, frames: Option<u32>, pitch_mm: Option<f32>) -> FrameBoundaries {
        let count = frames.unwrap_or_else(|| self.scanner.sensed_frames().max(1));
        let pitch = pitch_mm.map_or(self.scanner.capabilities().frame_pitch, native_dots);
        FrameBoundaries::evenly_spaced(count, pitch)
    }

    fn settings(&self, index: usize, settings: &ResolvedFrame) -> Result<ScanSettings, Error> {
        let capabilities = self.scanner.capabilities();
        Ok(ScanSettings {
            resolution: self.dpi(settings.dpi())?.to_dpi(),
            ir: settings.ir(),
            samples: samples(settings.multisample())?,
            window: self.frames.0[index].scan_area(capabilities),
            capabilities,
        })
    }
}

impl Driver for Ls5000Driver {
    fn capabilities(&mut self) -> Result<Capabilities, Error> {
        let adapter = self.scanner.adapter()?;
        Ok(super::reported_capabilities(
            Model::Ls5000,
            adapter,
            self.scanner.capabilities(),
            &Dpi::ALL,
            Dpi::to_dpi,
        ))
    }

    fn media_loaded(&mut self) -> Result<bool, Error> {
        Ok(self.scanner.adapter()? != Adapter::None)
    }

    fn prepare(
        &mut self,
        prepare: &ResolvedPrepare,
        _progress: &mut ProgressFn<'_>,
    ) -> Result<usize, Error> {
        self.scanner.warm_up()?;

        match prepare.exposure() {
            ResolvedExposure::HostMetered { lock_white_balance } => {
                self.lock_white_balance = lock_white_balance
            }
            // This model meters host-side, so resolution never hands it the firmware variant
            ResolvedExposure::FirmwareMetered => self.lock_white_balance = false,
            ResolvedExposure::Fixed { visible, ir } => {
                self.gain = ChannelExposures {
                    red: visible[0],
                    green: visible[1],
                    blue: visible[2],
                    // This model meters infrared rather than driving it off a zeroed field, so an
                    // omitted value keeps the default instead of blacking the plane out
                    ir: ir.unwrap_or(DEFAULT_GAIN.ir),
                };
                self.fixed = true;
            }
        }

        let mut frames = match prepare.placement() {
            ResolvedPlacement::Detect { .. } => {
                return Err(Unsupported::not_implemented(
                    Feature::Overview,
                    "no overview pass is written for this model",
                )
                .into());
            }
            ResolvedPlacement::Pitch {
                frames, pitch_mm, ..
            } => self.place_by_hand(*frames, *pitch_mm),
            // The feeder senses frames itself, so its table is the truth about where they are
            ResolvedPlacement::Sensed { frames } => match self.scanner.roll_table() {
                Ok(table) if !table.0.is_empty() => {
                    info!("Roll transport reports {} frames", table.0.len());
                    table
                }
                Ok(_) => self.place_by_hand(*frames, None),
                Err(e) => {
                    debug!(%e, "No roll transport table, placing frames on the pitch");
                    self.place_by_hand(*frames, None)
                }
            },
        };

        if let ResolvedPlacement::Pitch {
            frames: Some(n), ..
        }
        | ResolvedPlacement::Sensed { frames: Some(n) } = prepare.placement()
        {
            frames.0.truncate(*n as usize);
        }
        self.frames = frames;
        Ok(self.frames.0.len())
    }

    fn frames(&self) -> usize {
        self.frames.0.len()
    }

    fn sensed_frames(&mut self) -> Option<u32> {
        Some(self.scanner.sensed_frames())
    }

    fn scan_frame(
        &mut self,
        index: usize,
        settings: &ResolvedFrame,
        progress: &mut ProgressFn<'_>,
    ) -> Result<Image, Error> {
        let pass = self.settings(index, settings)?;

        match settings.focus() {
            FocusMode::Auto => {
                info!("Frame {index}: autofocusing");
                self.scanner.autofocus(pass.center())?;
            }
            // A setpoint skips the per-frame autofocus pass entirely
            FocusMode::At(setpoint) => self.scanner.set_focus(setpoint)?,
        }

        // Slow, and what it measures is the film rather than the frame, so it runs once and the
        // answer carries down the roll
        if !self.fixed {
            info!("Metering");
            let metering = Metering {
                lock_white_balance: self.lock_white_balance,
                ..Metering::default()
            };
            self.gain = self.scanner.autoexpose(pass.window, self.gain, metering)?;
            self.fixed = true;
            info!("Metered gain {}", self.gain);
        }

        info!(
            "Frame {index}: scanning at {} DPI, {}x sampled",
            pass.res(),
            pass.samples.count()
        );
        let gain = self.gain;
        Ok(self.scanner.scan_image_with(&pass, gain, progress)?)
    }

    fn lock_gain(&mut self) {
        self.fixed = true;
    }

    fn gain(&self) -> ChannelExposures {
        self.gain
    }

    fn eject(&mut self) -> Result<(), Error> {
        Ok(self.scanner.eject()?)
    }

    fn abort(&mut self) -> Result<(), Error> {
        // No vendor abort is characterized here, so a pass left half-read needs the handle
        // reopening rather than clearing in place
        Err(Unsupported::not_implemented(
            Feature::Abort,
            "no vendor abort is characterized here; drop the session to clear a pending pass",
        )
        .into())
    }
}

/// The repeat count as this model's window descriptor carries it
fn samples(count: u8) -> Result<Samples, Error> {
    Samples::new(count).ok_or_else(|| {
        Unsupported::not_one_of(
            Feature::Multisample,
            u32::from(count),
            Samples::ALL.iter().map(|&n| u32::from(n)),
        )
        .into()
    })
}
