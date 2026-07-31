//! The LS-50 ED behind a [`Session`](super::Session)
//!
//! Metering is firmware-side here, so there is no white balance lock and no preview to measure.
//! The frame table is re-declared on every pass, since one that cannot see the whole table leaves
//! the film where it is.

use super::{Driver, Error, Exposure, FocusMode, FrameSettings, Placement, Prepare};
use crate::decode::Image;
use crate::devices::{DeviceCapabilities, Model};
use crate::scanners::{
    FilmHolder, Focus, ProgressFn, Scanner,
    ls50::{
        Ls50,
        boundaries::FrameBoundaries,
        calibration::DEFAULT_GAIN,
        geometry::{Dpi, ScanSettings, native_dots},
        holder::Holder,
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
        debug!(holder = ?scanner.holder()?, adapter = ?scanner.adapter_name(), "Adapter");
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

    fn settings(&self, index: usize, settings: &FrameSettings) -> Result<ScanSettings, Error> {
        let capabilities = self.scanner.capabilities();
        Ok(ScanSettings {
            dpi: self.dpi(settings.dpi)?,
            ir: settings.ir,
            samples: 1,
            window: self.frames.0[index].scan_area(capabilities),
            capabilities,
        })
    }
}

impl Driver for Ls50Driver {
    fn capabilities(&self) -> DeviceCapabilities {
        super::reported_capabilities(
            Model::Ls50,
            self.scanner.capabilities(),
            &Dpi::ALL,
            Dpi::to_dpi,
        )
    }

    fn check(&self, prepare: &Prepare, settings: &FrameSettings) -> Result<(), Error> {
        self.dpi(settings.dpi)?;
        if settings.single_line {
            return Err(Error::Unsupported(
                "the single-line CCD is an LS-9000 option".into(),
            ));
        }
        if settings.multisample != 1 {
            return Err(Error::Unsupported(
                "multisampling is an LS-9000 option".into(),
            ));
        }
        match prepare.exposure {
            // The firmware meters this model, so there is no knob to hold the channels together
            Exposure::Auto {
                lock_white_balance: true,
            } => {
                return Err(Error::Unsupported(
                    "the white balance lock is not controllable on this model; fix the gain \
                     instead to hold the ratios"
                        .into(),
                ));
            }
            // Infrared is driven off a zeroed gain here, so a value for it means nothing
            Exposure::Fixed { ir: Some(_), .. } => {
                return Err(Error::Unsupported(
                    "this model drives infrared off a zeroed gain, so it takes three gains".into(),
                ));
            }
            _ => {}
        }
        if let Placement::Detect { .. } | Placement::Sensed { .. } = prepare.placement {
            return Err(Error::Unsupported(
                "this model neither senses frames nor has an overview pass; place them".into(),
            ));
        }
        super::reject_window(settings)
    }

    fn media_loaded(&mut self) -> Result<bool, Error> {
        Ok(self.scanner.holder()? != Holder::None)
    }

    fn prepare(
        &mut self,
        prepare: &Prepare,
        _progress: &mut ProgressFn<'_>,
    ) -> Result<usize, Error> {
        self.scanner.warm_up()?;

        if let Exposure::Fixed { visible, .. } = prepare.exposure {
            self.gain = ChannelExposures {
                red: visible[0],
                green: visible[1],
                blue: visible[2],
                ir: 0,
            };
            self.fixed = true;
        }

        let capabilities = self.scanner.capabilities();
        let (frames, pitch_mm, offsets_mm) = match &prepare.placement {
            Placement::Pitch {
                frames,
                pitch_mm,
                offsets_mm,
            } => (*frames, *pitch_mm, offsets_mm.as_slice()),
            _ => {
                return Err(Error::Unsupported(
                    "this model neither senses frames nor has an overview pass".into(),
                ));
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
        settings: &FrameSettings,
        progress: &mut ProgressFn<'_>,
    ) -> Result<Image, Error> {
        let pass = self.settings(index, settings)?;

        // Re-declared every pass: one that cannot see the whole table leaves the film where it is
        self.scanner.set_frame_boundaries(&self.frames)?;

        match settings.focus {
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

    fn eject(&mut self) -> Result<(), Error> {
        Ok(self.scanner.eject()?)
    }

    fn abort(&mut self) -> Result<(), Error> {
        // No vendor abort is characterized here, so a pass left half-read needs the handle
        // reopening rather than clearing in place
        Err(Error::Unsupported(
            "this model has no abort; drop the session to clear a pending pass".into(),
        ))
    }
}
