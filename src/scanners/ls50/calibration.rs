//! Per-channel exposure, and the autoexposure pre-pass that measures it
//!
//! The firmware meters, not the host: a window armed in [`ScanMode::AutoExposure`] measures
//! without streaming an image, and the result comes back through GET WINDOW. There is nothing
//! host-side for a target or a percentile to act on.

use super::{Ls50, geometry::ScanSettings, window::ScanMode};
use crate::scanners::nikon::{Channel, ChannelExposures};
use crate::scsi::{self, Transport};
use tracing::*;

/// What a pass uses with autoexposure off. A calibration knob, not a constant of the hardware:
/// tune per film and holder.
///
/// Infrared stays zero, which is what this model's infrared window carries either way.
pub const DEFAULT_GAIN: ChannelExposures = ChannelExposures {
    red: 120_000,
    green: 120_000,
    blue: 100_000,
    ir: 0,
};

impl<T> Ls50<T>
where
    T: Transport,
{
    /// The exposures the scanner currently has staged, out of its window descriptors
    pub fn channel_exposures(&mut self) -> Result<ChannelExposures, scsi::Error> {
        let mut exposures = DEFAULT_GAIN;
        for channel in Channel::RGB {
            exposures.set(channel, self.window_exposure(channel)?);
        }
        Ok(exposures)
    }

    /// One channel's exposure, from the vendor tail of its window descriptor
    ///
    /// Matched on the descriptor's own id rather than taken from the front of the list: a
    /// firmware that ignored the single-window request would otherwise report window 1's
    /// exposure for all three channels. An absent tail is an error rather than a zero, since
    /// zero is a gain the caller would go on to arm a black pass with.
    fn window_exposure(&mut self, channel: Channel) -> Result<u32, scsi::Error> {
        self.get_window(Some(channel))?
            .iter()
            .find(|descriptor| descriptor.id == channel.to_id())
            .and_then(|descriptor| crate::scanners::nikon::exposure_from_vendor(&descriptor.vendor))
            .ok_or(scsi::Error::InvalidResponse(
                "no window descriptor carried an exposure for the channel asked for",
            ))
    }

    /// Find the exposure that fills the range, by letting the firmware measure the window
    ///
    /// Arms in AE mode, scans, and reads back what it landed on. The pass streams no image, so
    /// there is nothing to drain. The firmware re-measures only when its calibration is stale
    /// and otherwise hands `from` straight back.
    pub fn autoexpose(
        &mut self,
        settings: &ScanSettings,
        from: ChannelExposures,
    ) -> Result<ChannelExposures, scsi::Error> {
        // Measure the visible channels only; infrared keeps its fixed exposure
        let measurement = ScanSettings {
            ir: false,
            ..*settings
        };
        self.arm(&measurement, from, ScanMode::AutoExposure)?;
        self.scan(&Channel::RGB)?;
        // Registers only settle at the end; reading early gives a half-measured answer
        self.wait_until_ready()?;

        let measured = self.channel_exposures()?;
        if measured == from {
            warn!("Autoexposure: firmware skipped the measurement, using the seed");
            return Ok(from);
        }
        debug!(?measured, "Autoexposure measured");
        Ok(measured)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This model drives infrared off a zeroed gain field
    #[test]
    fn infrared_has_no_exposure_of_its_own() {
        assert_eq!(DEFAULT_GAIN.get(Channel::Ir), 0);
    }
}
