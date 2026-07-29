//! Vendor-specific window descriptor fields, LS-9000ED

use super::{
    BITS_PER_PIXEL,
    geometry::{CcdMode, Dpi, Multisample},
};
use crate::scanners::ScanArea;
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

    /// Whether to ask the scanner to average the sensor bar down to `x_resolution`
    ///
    /// The single-line CCD divides the sensor axis either way, but the three-line CCD reads
    /// the bar out at its native pitch unless this is set: 8964 samples per sweep at 2000 DPI
    /// rather than 4482.
    fn averaging(self, x_resolution: u16) -> bool {
        matches!(self.quality, BaseQuality::Preview)
            || (matches!(self.ccd, CcdMode::ThreeLine) && x_resolution < Dpi::_4000.to_dpi())
    }

    /// Build the SET WINDOW descriptor for `window` at `x_resolution` DPI
    ///
    /// This scanner only ever does multi-level RGB at 16 bits with no halftoning, padding or
    /// compression, so those are fixed. The id is left for [`set_window`](super::Ls9000::set_window).
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
            vendor: self.vendor(x_resolution),
        }
    }

    /// The ten vendor-specific bytes that tail a window descriptor
    fn vendor(self, x_resolution: u16) -> Vec<u8> {
        let value = self;
        let mut buf = [0u8; 10];

        // High nibble is the multi-sample repeat count minus one
        buf[0] = ((value.multisample.count() - 1) << 4) as u8;

        // Bit 7 pairs with buf[3]: (0x81, 0x02) reads the sensor bar out at its native pitch,
        // (0x01, 0x04) averages it down to the window's resolution. See
        // [`averaging`](Self::averaging).
        //
        // Bit 0 is positive film on other Coolscans. Clearing it here is accepted and reads
        // back cleared, but the image is identical, so leave it set.
        let averaging = value.averaging(x_resolution);
        buf[1] = if averaging { 0x01 } else { 0x81 };
        buf[2] = match value.window_kind {
            WindowKind::Frame => 0x01,
            // Only the 83-DPI whole-strip overview
            WindowKind::Overview => 0x02,
        };
        // Bit 4 is "multi-sampling on", always set exactly when buf[0]'s high nibble is nonzero
        buf[3] = if averaging { 0x04 } else { 0x02 }
            | if value.multisample.count() > 1 {
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
        let bytes = WindowParams {
            exposure: 0x0007_ABDD,
            ..params(BaseQuality::Scan)
        }
        .vendor(4000);
        assert_eq!(
            bytes,
            [0x00, 0x81, 0x01, 0x02, 0x40, 0xFF, 0x00, 0x07, 0xAB, 0xDD]
        );
    }

    /// same but 16x multi-sample
    #[test]
    fn matches_captured_16x_scan() {
        let bytes = WindowParams {
            multisample: Multisample::X16,
            exposure: 0x0007_55AB,
            ..params(BaseQuality::Scan)
        }
        .vendor(4000);
        assert_eq!(
            bytes,
            [0xF0, 0x81, 0x01, 0x12, 0x40, 0xFF, 0x00, 0x07, 0x55, 0xAB]
        );
    }

    /// The 666x333 prescan, 16x multi-sample
    #[test]
    fn matches_captured_prescan() {
        let bytes = WindowParams {
            multisample: Multisample::X16,
            exposure: 0x0007_55AB,
            ..params(BaseQuality::Preview)
        }
        .vendor(666);
        assert_eq!(
            bytes,
            [0xF0, 0x01, 0x01, 0x14, 0x40, 0xFF, 0x00, 0x07, 0x55, 0xAB]
        );
    }

    /// "super fine" single-line CCD at 4000 DPI
    #[test]
    fn matches_captured_singleline_scan() {
        let bytes = WindowParams {
            ccd: CcdMode::SingleLine,
            exposure: 0x000A_8212,
            ..params(BaseQuality::Scan)
        }
        .vendor(4000);
        assert_eq!(
            bytes,
            [0x00, 0x81, 0x01, 0x02, 0x02, 0xFF, 0x00, 0x0A, 0x82, 0x12]
        );
    }

    /// The 83-DPI whole-strip overview square-sampled, so it's a Scan, and single-line CCD
    #[test]
    fn matches_captured_overview() {
        let bytes = WindowParams {
            ccd: CcdMode::SingleLine,
            window_kind: WindowKind::Overview,
            exposure: 0x0005_E9CA,
            ..params(BaseQuality::Scan)
        }
        .vendor(83);
        assert_eq!(
            bytes,
            [0x00, 0x81, 0x02, 0x02, 0x02, 0xFF, 0x00, 0x05, 0xE9, 0xCA]
        );
    }

    /// A three-line scan below 4000 DPI has to ask for averaging, or the sensor bar comes back
    /// at its native pitch: 8964 samples per sweep at 2000 DPI, not 4482.
    ///
    /// Inferred rather than captured. Nikon Scan's traces hold no reduced-resolution three-line
    /// scan, only 4000-DPI scans and 666-DPI prescans.
    #[test]
    fn a_reduced_resolution_three_line_scan_asks_for_averaging() {
        let bytes = params(BaseQuality::Scan).vendor(2000);
        assert_eq!(bytes[1..4], [0x01, 0x01, 0x04]);
    }

    /// The single-line CCD divides the sensor axis whichever way the averaging bits are set,
    /// which the 83-DPI overview capture shows, so leave those windows alone
    #[test]
    fn a_reduced_resolution_single_line_scan_does_not() {
        let bytes = WindowParams {
            ccd: CcdMode::SingleLine,
            ..params(BaseQuality::Scan)
        }
        .vendor(2000);
        assert_eq!(bytes[1..4], [0x81, 0x01, 0x02]);
    }
}
