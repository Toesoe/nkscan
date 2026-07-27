//! Scan geometry and sampling settings
//!
//! Everything is in 1/4000-in dots, the sensor's native pitch, since `new()` pins the
//! measurement units to a 4000 divisor at open.

use super::window::BaseQuality;

/// CCD read mode
#[derive(Debug, Copy, Clone)]
pub enum CcdMode {
    /// Nikon calls this "Super Fine" mode, which may reduce banding at the cost of scan speed
    SingleLine,
    /// Normal three-line simultaneous readout
    ThreeLine,
}

/// Multisampling mode
/// Increasing this value increases scan time for the benefit of decreased noise
/// NOTE: When IR is enabled, the IR channel isn't multisampled
#[derive(Debug, Copy, Clone)]
pub enum Multisample {
    X1,
    X2,
    X4,
    X8,
    X16,
}

impl Multisample {
    /// Number of repeats per stage position
    pub fn count(self) -> u32 {
        match self {
            Multisample::X1 => 1,
            Multisample::X2 => 2,
            Multisample::X4 => 4,
            Multisample::X8 => 8,
            Multisample::X16 => 16,
        }
    }
}

/// The measurement units this driver sets at open
///
/// Only for converting millimeters; anything that scales with the device takes it from
/// [`Capabilities`](super::Capabilities) instead.
pub const DOTS_PER_INCH: u32 = 4000;

/// A millimeter figure in this scanner's dots
pub fn native_dots(millimeters: f32) -> u32 {
    crate::scanners::nikon::native_dots(millimeters, DOTS_PER_INCH)
}

/// DPI mode to read out
/// The scanner natively operates at 4000 DPI and does firmware-level division to downsample
#[derive(Debug, Copy, Clone)]
pub enum Dpi {
    _4000,
    _2000,
    _1333,
    _666,
    _333,
}

impl Dpi {
    /// Firmware divisor; scan dpi = 4000/k
    pub fn divisor(self) -> u32 {
        match self {
            Dpi::_4000 => 1,
            Dpi::_2000 => 2,
            Dpi::_1333 => 3,
            Dpi::_666 => 6,
            Dpi::_333 => 12,
        }
    }

    /// Resolution in DPI, as a window descriptor carries it
    pub fn to_dpi(self) -> u16 {
        (4000 / self.divisor()) as u16
    }
}

pub use crate::scanners::ScanArea;

impl ScanArea {
    /// How long the sensor is, in dots
    pub const SENSOR_DOTS: u32 = 10_000;
    /// The width every captured scan uses: 56.9 mm, the 56 mm of 120 film
    pub const FILM_WIDTH_DOTS: u32 = 8964;
    /// Full stage travel, 220 mm, the whole 120 strip
    pub const STRIP_DOTS: u32 = 34_644;

    /// The whole strip, as the 83-DPI thumbnail pass scans it
    pub fn overview() -> Self {
        Self::centered(0, Self::FILM_WIDTH_DOTS, Self::STRIP_DOTS)
    }

    /// 83 DPI is the 4000-dot grid divided by 48. Neither strip dimension divides evenly, and
    /// the scanner truncates rather than rounds, so a full overview is exactly 804636 bytes
    pub const OVERVIEW_DIVISOR: u32 = 48;

    /// Output pixels of the overview pass
    pub fn overview_dims() -> (u32, u32) {
        (
            Self::FILM_WIDTH_DOTS / Self::OVERVIEW_DIVISOR,
            Self::STRIP_DOTS / Self::OVERVIEW_DIVISOR,
        )
    }

    pub fn centered(y_pos: u32, x_size: u32, y_size: u32) -> Self {
        Self {
            x_pos: (Self::SENSOR_DOTS - x_size) / 2,
            y_pos,
            x_size,
            y_size,
        }
    }
}

/// Scan settings for one frame
#[derive(Debug, Clone, Copy)]
pub struct ScanSettings {
    /// CCD read mode
    pub ccd_mode: CcdMode,
    /// Include an IR pass for dust removal
    pub ir: bool,
    /// Scan resolution in DPI
    pub dpi: Dpi,
    /// Square sampling, or the half-rate stage stepping of a preview
    pub quality: BaseQuality,
    /// Multisample
    pub multisample: Multisample,
    /// The window in the scanner FoV to actually scan
    pub window: ScanArea,
}

impl ScanSettings {
    /// Dots per output column along stage travel
    ///
    /// A preview halves the stage rate without touching the sensor, so it is the one mode
    /// where the two axes don't share a divisor
    pub fn stage_divisor(&self) -> u32 {
        self.dpi.divisor()
            * match self.quality {
                BaseQuality::Scan => 1,
                BaseQuality::Preview => 2,
            }
    }

