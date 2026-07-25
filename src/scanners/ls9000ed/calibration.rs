//! The pre-scan calibration pass
//!
//! A session-level preamble, not part of any scan. Nikon Scan runs it once, gating the first SCAN.

use super::{
    Channel, Ls9000ed, Window,
    boundaries::FrameBoundaries,
    dtc::{self, Dtc},
    geometry::{CcdMode, Multisample},
    window::{BaseQuality, WindowKind, WindowParams},
};
use crate::scsi::{Error as ScsiError, Transport};
use tracing::*;

/// Per-channel exposure seeds for the calibration windows
///
/// Nikon Scan seeds these from the scanner's own window descriptors, which is what
/// [`Ls9000ed::channel_exposures`] reads. Autoexposure overwrites them later.
#[derive(Debug, Clone, Copy)]
pub struct ChannelExposures {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub ir: u32,
}

impl Default for ChannelExposures {
    /// Seeds from a capture. IR is the same in every session, RGB vary with the film,
    /// so treat those as a starting point rather than a correct value.
    fn default() -> Self {
        Self {
            red: 0x0001_14D9,
            green: 0x0000_C4F3,
            blue: 0x0000_A1CD,
            ir: 0x0001_6151,
        }
    }
}

impl ChannelExposures {
    /// In the order Nikon Scan stages them: R, G, B, then IR
    fn per_channel(self) -> [(Channel, u32); 4] {
        [
            (Channel::Red, self.red),
            (Channel::Green, self.green),
            (Channel::Blue, self.blue),
            (Channel::IR, self.ir),
        ]
    }
}

/// The full-area window staged on every channel before calibrating, identical in all captures regardless of film format
fn calibration_window() -> Window {
    Window {
        x_pos: 0,
        y_pos: 0,
        x_size: Window::FILM_WIDTH_DOTS,
        y_size: 13176,
    }
}

impl<T> Ls9000ed<T>
where
    T: Transport,
{
    /// The exposures the scanner currently has staged, read back off its own window descriptors
    ///
    /// Any channel the scanner doesn't report keeps its [`Default`] value.
    pub fn channel_exposures(&mut self) -> Result<ChannelExposures, ScsiError> {
        let mut exposures = ChannelExposures::default();
        for descriptor in self.get_window(None)? {
            let Some(tail) = descriptor.vendor.get(6..10) else {
                continue;
            };
            let exposure = u32::from_be_bytes(tail.try_into().expect("6..10 is four bytes"));
            match descriptor.id {
                1 => exposures.red = exposure,
                2 => exposures.green = exposure,
                3 => exposures.blue = exposure,
                9 => exposures.ir = exposure,
                _ => {}
            }
        }
        Ok(exposures)
    }

    /// Run the pre-scan calibration, after the holder is loaded and before the first [`scan`](Self::scan)
    ///
    /// `boundaries` is the frame table for the format being scanned. Nikon Scan writes its nominal
    /// table first and then overwrites it with the real format; we write only the real one.
    pub fn calibrate(
        &mut self,
        boundaries: &FrameBoundaries,
        exposures: ChannelExposures,
    ) -> Result<(), ScsiError> {
        debug!(?boundaries, "Calibrating");

        // Nikon Scan reads this per channel immediately before staging the windows. We don't
        // use the payload, but the read may be what latches the frame setup firmware-side.
        for (channel, _) in exposures.per_channel() {
            let setup = self.read_framed_dtc(Dtc::FrameSetup, Some(channel), dtc::HEADER_LEN)?;
            trace!(?channel, len = setup.len(), "Frame setup");
        }

        // Stage a full-area window on every channel, IR included
        for (channel, exposure) in exposures.per_channel() {
            let params = WindowParams {
                ccd: CcdMode::SingleLine,
                multisample: Multisample::X1,
                quality: BaseQuality::Scan,
                window_kind: WindowKind::Frame,
                exposure,
            };
            self.set_window(channel, params.descriptor(4000, calibration_window()))?;
        }

        // Commit the staged focus by writing back whatever the scanner reports
        let focus = self.get_focus()?;
        self.set_focus(focus)?;

        // The scanner wants its current frame table read before it accepts a new one
        let current = self.frame_boundaries()?;
        trace!(?current, "Frame boundaries before calibration");
        self.set_frame_boundaries(boundaries)?;

        // Nothing consumes these yet, but they're part of the observed sequence
        for channel in [Channel::Red, Channel::Green, Channel::Blue] {
            let channel = Some(channel);
            let dark = self.read_dtc(Dtc::DarkCurrent, channel, dtc::HEADER_LEN + 4)?;
            let line = self.read_framed_dtc(Dtc::ExtendedLine, channel, 20)?;
            trace!(?channel, ?dark, ?line, "Channel calibration readback");
        }

        Ok(())
    }
}
