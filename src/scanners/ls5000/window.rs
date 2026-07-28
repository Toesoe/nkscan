//! Vendor-specific window descriptor fields, LS-5000 ED

use super::geometry::{Samples, ScanSettings};
use crate::scanners::nikon::Channel;
use crate::scsi::cdbs::{CompressionType, ImageCompositionCode, PaddingType, WindowDescriptor};

/// The scanning kind, byte 2 of the tail
///
/// Every window driven for a frame carries this, metering included: metering is an ordinary
/// pass with the autoexposure byte set, not a kind of its own.
const SCANNING_KIND_NORMAL: u8 = 0x01;

/// The vendor-specific portion of a window descriptor
#[derive(Debug, Copy, Clone)]
pub struct WindowParams {
    /// Passes the sensor averages in hardware
    pub samples: Samples,
    /// Per-channel exposure, as [`ChannelExposures`](crate::scanners::nikon::ChannelExposures)
    /// carries it. What the number physically is has not been established.
    pub exposure: u32,
}

impl WindowParams {
    /// The SET WINDOW descriptor for `channel`
    ///
    /// Every pass is multi-level RGB with no halftoning, padding or compression, so those are
    /// fixed. Dimensions go in native units, not output pixels.
    pub fn descriptor(self, settings: &ScanSettings, channel: Channel) -> WindowDescriptor {
        let (native_width, native_height) = settings.native_dims();
        let resolution = settings.res();
        WindowDescriptor {
            id: channel.to_id(),
            auto: false,
            x_resolution: resolution,
            y_resolution: resolution,
            x_upper_left: settings.window.x_pos,
            y_upper_left: settings.window.y_pos,
            width: native_width,
            length: native_height,
            brightness: 0,
            threshold: 0,
            contrast: 0,
            composition: ImageCompositionCode::Rgb,
            bits_per_pixel: settings.capabilities.max_bits,
            halftone_pattern: 0,
            rif: false,
            padding: PaddingType::NoPadding,
            bit_ordering: 0,
            compression: CompressionType::NoCompression,
            compression_arg: 0,
            vendor: self.into(),
        }
    }
}

impl From<WindowParams> for Vec<u8> {
    fn from(value: WindowParams) -> Self {
        let multi = value.samples.is_multi();
        let mut buf = [0u8; 10];
        // Sample count less one. This is what arms multi-sampling; the scanning kind at byte 2
        // stays normal throughout.
        buf[0] = if multi {
            (value.samples.count() - 1) << 4
        } else {
            0x00
        };
        // Uncharacterized, and reproduced rather than understood: 0x80 on a single-sampled
        // window and 0x00 on a multi-sampled one
        buf[1] = if multi { 0x00 } else { 0x80 };
        buf[2] = SCANNING_KIND_NORMAL;
        // Both move with the sample count, and the pass does not stream without them
        buf[3] = if multi { 0x10 } else { 0x02 };
        buf[4] = if multi { 0x40 } else { 0x02 };
        buf[5] = 0xFF; // autoexposure enable, set on every window driven here
        buf[6..10].copy_from_slice(&value.exposure.to_be_bytes());
        buf.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanners::{
        ls5000::{capabilities::fixture, geometry::whole_frame},
        nikon::exposure_from_vendor,
    };

    fn settings(resolution: u16, samples: u8) -> ScanSettings {
        let capabilities = fixture::capabilities();
        ScanSettings {
            resolution,
            ir: false,
            samples: Samples::new(samples).unwrap(),
            window: whole_frame(0, capabilities),
            capabilities,
        }
    }

    fn params(samples: u8, exposure: u32) -> WindowParams {
        WindowParams {
            samples: Samples::new(samples).unwrap(),
            exposure,
        }
    }

    /// The 50 bytes a single-sampled window is driven with
    #[test]
    fn descriptor_matches_the_driven_layout() {
        let settings = settings(4000, 1);
        let bytes = params(1, 68_799)
            .descriptor(&settings, Channel::Red)
            .to_bytes();

        assert_eq!(bytes.len(), 50);
        assert_eq!(bytes[0], 0x01); // red
        assert_eq!(u16::from_be_bytes([bytes[2], bytes[3]]), 4000);
        assert_eq!(u16::from_be_bytes([bytes[4], bytes[5]]), 4000);
        assert_eq!(&bytes[6..14], &[0u8; 8]); // at the origin
        let (width, length) = settings.native_dims();
        assert_eq!(u32::from_be_bytes(bytes[14..18].try_into().unwrap()), width);
        assert_eq!(
            u32::from_be_bytes(bytes[18..22].try_into().unwrap()),
            length
        );
        assert_eq!(bytes[25], 0x05); // RGB
        assert_eq!(bytes[26], 0x10); // 16-bit
        assert_eq!(
            &bytes[40..],
            &[0x00, 0x80, 0x01, 0x02, 0x02, 0xFF, 0x00, 0x01, 0x0C, 0xBF]
        );
    }

    /// The 50 bytes a four-times multi-sampled window is driven with
    #[test]
    fn a_multi_sampled_descriptor_matches_the_driven_layout() {
        let bytes = params(4, 314_216)
            .descriptor(
                &ScanSettings {
                    ir: true,
                    ..settings(4000, 4)
                },
                Channel::Ir,
            )
            .to_bytes();

        assert_eq!(bytes[0], 0x09); // infrared
        assert_eq!(
            &bytes[40..],
            &[0x30, 0x00, 0x01, 0x10, 0x40, 0xFF, 0x00, 0x04, 0xCB, 0x68]
        );
    }

    /// The count is the nibble, not the scanning kind. Putting it in the kind arms a pass that
    /// never streams, which costs the idle timeout rather than raising an error.
    #[test]
    fn the_sample_count_rides_the_nibble_and_never_the_scanning_kind() {
        for (count, nibble) in [(1u8, 0x00u8), (2, 0x10), (4, 0x30), (8, 0x70), (16, 0xF0)] {
            let tail: Vec<u8> = params(count, 0).into();
            assert_eq!(tail[0], nibble, "{count} samples");
            assert_eq!(
                tail[2], SCANNING_KIND_NORMAL,
                "{count} samples changed the scanning kind"
            );
            let multi = count > 1;
            assert_eq!(tail[3], if multi { 0x10 } else { 0x02 });
            assert_eq!(tail[4], if multi { 0x40 } else { 0x02 });
        }
    }

    /// The resolution reaches the descriptor unrounded, so a metering pass off the DPI ladder
    /// is expressible
    #[test]
    fn an_off_ladder_resolution_reaches_the_descriptor() {
        let bytes = params(1, 0)
            .descriptor(&settings(285, 1), Channel::Red)
            .to_bytes();
        assert_eq!(u16::from_be_bytes([bytes[2], bytes[3]]), 285);
        assert_eq!(u16::from_be_bytes([bytes[4], bytes[5]]), 285);
    }

    #[test]
    fn infrared_carries_its_own_exposure() {
        let bytes = params(1, 296_145)
            .descriptor(&settings(4000, 1), Channel::Ir)
            .to_bytes();
        assert_eq!(bytes[0], 0x09);
        assert_eq!(
            u32::from_be_bytes(bytes[46..50].try_into().unwrap()),
            296_145
        );
    }

    /// GET WINDOW hands the measured exposure back in the same tail slot
    #[test]
    fn exposure_round_trips_through_the_vendor_tail() {
        let vendor: Vec<u8> = params(1, 120_000).into();
        assert_eq!(exposure_from_vendor(&vendor), Some(120_000));
        assert_eq!(exposure_from_vendor(&vendor[..4]), None);
    }
}