    /// `None` if the window doesn't divide evenly at this resolution.
    pub fn output_dims(&self) -> Option<(u32, u32)> {
        let (k, stage) = (self.dpi.divisor(), self.stage_divisor());
        (self.window.x_size.is_multiple_of(k) && self.window.y_size.is_multiple_of(stage))
            .then(|| (self.window.y_size / stage, self.window.x_size / k))
    }

    /// CCD lines read per stage position
    pub fn lines(&self) -> u32 {
        match self.ccd_mode {
            CcdMode::ThreeLine => 3,
            CcdMode::SingleLine => 1,
        }
    }

    /// Readouts emitted per stage position: one RGB triple per multi-sample
    /// repeat, plus a single infrared readout when enabled.
    ///
    /// Infrared is captured once no matter the multi-sample setting
    pub fn readouts(&self) -> u32 {
        3 * self.multisample.count() + u32::from(self.ir)
    }

    /// Spacing between the CCD's lines, in output columns
    ///
    /// The lines sit 12 dots apart along stage travel, so this is against the stage divisor
    /// rather than the sensor one
    pub fn ccd_block(&self) -> u32 {
        match self.ccd_mode {
            CcdMode::ThreeLine => (12 / self.stage_divisor()).max(1),
            CcdMode::SingleLine => 1,
        }
    }

    /// Stage positions the scan will step through
    pub fn stages(&self) -> Option<u32> {
        self.output_dims().map(|(w, _)| w / self.lines())
    }

    /// Total bytes the scanner will return for this frame
    pub fn expected_bytes(&self) -> Option<u64> {
        let (_, height) = self.output_dims()?;
        let per_stage = u64::from(self.readouts()) * u64::from(height) * u64::from(self.lines());
        Some(2 * u64::from(self.stages()?) * per_stage)
    }

    /// The base settings used for autoexposure passes
    ///
    /// `ir` should match the scan being metered for. It costs one extra readout per stage, and
    /// it is the only way the infrared gain gets measured rather than assumed.
    pub fn autoexposure(window: ScanArea, ir: bool) -> Self {
        Self {
            ccd_mode: CcdMode::ThreeLine,
            ir,
            dpi: Dpi::_666,
            quality: BaseQuality::Preview,
            multisample: Multisample::X1,
            window,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The 666x333 prescan of a 6x9 frame, pinned to the byte count summed off the wire
    /// in singleline_ccd. Sensor and stage divide by 6 and 12 respectively.
    fn prescan(multisample: Multisample) -> ScanSettings {
        ScanSettings {
            ccd_mode: CcdMode::ThreeLine,
            ir: true,
            dpi: Dpi::_666,
            quality: BaseQuality::Preview,
            multisample,
            window: ScanArea::centered(18672, ScanArea::FILM_WIDTH_DOTS, 13176),
        }
    }

    #[test]
    fn preview_halves_only_the_stage_axis() {
        let settings = prescan(Multisample::X1);
        assert_eq!(settings.stage_divisor(), 12);
        assert_eq!(settings.output_dims(), Some((1098, 1494)));
        assert_eq!(settings.stages(), Some(366));
    }

    /// A preview steps the stage at the 12-dot CCD spacing, so the lines land in
    /// adjacent output columns rather than 12 apart
    #[test]
    fn preview_collapses_the_interleave_block() {
        assert_eq!(prescan(Multisample::X1).ccd_block(), 1);
        assert_eq!(
            ScanSettings {
                dpi: Dpi::_4000,
                quality: BaseQuality::Scan,
                ..prescan(Multisample::X1)
            }
            .ccd_block(),
            12
        );
    }

    #[test]
    fn prescan_byte_counts_match_the_captures() {
        assert_eq!(prescan(Multisample::X1).expected_bytes(), Some(13_123_296));
        assert_eq!(
            prescan(Multisample::X16).expected_bytes(),
            Some(160_760_376)
        );

        // The 6x4.5 prescan from 8x_multisampling
        let short = ScanSettings {
            window: ScanArea::centered(26160, ScanArea::FILM_WIDTH_DOTS, 6696),
            ..prescan(Multisample::X1)
        };
        assert_eq!(short.expected_bytes(), Some(6_669_216));
    }

    /// The full-resolution single-line pass from the same session
    #[test]
    fn full_resolution_byte_count_matches_the_capture() {
        let settings = ScanSettings {
            ccd_mode: CcdMode::SingleLine,
            dpi: Dpi::_4000,
            quality: BaseQuality::Scan,
            ..prescan(Multisample::X1)
        };
        assert_eq!(settings.expected_bytes(), Some(944_877_312));
    }
}
