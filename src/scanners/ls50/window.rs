//! Vendor-specific window descriptor fields, LS-50 ED

use super::geometry::ScanSettings;
use crate::scanners::nikon::Channel;
use crate::scsi::cdbs::{CompressionType, ImageCompositionCode, PaddingType, WindowDescriptor};

/// What the scanner does with the pass, byte 2 of the vendor tail
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ScanMode {
    Normal,
    /// Measures exposure and streams nothing (SANE's pre-pass)
    AutoExposure,
    /// Only 1 works: anything higher arms a multi-pass scan that never streams
    Samples(u8),
}

impl ScanMode {
    fn to_byte(self) -> u8 {
        match self {
            ScanMode::Normal => 0x01,
            ScanMode::AutoExposure => 0x20,
            ScanMode::Samples(n) => n.max(1),
        }
    }
}

/// The vendor-specific portion of a window descriptor
#[derive(Debug, Copy, Clone)]
pub struct WindowParams {
    pub mode: ScanMode,
    /// Analog gain, linear in the value and free in time. Zero for infrared, which gets none.
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
        let mut buf = [0u8; 10];
        buf[0] = 0x00;
        buf[1] = 0x81; // 0x80 | positive
        buf[2] = value.mode.to_byte();
        buf[3] = 0x02; // compression: none
        buf[4] = 0x02; // color interleaving
        buf[5] = 0xFF; // autoexposure enable
        buf[6..10].copy_from_slice(&value.exposure.to_be_bytes());
        buf.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanners::{ScanArea, ls50::geometry::Dpi, nikon::exposure_from_vendor};

    fn settings(dpi: Dpi) -> ScanSettings {
        let capabilities = crate::scanners::ls50::capabilities::fixture::capabilities();
        ScanSettings {
            dpi,
            ir: false,
            samples: 1,
            window: ScanArea::frame(0, capabilities),
            capabilities,
        }
    }

    fn params(mode: ScanMode) -> WindowParams {
        WindowParams {
            mode,
            exposure: 120_000,
        }
    }

    /// The 50 bytes the scanner has actually been driven with
    #[test]
    fn descriptor_matches_the_driven_layout() {
        let settings = settings(Dpi::_1000);
        let bytes = params(ScanMode::Normal)
            .descriptor(&settings, Channel::Red)
            .to_bytes();
        let (native_width, native_height) = settings.native_dims();

        assert_eq!(bytes.len(), 50);
        assert_eq!(bytes[0], 0x01); // channel
        assert_eq!(bytes[1], 0x00);
        assert_eq!(u16::from_be_bytes([bytes[2], bytes[3]]), settings.res());
        assert_eq!(u16::from_be_bytes([bytes[4], bytes[5]]), settings.res());
        assert_eq!(&bytes[6..14], &[0u8; 8]); // x and y offset at the origin
        assert_eq!(
            u32::from_be_bytes(bytes[14..18].try_into().unwrap()),
            native_width
        );
        assert_eq!(
            u32::from_be_bytes(bytes[18..22].try_into().unwrap()),
            native_height
        );
        assert_eq!(bytes[25], 0x05); // RGB
        assert_eq!(bytes[26], 0x0E); // 14-bit
        assert_eq!(
            &bytes[40..],
            &[0x00, 0x81, 0x01, 0x02, 0x02, 0xFF, 0x00, 0x01, 0xD4, 0xC0]
        );
    }

    #[test]
    fn autoexposure_pass_flips_the_mode_byte() {
        let settings = settings(Dpi::_1000);
        let ae = params(ScanMode::AutoExposure)
            .descriptor(&settings, Channel::Red)
            .to_bytes();
        let normal = params(ScanMode::Normal)
            .descriptor(&settings, Channel::Red)
            .to_bytes();
        assert_eq!(ae[42], 0x20);
        assert_eq!(normal[42], 0x01);
    }

    /// The count reaches the mode byte unchanged; whether the firmware reads it as a literal
    /// or as a coded value is unverified
    #[test]
    fn the_sample_count_reaches_the_mode_byte_verbatim() {
        let bytes = params(ScanMode::Samples(4))
            .descriptor(&settings(Dpi::_1000), Channel::Red)
            .to_bytes();
        assert_eq!(bytes[42], 0x04);
        assert_eq!(bytes[40], 0x00);
        assert_eq!(bytes[43], 0x02);
    }

    #[test]
    fn infrared_gets_no_exposure() {
        let bytes = WindowParams {
            exposure: 0,
            ..params(ScanMode::Normal)
        }
        .descriptor(&settings(Dpi::_1000), Channel::Ir)
        .to_bytes();
        assert_eq!(bytes[0], 0x09);
        assert_eq!(&bytes[46..50], &[0u8; 4]);
    }

    #[test]
    fn frame_selection_rides_the_window_origin() {
        let capabilities = crate::scanners::ls50::capabilities::fixture::capabilities();
        let pitch = capabilities.frame_pitch;
        let settings = ScanSettings {
            window: ScanArea::frame(pitch, capabilities),
            ..settings(Dpi::_1000)
        };
        let bytes = params(ScanMode::Normal)
            .descriptor(&settings, Channel::Red)
            .to_bytes();
        assert_eq!(&bytes[6..10], &[0u8; 4]); // x stays at the origin
        assert_eq!(u32::from_be_bytes(bytes[10..14].try_into().unwrap()), pitch);
    }

    /// GET WINDOW hands the measured exposure back in the same tail slot
    #[test]
    fn exposure_round_trips_through_the_vendor_tail() {
        let vendor: Vec<u8> = params(ScanMode::Normal).into();
        assert_eq!(exposure_from_vendor(&vendor), Some(120_000));
        assert_eq!(exposure_from_vendor(&vendor[..4]), None);
    }
}
