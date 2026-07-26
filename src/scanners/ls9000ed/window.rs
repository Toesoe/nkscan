//! Vendor-specific window descriptor fields, LS-9000ED

use super::{BITS_PER_PIXEL, CcdMode, Multisample, ScanArea};
use crate::scsi::cdbs::{CompressionType, ImageCompositionCode, PaddingType, WindowDescriptor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Whether the stage steps at the full sensor rate or half of it
///
/// Nikon Scan pairs `Scan` with a square window and `Preview` with a half-height one. The
/// 83-DPI whole-strip overview is a `Scan`, square-sampled at 83x83.
pub enum BaseQuality {
    /// Square sampling: the 4000-DPI scan and the 83-DPI overview
    Scan,
    /// Half-rate stage stepping: the 666x333 metering and preview pass
    Preview,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// What kind of pass this window sets up
pub enum WindowKind {
    /// A window into a particular frame
    Frame,
    /// The whole viewport
    Overview,
}

/// The vendor-specific portion of a window descriptor
#[derive(Debug, Clone, Copy)]
pub struct WindowParams {
    pub ccd: CcdMode,
    pub multisample: Multisample,
    pub quality: BaseQuality,
    pub window_kind: WindowKind,
    /// Per-channel analog gain, linear in the value and free in time
    ///
    /// 32x the value saturates the ADC while the scan still takes 889 ms, so this amplifies
    /// rather than integrating longer. Nikon Scan's Analog Gain palette is the same knob in
    /// EV. Persists across sessions, so a readback is whatever was last written.
    pub exposure: u32,
}

impl WindowParams {
    /// Y resolution follows from the sampling mode: every captured `Scan` is square, every `Preview` halves Y
    fn y_resolution(self, x_resolution: u16) -> u16 {
        match self.quality {
            BaseQuality::Scan => x_resolution,
            BaseQuality::Preview => x_resolution / 2,
        }
    }

    /// Build the SET WINDOW descriptor for `window` at `x_resolution` DPI
    ///
    /// This scanner only ever does multi-level RGB at 16 bits with no halftoning, padding or
    /// compression, so those are fixed. The id is left for [`set_window`](super::Ls9000ed::set_window).
    pub fn descriptor(self, x_resolution: u16, window: ScanArea) -> WindowDescriptor {
        WindowDescriptor {
            id: 0,
            auto: false,
            x_resolution,
            y_resolution: self.y_resolution(x_resolution),
            x_upper_left: window.x_pos,
            y_upper_left: window.y_pos,
            width: window.x_size,
            length: window.y_size,
            brightness: 0,
            threshold: 0,
            contrast: 0,
            composition: ImageCompositionCode::Rgb,
            bits_per_pixel: BITS_PER_PIXEL,
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

        // High nibble is the multi-sample repeat count minus one
        buf[0] = ((value.multisample.count() - 1) << 4) as u8;

        // Bit 7 is averaging, and tracks the sampling mode: square sampling is (0x81, 0x02),
        // the half-height prescan (0x01, 0x04).
        //
        // Bit 0 is positive film on other Coolscans. Clearing it here is accepted and reads
        // back cleared, but the image is identical, so leave it set.
        buf[1] = match value.quality {
            BaseQuality::Scan => 0x81,
            BaseQuality::Preview => 0x01,
        };
        buf[2] = match value.window_kind {
            WindowKind::Frame => 0x01,
            // Only the 83-DPI whole-strip overview
            WindowKind::Overview => 0x02,
        };
        // Bit 4 is "multi-sampling on", always set exactly when buf[0]'s high nibble is nonzero
        buf[3] = match value.quality {
            BaseQuality::Scan => 0x02,
            BaseQuality::Preview => 0x04,
        } | if value.multisample.count() > 1 {
            0x10
        } else {
            0x00
        };
        buf[4] = match value.ccd {
            CcdMode::SingleLine => 0x02,
            CcdMode::ThreeLine => 0x40,
        };
        // 0xFF in all captures
        buf[5] = 0xFF;
        buf[6..10].copy_from_slice(&value.exposure.to_be_bytes());

        buf.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(quality: BaseQuality) -> WindowParams {
        WindowParams {
            ccd: CcdMode::ThreeLine,
            multisample: Multisample::X1,
            quality,
            window_kind: WindowKind::Frame,
            exposure: 0,
        }
    }

    /// 6x4.5 frame, 1x, three-line CCD, 4000 DPI, red channel
    #[test]
    fn matches_captured_full_resolution_scan() {
        let bytes: Vec<u8> = WindowParams {
            exposure: 0x0007_ABDD,
            ..params(BaseQuality::Scan)
        }
        .into();
        assert_eq!(
            bytes,
            [0x00, 0x81, 0x01, 0x02, 0x40, 0xFF, 0x00, 0x07, 0xAB, 0xDD]
        );
    }

    /// same but 16x multi-sample
    #[test]
    fn matches_captured_16x_scan() {
        let bytes: Vec<u8> = WindowParams {
            multisample: Multisample::X16,
            exposure: 0x0007_55AB,
            ..params(BaseQuality::Scan)
        }
        .into();
        assert_eq!(
            bytes,
            [0xF0, 0x81, 0x01, 0x12, 0x40, 0xFF, 0x00, 0x07, 0x55, 0xAB]
        );
    }

    /// The 666x333 prescan, 16x multi-sample
    #[test]
    fn matches_captured_prescan() {
        let bytes: Vec<u8> = WindowParams {
            multisample: Multisample::X16,
            exposure: 0x0007_55AB,
            ..params(BaseQuality::Preview)
        }
        .into();
        assert_eq!(
            bytes,
            [0xF0, 0x01, 0x01, 0x14, 0x40, 0xFF, 0x00, 0x07, 0x55, 0xAB]
        );
    }

    /// "super fine" single-line CCD at 4000 DPI
    #[test]
    fn matches_captured_singleline_scan() {
        let bytes: Vec<u8> = WindowParams {
            ccd: CcdMode::SingleLine,
            exposure: 0x000A_8212,
            ..params(BaseQuality::Scan)
        }
        .into();
        assert_eq!(
            bytes,
            [0x00, 0x81, 0x01, 0x02, 0x02, 0xFF, 0x00, 0x0A, 0x82, 0x12]
        );
    }

    /// The 83-DPI whole-strip overview square-sampled, so it's a Scan, and single-line CCD
    #[test]
    fn matches_captured_overview() {
        let bytes: Vec<u8> = WindowParams {
            ccd: CcdMode::SingleLine,
            window_kind: WindowKind::Overview,
            exposure: 0x0005_E9CA,
            ..params(BaseQuality::Scan)
        }
        .into();
        assert_eq!(
            bytes,
            [0x00, 0x81, 0x02, 0x02, 0x02, 0xFF, 0x00, 0x05, 0xE9, 0xCA]
        );
    }
}
