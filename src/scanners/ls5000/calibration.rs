//! Per-channel exposure, and the metering pass that measures it
//!
//! The host meters, not the firmware: the scanner runs an ordinary pass at
//! [`METER_RESOLUTION`] with the autoexposure byte set, and what comes back is decoded here and
//! turned into gains. The mechanism and its settings are the shared ones in
//! [`nikon::metering`](crate::scanners::nikon::metering), unchanged; what belongs here is the
//! geometry the metering pass runs over.

use super::{
    Ls5000, ScanError,
    geometry::{METER_RESOLUTION, Samples, ScanSettings},
};
use crate::scanners::{
    ScanArea,
    nikon::{
        Channel, ChannelExposures,
        metering::{Metering, meter, meter_ir, meter_locked},
    },
};
use crate::scsi::{self, Transport};
use tracing::*;

/// What a pass uses with metering off
///
/// A starting point rather than a constant of the hardware: tune per film and holder. Metering
/// rescales the absolute anyway, so only the ratios matter much.
pub const DEFAULT_GAIN: ChannelExposures = ChannelExposures {
    red: 90_000,
    green: 180_000,
    blue: 165_000,
    ir: 280_000,
};

impl<T> Ls5000<T>
where
    T: Transport,
{
    /// The exposures the scanner currently has staged, out of its window descriptors
    pub fn channel_exposures(&mut self) -> Result<ChannelExposures, scsi::Error> {
        let mut exposures = DEFAULT_GAIN;
        for descriptor in self.get_window(None)? {
            let Some(exposure) = crate::scanners::nikon::exposure_from_vendor(&descriptor.vendor)
            else {
                continue;
            };
            if let Some(channel) = Channel::from_id(descriptor.id).filter(|c| *c != Channel::All) {
                exposures.set(channel, exposure);
            }
        }
        Ok(exposures)
    }

    /// The settings one metering pass runs at: the whole frame, low resolution, infrared on
    ///
    /// Infrared is captured because it is metered too, and it is the plane a dust pass depends
    /// on getting right.
    pub fn metering_settings(&self, frame: ScanArea) -> ScanSettings {
        ScanSettings {
            resolution: METER_RESOLUTION,
            ir: true,
            samples: Samples::default(),
            window: frame,
            capabilities: self.capabilities(),
        }
    }

    /// Meter `frame`, starting from `from`
    ///
    /// Each pass is a real scan whose image is decoded and thrown away once the gains are off
    /// it. Gain is linear, so one pass lands close and the next measures from where it got to.
    ///
    /// Nothing bounds the result: what the firmware will take is not known, and a gain capped
    /// on the way to the scanner would come back as an underexposed frame with nothing in the
    /// log to say why.
    pub fn autoexpose(
        &mut self,
        frame: ScanArea,
        from: ChannelExposures,
        metering: Metering,
    ) -> Result<ChannelExposures, ScanError> {
        let settings = self.metering_settings(frame);
        let mut gain = from;

        for pass in 0..metering.passes.max(1) {
            let image = self.scan_image(&settings, gain)?;
            let visible = if metering.lock_white_balance {
                meter_locked(&image.rgb, gain, metering.percentile, metering.target)
            } else {
                meter(&image.rgb, gain, metering.percentile, metering.target)
            };
            gain = match &image.ir {
                Some(ir) => meter_ir(ir, visible, metering.percentile, metering.target),
                None => visible,
            };
            debug!(pass, ?gain, "Metered");
        }
        Ok(gain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Metering rescales from whatever it is given, so a zeroed seed would leave a channel with
    /// nothing to scale against and stick there
    #[test]
    fn every_channel_seeds_at_something_metering_can_scale() {
        for channel in Channel::RGBI {
            assert!(DEFAULT_GAIN.get(channel) > 0, "{channel:?}");
        }
    }
}
