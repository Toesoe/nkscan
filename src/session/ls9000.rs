//! The LS-9000 ED behind a [`Session`](super::Session)
//!
//! Frames are found by an 83-DPI overview pass unless the caller placed them, exposure is metered
//! host-side on every frame, and the holder has to be in before anything moves.

use super::{Driver, Error, FocusMode, Placement};
use crate::adapter::Adapter;
use crate::capability::Capabilities;
use crate::capability::resolve::{ResolvedExposure, ResolvedFrame, ResolvedPrepare};
use crate::capability::unsupported::{Allowed, Feature, Unsupported};
use crate::decode::Image;
use crate::devices::Model;
use crate::scanners::{
    FilmHolder, Focus, ProgressFn, ReadError, ScanArea, Scanner,
    ls9000::{
        Ls9000,
        boundaries::{FrameBoundaries, FrameRect},
        calibration::{DEFAULT_GAIN, Metering},
        geometry::{CcdMode, Dpi, Multisample, ScanSettings, native_dots},
        window::BaseQuality,
    },
    nikon::ChannelExposures,
};
use crate::scsi::Transport;
use tracing::*;

pub(super) struct Ls9000Driver {
    scanner: Ls9000<Box<dyn Transport + Send>>,
    frames: FrameBoundaries,
    gain: ChannelExposures,
    /// Set once the gain is settled, by a fixed exposure or by [`Driver::lock_gain`]
    fixed: bool,
    /// Carried from [`Driver::prepare`], since metering runs per frame but is configured per strip
    lock_white_balance: bool,
}

impl Ls9000Driver {
    pub(super) fn open(transport: Box<dyn Transport + Send>, model: Model) -> Result<Self, Error> {
        let mut scanner = Ls9000::new(transport)?;
        super::confirm_model(scanner.identify()?, model)?;
        Ok(Self {
            scanner,
            frames: FrameBoundaries(Vec::new()),
            gain: DEFAULT_GAIN,
            fixed: false,
            lock_white_balance: false,
        })
    }

    /// The resolution to scan at, against this unit's own floor
    fn dpi(&self, requested: u16) -> Result<Dpi, Error> {
        super::resolve_dpi(
            requested,
            &Dpi::ALL,
            self.scanner.capabilities().x_resolution,
            Dpi::to_dpi,
        )
    }

    /// Block until film is loaded and the scanner has finished coming up
    ///
    /// The VPD page reports a holder the moment it is detected, but the scanner spends seconds
    /// afterwards initializing. That state is not a unit attention, so it survives the drain, and
    /// the first real command is refused several seconds later.
    /// Frames at a given pitch, skipping the overview entirely
    fn place_by_hand(
        &self,
        frames: Option<u32>,
        pitch_mm: Option<f32>,
        offsets_mm: &[f32],
    ) -> FrameBoundaries {
        let count = frames.unwrap_or(1);
        let pitch = pitch_mm
            .map(native_dots)
            .unwrap_or(ScanArea::STRIP_DOTS / count.max(1));
        debug!(pitch, count, "Placing frames by hand");
        FrameBoundaries(
            (0..count)
                .map(|i| {
                    let offset = offsets_mm
                        .get(i as usize)
                        .or(offsets_mm.last())
                        .copied()
                        .unwrap_or(0.0);
                    FrameRect::aligned(native_dots(offset) + i * pitch, pitch)
                })
                .collect(),
        )
    }

    /// An 83-DPI pass over the whole strip, then look for the frames in it
    fn search(
        &mut self,
        frames: usize,
        progress: &mut ProgressFn<'_>,
    ) -> Result<FrameBoundaries, Error> {
        info!("Taking an overview to find the frames");
        let (overview, _) = Driver::overview(self, progress)?;
        FrameBoundaries::detect(&overview.rgb, frames)
            .ok_or_else(|| Error::Media("no frames found on the strip".into()))
    }

    /// Classify a failed pass, since a holder pulled mid-scan reads as a transport fault
    fn diagnose(
        &mut self,
        error: ReadError<crate::scanners::ls9000::decode::DecodeError>,
    ) -> Error {
        if let ReadError::Cancelled = error {
            return Error::Cancelled;
        }
        if matches!(self.scanner.adapter(), Ok(Adapter::None)) {
            return Error::Media("the film holder was removed mid-pass".into());
        }
        error.into()
    }
}

