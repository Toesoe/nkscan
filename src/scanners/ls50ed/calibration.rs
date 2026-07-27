//! Per-channel exposure, and the autoexposure pre-pass that measures it
//!
//! The firmware meters, not the host: a window armed in [`ScanMode::AutoExposure`] measures
//! without streaming an image, and the result comes back through GET WINDOW. There is nothing
//! host-side for a target or a percentile to act on.

use super::{Channel, Ls50ed, ScanSettings, window::ScanMode};
use crate::scsi::{self, Transport};
use tracing::*;

/// Per-channel analog gain, linear in the value and free in time. Infrared's window carries a
/// zeroed field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelExposures {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
}

impl Default for ChannelExposures {
    /// From a capture, and what a pass uses with autoexposure off. A calibration knob, not a
    /// constant of the hardware: tune per film and holder.
    fn default() -> Self {
        Self {
            red: 120_000,
            green: 120_000,
            blue: 100_000,
        }
    }
}

impl ChannelExposures {
    pub fn get(&self, channel: Channel) -> Option<u32> {
        match channel {
            Channel::Red => Some(self.red),
            Channel::Green => Some(self.green),
            Channel::Blue => Some(self.blue),
            Channel::Ir => None,
        }
    }

    fn set(&mut self, channel: Channel, exposure: u32) {
        match channel {
            Channel::Red => self.red = exposure,
            Channel::Green => self.green = exposure,
            Channel::Blue => self.blue = exposure,
            Channel::Ir => {}
        }
    }
}

impl<T> Ls50ed<T>
where
    T: Transport,
{
    /// The exposures the scanner currently has staged, out of its window descriptors
    pub fn channel_exposures(&mut self) -> Result<ChannelExposures, scsi::Error> {
        let mut exposures = ChannelExposures::default();
        for channel in Channel::RGB {
            exposures.set(channel, self.window_exposure(channel)?);
        }
        Ok(exposures)
    }

    /// One channel's exposure, from the vendor tail of its window descriptor
    fn window_exposure(&mut self, channel: Channel) -> Result<u32, scsi::Error> {
        let descriptors = self.get_window(Some(channel))?;
        Ok(descriptors
            .first()
            .and_then(|descriptor| super::WindowParams::exposure_from_vendor(&descriptor.vendor))
            .unwrap_or(0))
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

    #[test]
    fn infrared_has_no_exposure_of_its_own() {
        let exposures = ChannelExposures::default();
        assert_eq!(exposures.get(Channel::Red), Some(120_000));
        assert_eq!(exposures.get(Channel::Blue), Some(100_000));
        assert_eq!(exposures.get(Channel::Ir), None);
    }

    #[test]
    fn set_ignores_infrared() {
        let mut exposures = ChannelExposures::default();
        exposures.set(Channel::Green, 4242);
        exposures.set(Channel::Ir, 4242);
        assert_eq!(exposures.get(Channel::Green), Some(4242));
        assert_eq!(exposures.get(Channel::Ir), None);
    }
}
