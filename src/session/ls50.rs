//! The LS-50 ED behind a [`Session`](super::Session)
//!
//! Metering is firmware-side here, so there is no white balance lock and no preview to measure.
//! The frame table is re-declared on every pass, since one that cannot see the whole table leaves
//! the film where it is.

use super::{Driver, Error, FocusMode, Placement};
use crate::adapter::Adapter;
use crate::capability::Capabilities;
use crate::capability::resolve::{ResolvedExposure, ResolvedFrame, ResolvedPrepare};
use crate::capability::unsupported::{Feature, Unsupported};
use crate::decode::Image;
use crate::devices::Model;
use crate::scanners::ScanArea;
use crate::scanners::ls50::geometry;
use crate::scanners::nikon::usb::UsbCoolscan;
use crate::scanners::{
    FilmHolder, Focus, ProgressFn, Scanner,
    ls50::{
        Ls50,
        boundaries::FrameBoundaries,
        calibration::DEFAULT_GAIN,
        geometry::{Dpi, ScanSettings, native_dots},
    },
    nikon::ChannelExposures,
};
use crate::scsi::Transport;
use tracing::*;

pub(super) struct Ls50Driver {
    scanner: Ls50<Box<dyn Transport + Send>>,
    frames: FrameBoundaries,
    gain: ChannelExposures,
    /// Set once the gain is settled, by a fixed exposure or by the one metering pass
    fixed: bool,
}

impl Ls50Driver {
    pub(super) fn open(transport: Box<dyn Transport + Send>, model: Model) -> Result<Self, Error> {
        let mut scanner = Ls50::new(transport)?;
        super::confirm_model(scanner.identify()?, model)?;
        debug!(adapter = %scanner.adapter()?, name = ?scanner.adapter_name(), "Adapter");
        Ok(Self {
            scanner,
            frames: FrameBoundaries(Vec::new()),
            gain: DEFAULT_GAIN,
            fixed: false,
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

    fn settings(&self, index: usize, settings: &ResolvedFrame) -> Result<ScanSettings, Error> {
        let capabilities = self.scanner.capabilities();
        Ok(ScanSettings {
            dpi: self.dpi(settings.dpi())?,
            ir: settings.ir(),
            samples: 1,
            window: self.frames.0[index].scan_area(capabilities),
            capabilities,
        })
    }
}

impl Driver for Ls50Driver {
    fn capabilities(&mut self) -> Result<Capabilities, Error> {
        let adapter = self.scanner.adapter()?;
        Ok(super::reported_capabilities(
            Model::Ls50,
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

        if let ResolvedExposure::Fixed { visible, .. } = prepare.exposure() {
            self.gain = ChannelExposures {
                red: visible[0],
                green: visible[1],
                blue: visible[2],
                ir: 0,
            };
            self.fixed = true;
        }

        let capabilities = self.scanner.capabilities();
        let (frames, pitch_mm, offsets_mm) = match prepare.placement() {
            Placement::Pitch {
                frames,
                pitch_mm,
                offsets_mm,
            } => (*frames, *pitch_mm, offsets_mm.as_slice()),
            // Resolution allows both where the hardware has them; neither pass is written here
            Placement::Detect { .. } => {
                return Err(Unsupported::not_implemented(
                    Feature::Overview,
                    "no overview pass is written for this model",
                )
                .into());
            }
            Placement::Sensed { .. } => {
                return Err(Unsupported::not_on_model(Feature::FramePlacement, Model::Ls50).into());
            }
        };

        let count = frames.unwrap_or_else(|| self.scanner.sensed_frames().max(1));
        let pitch = pitch_mm.map_or(capabilities.frame_pitch, native_dots);
        let offsets: &[f32] = if offsets_mm.is_empty() {
            &[0.0]
        } else {
            offsets_mm
        };

        self.frames = FrameBoundaries::evenly_spaced(count, pitch, offsets, capabilities.max_x());
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

        // Re-declared every pass: one that cannot see the whole table leaves the film where it is
        self.scanner.set_frame_boundaries(&self.frames)?;

        match settings.focus() {
            FocusMode::Auto => {
                info!("Frame {index}: autofocusing");
                self.scanner.autofocus(pass.center())?;
            }
            // A setpoint skips the per-frame autofocus pass entirely
            FocusMode::At(setpoint) => self.scanner.set_focus(setpoint)?,
        }

        // Firmware-side and slow, and what it measures is the film rather than the frame, so it
        // runs once and the answer carries down the strip
        if !self.fixed {
            info!("Metering");
            self.gain = self.scanner.autoexpose(&pass, self.gain)?;
            self.fixed = true;
            info!("Metered gain {}", self.gain);
        }

        info!("Frame {index}: scanning at {} DPI", pass.res());
        let gain = self.gain;
        Ok(self.scanner.scan_image_with(&pass, gain, progress)?)
    }

    fn lock_gain(&mut self) {
        self.fixed = true;
    }

    fn gain(&self) -> ChannelExposures {
        self.gain
    }

    fn overview(&mut self, progress: &mut ProgressFn<'_>) -> Result<(Image, u16), Error> {
        let capabilities = self.scanner.capabilities();
        let target_dpi = self.dpi(97)?;

        let settings = ScanSettings {
            dpi: target_dpi,
            ir: false,
            samples: 1,
            window: ScanArea {
                x_pos: 0,
                y_pos: 0,
                x_size: capabilities.max_x(), // 3946
                y_size: capabilities.preview_roll_length(), // 250,278
            },
            capabilities,
        };

        let image = self.scanner.preview_roll(&settings, progress)?;

        Ok((image, 0u16))
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