impl Driver for Ls9000Driver {
    fn capabilities(&mut self) -> Result<Capabilities, Error> {
        let adapter = self.scanner.adapter()?;
        Ok(super::reported_capabilities(
            Model::Ls9000,
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
        progress: &mut ProgressFn<'_>,
    ) -> Result<usize, Error> {
        // A frame longer than the boundary the unit reports stalls the stage, and the window
        // check that also refuses it fires per pass — once the mechanism is already travelling.
        //
        // Placing by pitch is arithmetic, so it can be refused before anything moves, and it is
        // where the hazard is: asking for one frame without a pitch spreads it over the whole
        // strip, which is over twice the boundary. Detect cannot be checked this early because
        // its frames come out of the overview pass, and they are bounded by it.
        if let Placement::Pitch {
            frames,
            pitch_mm,
            offsets_mm,
        } = prepare.placement()
        {
            let boundary = self.scanner.capabilities().boundary_y;
            let placed = self.place_by_hand(*frames, *pitch_mm, offsets_mm);
            if let Some(long) = placed
                .0
                .iter()
                .find(|frame| frame.scan_area().y_size > boundary)
            {
                let asked = long.scan_area().y_size;
                warn!(
                    asked,
                    boundary, "Refusing a frame past the reported boundary"
                );
                return Err(Unsupported::out_of_range(
                    Feature::FramePlacement,
                    asked,
                    Allowed::Range {
                        min: 1,
                        max: boundary,
                    },
                )
                .into());
            }
        }

        // The session preamble, which gates the first scan
        self.scanner.calibrate(DEFAULT_GAIN)?;

        match prepare.exposure() {
            ResolvedExposure::HostMetered { lock_white_balance } => {
                self.lock_white_balance = lock_white_balance
            }
            // This model meters host-side, so resolution never hands it the firmware variant
            ResolvedExposure::FirmwareMetered => self.lock_white_balance = false,
            ResolvedExposure::Fixed { .. } => {}
        }
        if let ResolvedExposure::Fixed { visible, ir } = prepare.exposure() {
            self.gain = ChannelExposures {
                red: visible[0],
                green: visible[1],
                blue: visible[2],
                // Left alone rather than zeroed: this model meters infrared like any other channel
                ir: ir.unwrap_or(self.gain.ir),
            };
            self.fixed = true;
        }

        self.frames = match prepare.placement() {
            Placement::Detect { frames } => self.search(*frames, progress)?,
            Placement::Pitch {
                frames,
                pitch_mm,
                offsets_mm,
            } => self.place_by_hand(*frames, *pitch_mm, offsets_mm),
            Placement::Sensed { .. } => {
                return Err(
                    Unsupported::not_on_model(Feature::FramePlacement, Model::Ls9000).into(),
                );
            }
        };
        self.scanner.set_frame_boundaries(&self.frames)?;
        Ok(self.frames.0.len())
    }

    fn frames(&self) -> usize {
        self.frames.0.len()
    }

    fn scan_frame(
        &mut self,
        index: usize,
        settings: &ResolvedFrame,
        progress: &mut ProgressFn<'_>,
    ) -> Result<Image, Error> {
        let rect = self.frames.0[index];
        match settings.focus() {
            FocusMode::Auto => {
                info!("Frame {index}: autofocusing");
                self.scanner.autofocus(rect.center())?;
            }
            // A setpoint skips the per-frame autofocus pass entirely
            FocusMode::At(setpoint) => self.scanner.set_focus(setpoint)?,
        }

        let base = ScanSettings::autoexposure(rect.scan_area(), settings.ir());

        // Host-side and per frame, since it meters the frame's own content
        if !self.fixed {
            info!("Frame {index}: metering");
            let metering = Metering {
                lock_white_balance: self.lock_white_balance,
                ..Metering::default()
            };
            let (gain, _) = self
                .scanner
                .autoexpose_with(&base, DEFAULT_GAIN, &metering, &mut *progress)
                .map_err(|e| self.diagnose(e))?;
            info!("Frame {index}: metered gain {gain}");
            self.gain = gain;
        }

        let pass = ScanSettings {
            dpi: self.dpi(settings.dpi())?,
            quality: BaseQuality::Scan,
            ir: settings.ir(),
            multisample: multisample(settings.multisample())?,
            ccd_mode: if settings.single_line() {
                CcdMode::SingleLine
            } else {
                CcdMode::ThreeLine
            },
            ..base
        };

        info!("Frame {index}: scanning");
        let gain = self.gain;
        self.scanner
            .scan_image_with(&pass, gain, progress)
            .map_err(|e| self.diagnose(e))
    }

    fn lock_gain(&mut self) {
        self.fixed = true;
    }

    fn gain(&self) -> ChannelExposures {
        self.gain
    }

    /// Loading a holder raises a unit attention and starts the mechanism moving, and neither is
    /// finished by the time the holder first reports in
    fn overview(&mut self, progress: &mut ProgressFn<'_>) -> Result<(Image, u16), Error> {
        let rgb = self.scanner.overview_with(DEFAULT_GAIN, progress)?;
        // The pass divides the sensor by a fixed 48, so its resolution follows the optical one
        let dpi =
            self.scanner.capabilities().x_resolution.optical / ScanArea::OVERVIEW_DIVISOR as u16;
        Ok((Image { rgb, ir: None }, dpi))
    }

    fn after_media_ready(&mut self) -> Result<(), Error> {
        self.scanner.drain_unit_attentions()?;
        self.scanner.wait_until_ready()?;
        Ok(())
    }

    fn eject(&mut self) -> Result<(), Error> {
        Ok(self.scanner.eject()?)
    }

    fn abort(&mut self) -> Result<(), Error> {
        Ok(self.scanner.abort_scan()?)
    }
}

/// The repeat count as this model's window descriptor carries it
fn multisample(count: u8) -> Result<Multisample, Error> {
    match count {
        1 => Ok(Multisample::X1),
        2 => Ok(Multisample::X2),
        4 => Ok(Multisample::X4),
        8 => Ok(Multisample::X8),
        16 => Ok(Multisample::X16),
        _ => Err(
            Unsupported::not_one_of(Feature::Multisample, u32::from(count), [1, 2, 4, 8, 16])
                .into(),
        ),
    }
}
