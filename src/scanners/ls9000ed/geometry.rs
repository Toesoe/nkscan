//! Scan geometry and sampling settings
//!
//! Everything is in 1/4000-in dots, the sensor's native pitch, since `new()` pins the
//! measurement units to a 4000 divisor at open.

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

/// DPI mode to read out
/// The scanner nativley operates at 4000 DPI and does firmware-level division to downsample
#[derive(Debug, Copy, Clone)]
pub enum Dpi {
    _4000,
    _2000,
    _1333,
    _333,
}

impl Dpi {
    /// Firmware divisor; scan dpi = 4000/k
    pub fn divisor(self) -> u32 {
        match self {
            Dpi::_4000 => 1,
            Dpi::_2000 => 2,
            Dpi::_1333 => 3,
            Dpi::_333 => 12,
        }
    }

    /// Resolution in DPI, as a window descriptor carries it
    pub fn to_dpi(self) -> u16 {
        (4000 / self.divisor()) as u16
    }
}

#[derive(Debug, Copy, Clone)]
/// A scan window in 1/4000-in dots, matching the sensor's native pitch.
/// Resolution-independent: changing DPI reframes the same physical area.
pub struct Window {
    /// Offset along the sensor bar (0..10_000, 63.5 mm).
    pub x_pos: u32,
    /// Offset along stage travel (0..~34_644, 220 mm, a full 120 strip). This is what selects which frame.
    pub y_pos: u32,
    /// Sensor extent or image HEIGHT. This is like the first dimension in MF film, the 6 in 6x9.
    pub x_size: u32,
    /// Stage extent or image WIDTH. Distinguishes 6x4.5 / 6x6 / 6x9.
    /// This must be a multiple of 36 (one CCD interleave block) at any resolution.
    pub y_size: u32,
}

impl Window {
    /// How long the sensor is, in dots
    pub const SENSOR_DOTS: u32 = 10_000;
    /// The width every captured scan uses: 56.9 mm, the 56 mm of 120 film
    pub const FILM_WIDTH_DOTS: u32 = 8964;
    /// Full stage travel, 220 mm, the whole 120 strip
    pub const STRIP_DOTS: u32 = 34_644;

    /// The whole strip, as the 83-DPI thumbnail pass scans it
    pub fn overview() -> Self {
        Self::centred(0, Self::FILM_WIDTH_DOTS, Self::STRIP_DOTS)
    }

    /// 83 DPI is the 4000-dot grid divided by 48. Neither strip dimension divides evenly, and
    /// the scanner truncates rather than rounds - a full overview is exactly 804636 bytes.
    pub const OVERVIEW_DIVISOR: u32 = 48;

    /// Output pixels of the overview pass
    pub fn overview_dims() -> (u32, u32) {
        (
            Self::FILM_WIDTH_DOTS / Self::OVERVIEW_DIVISOR,
            Self::STRIP_DOTS / Self::OVERVIEW_DIVISOR,
        )
    }

    pub fn centred(y_pos: u32, x_size: u32, y_size: u32) -> Self {
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
    /// Multisample
    pub multisample: Multisample,
    /// The window in the scanner FoV to actually scan
    pub window: Window,
}

impl ScanSettings {
    /// `None` if the window doesn't divide evenly at this resolution.
    pub fn output_dims(&self) -> Option<(u32, u32)> {
        let k = self.dpi.divisor();
        (self.window.x_size.is_multiple_of(k) && self.window.y_size.is_multiple_of(k))
            .then(|| (self.window.y_size / k, self.window.x_size / k))
    }

    /// CCD lines read per stage position.
    pub fn lines(&self) -> u32 {
        match self.ccd_mode {
            CcdMode::ThreeLine => 3,
            CcdMode::SingleLine => 1,
        }
    }

    /// Readouts emitted per stage position: one RGB triple per multi-sample
    /// repeat, plus a single infrared readout when enabled.
    ///
    /// Infrared is captured once no matter the multi-sample setting.
    pub fn readouts(&self) -> u32 {
        3 * self.multisample.count() + u32::from(self.ir)
    }

    /// Spacing between the CCD's lines, in output columns
    pub fn ccd_block(&self) -> u32 {
        match self.ccd_mode {
            CcdMode::ThreeLine => 12 / self.dpi.divisor(),
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
}
