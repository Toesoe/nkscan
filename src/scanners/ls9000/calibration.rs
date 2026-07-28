//! The pre-scan calibration pass
//!
//! A session-level preamble, not part of any scan. Nikon Scan runs it once, gating the first SCAN.

use super::{
    Ls9000,
    boundaries::FrameBoundaries,
    dtc::{self, Dtc},
    geometry::{CcdMode, Multisample},
    window::{BaseQuality, WindowKind, WindowParams},
};
use crate::{
    scanners::{
        Focus, ScanArea,
        nikon::{self, Channel, ChannelExposures},
    },
    scsi::{self, Transport},
};
use tracing::*;

/// This scanner's white balance, off the bare backlight through an empty holder
///
/// Equal gain does not scan neutral: the LEDs and the CCD are not equally strong across the
/// three bands, and red needs about 1.7x what blue does to match it.
///
/// The ratios are what matter. [`meter`] rescales the absolute anyway, and [`meter_locked`]
/// scales all three by one factor and cannot touch the ratios at all. It is a property of the
/// hardware, so [`white_balance`](super::Ls9000::white_balance) re-measures it per unit.
///
/// Infrared is only a starting point, and only used by a pass that does not meter it:
/// [`meter_ir`] measures it whenever one is captured. The dyes are near transparent in
/// infrared, so this is base density, which barely moves between frames.
pub const DEFAULT_GAIN: ChannelExposures = ChannelExposures {
    red: 283_048,
    green: 202_864,
    blue: 166_589,
    ir: 453_477,
};

/// The full-area window staged on every channel before calibrating, identical in all captures regardless of film format
fn calibration_window() -> ScanArea {
    ScanArea {
        x_pos: 0,
        y_pos: 0,
        x_size: ScanArea::FILM_WIDTH_DOTS,
        y_size: 13176,
    }
}

impl<T> Ls9000<T>
where
    T: Transport,
{
    /// The gain the scanner currently has staged, read back off its own window descriptors
    ///
    /// Any channel the scanner does not report keeps its [`Default`] value
    pub fn channel_exposures(&mut self) -> Result<ChannelExposures, scsi::Error> {
        let mut exposures = DEFAULT_GAIN;
        for descriptor in self.get_window(None)? {
            let Some(exposure) = nikon::exposure_from_vendor(&descriptor.vendor) else {
                continue;
            };
            // ScanArea 0 is the composite and carries no exposure of its own
            if let Some(channel) = Channel::from_id(descriptor.id).filter(|c| *c != Channel::All) {
                exposures.set(channel, exposure);
            }
        }
        Ok(exposures)
    }

    /// Run the pre-scan calibration, after the holder is loaded and before the first [`scan`](Self::scan)
    ///
    /// Writes the nominal frame table, because nothing is known about the real frames until an
    /// overview pass has found them. Follow that with
    /// [`set_frame_boundaries`](Self::set_frame_boundaries), as Nikon Scan does.
    pub fn calibrate(&mut self, exposures: ChannelExposures) -> Result<(), scsi::Error> {
        debug!("Calibrating");

        // Nikon Scan reads this per channel immediately before staging the windows. We don't
        // use the payload, but the read may be what latches the frame setup firmware-side.
        for channel in Channel::RGBI {
            let setup = self.read_framed_dtc(Dtc::FrameSetup, Some(channel), dtc::HEADER_LEN)?;
            trace!(?channel, len = setup.len(), "Frame setup");
        }

        // Stage a full-area window on every channel, IR included
        for channel in Channel::RGBI {
            let params = WindowParams {
                ccd: CcdMode::SingleLine,
                multisample: Multisample::X1,
                quality: BaseQuality::Scan,
                window_kind: WindowKind::Frame,
                exposure: exposures.get(channel),
            };
            self.set_window(channel, params.descriptor(4000, calibration_window()))?;
        }

        // Commit the staged focus by writing back whatever the scanner reports
        let focus = self.focus()?;
        self.set_focus(focus)?;

        // The scanner wants its current frame table read before it accepts a new one
        let current = self.frame_boundaries()?;
        trace!(?current, "Frame boundaries before calibration");
        self.set_frame_boundaries(&FrameBoundaries::nominal())?;

        // Nothing consumes these yet, but they're part of the observed sequence
        for channel in Channel::RGB {
            let channel = Some(channel);
            let dark = self.read_dtc(Dtc::DarkCurrent, channel, dtc::HEADER_LEN + 4)?;
            let line = self.read_framed_dtc(Dtc::ExtendedLine, channel, 20)?;
            trace!(?channel, ?dark, ?line, "Channel calibration readback");
        }

        Ok(())
    }
}

/// The metering itself is shared: this scanner and the LS-5000 both let the host decide the
/// gains from a low-resolution pass. Only the pass geometry and the defaults differ.
pub use crate::scanners::nikon::metering::{Metering, meter, meter_ir, meter_locked};
