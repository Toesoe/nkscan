//! Scan geometry and sampling settings
//!
//! Everything is in 1/4000-in dots, the sensor's native pitch, since
//! [`Ls5000::new`](super::Ls5000::new) pins the measurement units to a 4000 divisor at open.

use crate::scanners::{ScanArea, nikon::limits::DeviceLimits};

/// The measurement units this driver sets at open
///
/// Only for converting millimeters; anything that scales with the device takes it from
/// [`DeviceLimits`] instead.
pub const DOTS_PER_INCH: u32 = 4000;

/// A whole frame at `y_pos`, spanning the adapter's full scan area
///
/// The full boundary rather than one dot short of it: the firmware takes the boundary itself.
pub fn whole_frame(y_pos: u32, capabilities: DeviceLimits) -> ScanArea {
    ScanArea {
        x_pos: 0,
        y_pos,
        x_size: capabilities.boundary_x,
        y_size: capabilities.boundary_y,
    }
}

/// DPI mode to read out
///
/// The sensor reads at its optical resolution and the firmware divides down. The device reports
/// a continuous range of 90 to 4000, so this is the scanning subset rather than everything
/// the firmware will take.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Dpi {
    _4000,
    _2000,
    _1333,
    _1000,
    _800,
    _500,
    _250,
}

impl Dpi {
    /// Firmware divisor; scan dpi = optical/k
    ///
    /// `_1333` is 4000/3 named to the nearest whole DPI.
    pub fn divisor(self) -> u32 {
        match self {
            Dpi::_4000 => 1,
            Dpi::_2000 => 2,
            Dpi::_1333 => 3,
            Dpi::_1000 => 4,
            Dpi::_800 => 5,
            Dpi::_500 => 8,
            Dpi::_250 => 16,
        }
    }

    /// The name of the mode, for a caller that has to print or parse one
    ///
    /// Nominal, off the 4000-DPI sensor. What a window descriptor carries is
    /// [`ScanSettings::res`], which divides the resolution the device reports.
    pub fn to_dpi(self) -> u16 {
        (4000 / self.divisor()) as u16
    }

    /// Every division offered here, lowest divisor first
    pub const ALL: [Dpi; 7] = [
        Dpi::_4000,
        Dpi::_2000,
        Dpi::_1333,
        Dpi::_1000,
        Dpi::_800,
        Dpi::_500,
        Dpi::_250,
    ];
}

/// How many times the sensor reads each line
///
/// The scanner does not combine them: a multi-sampled pass streams every sample and the host
/// averages. That readout is a different shape from the planar one this driver decodes (see
/// `docs/OPEN_QUESTIONS.md`), so [`arm`](super::Ls5000::arm) refuses a count above 1 rather
/// than putting one on the wire. The encoding is here because it is the part that is known.
///
/// The count goes on the wire as a nibble, so 16 is the ceiling.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Samples(u8);

impl Samples {
    pub const ALL: [u8; 5] = [1, 2, 4, 8, 16];

    /// `None` for a count the nibble cannot carry
    pub fn new(count: u8) -> Option<Self> {
        Self::ALL.contains(&count).then_some(Self(count))
    }

    pub fn count(self) -> u8 {
        self.0
    }

    pub fn is_multi(self) -> bool {
        self.0 > 1
    }
}

impl Default for Samples {
    fn default() -> Self {
        Self(1)
    }
}

/// The resolution the metering pass runs at
///
/// Not a division of the optical resolution, which is why [`ScanSettings`] carries a raw
/// resolution rather than a [`Dpi`]: the firmware takes any value in the range the device
/// reports, and this one is off the ladder.
pub const METER_RESOLUTION: u16 = 285;

/// 16-bit linear RGB (plus infrared) over one window
#[derive(Debug, Clone, Copy)]
pub struct ScanSettings {
    /// What the window descriptor carries, in DPI. Anything in the reported range.
    pub resolution: u16,
    /// Capture channel 0x09 as a 4th planar channel
    pub ir: bool,
    /// Passes the sensor averages in hardware
    pub samples: Samples,
    /// What to scan, which is also what picks the frame off the roll
    pub window: ScanArea,
    pub capabilities: DeviceLimits,
}

impl ScanSettings {
    /// The sensor's own resolution, which is what the firmware divides down
    fn optical(&self) -> u32 {
        u32::from(self.capabilities.x_resolution.optical)
    }

    pub fn n_colors(&self) -> usize {
        3 + usize::from(self.ir)
    }

    /// The resolution the window descriptor carries
    pub fn res(&self) -> u16 {
        self.resolution
    }

    /// Whether the device says it will read out at this resolution
    pub fn resolution_is_supported(&self) -> bool {
        let range = self.capabilities.x_resolution;
        (range.min..=range.max).contains(&self.resolution)
    }

    /// Output pixels across and lines down
    ///
    /// The line count rounds up and the pixel count truncates: a 3946 by 5959 window at 285
    /// DPI reads out 281 wide by 425 down. Truncating the height would declare 424 and leave
    /// the last line of every frame unread.
    pub fn output_dims(&self) -> (u32, u32) {
        let (res, optical) = (u32::from(self.resolution), self.optical());
        (
            self.window.x_size * res / optical,
            (self.window.y_size * res).div_ceil(optical),
        )
    }

    /// Window dimensions, which the descriptor takes in native units
    ///
    /// The window as asked for rather than a rounded extent: the firmware is what rounds.
    pub fn native_dims(&self) -> (u32, u32) {
        (self.window.x_size, self.window.y_size)
    }

    /// Where [`autofocus`](super::Ls5000::autofocus) wants aiming
    pub fn center(&self) -> (u32, u32) {
        let (width, length) = self.native_dims();
        (
            self.window.x_pos + width / 2,
            self.window.y_pos + length / 2,
        )
    }

    /// Each plane is padded to an even sample count, then the line to a 512 multiple
    pub fn bytes_per_line(&self) -> usize {
        let (width, _) = self.output_dims();
        let even_width = width as usize + (width as usize & 1);
        (self.n_colors() * even_width * 2).div_ceil(512) * 512
    }

    pub fn expected_bytes(&self) -> u64 {
        let (_, height) = self.output_dims();
        self.bytes_per_line() as u64 * u64::from(height)
    }
}

/// Where frame `index` starts relative to an even pitch, in this scanner's dots
///
/// The last value repeats, so one offset shifts the whole strip and a list of them corrects
/// gap drift along it. An empty list shifts nothing.
///
/// The same shape as the LS-50's `frame_offset`, and kept separate: the two drivers disagree
/// about enough of this generation that merging on a resemblance is how a wrong answer sticks.
/// docs/OPEN_QUESTIONS.md sections 13 to 18.
pub fn frame_offset(offsets: &[f32], index: u32) -> u32 {
    match offsets.len().checked_sub(1) {
        Some(last) => native_dots(offsets[(index as usize).min(last)]),
        None => 0,
    }
}

/// A millimeter figure in this scanner's dots
pub fn native_dots(millimeters: f32) -> u32 {
    crate::scanners::nikon::native_dots(millimeters, DOTS_PER_INCH)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(dpi: Dpi, ir: bool) -> ScanSettings {
        let capabilities = super::super::capabilities::fixture::capabilities();
        ScanSettings {
            resolution: dpi.to_dpi(),
            ir,
            samples: Samples::default(),
            window: whole_frame(0, capabilities),
            capabilities,
        }
    }

    /// A whole frame at the metering resolution: 281 pixels over 425 lines, 2560 bytes a line
    #[test]
    fn the_metering_pass_geometry() {
        let capabilities = super::super::capabilities::fixture::capabilities();
        let metering = ScanSettings {
            resolution: METER_RESOLUTION,
            ir: true,
            ..settings(Dpi::_4000, true)
        };
        assert_eq!(metering.output_dims(), (281, 425));
        assert_eq!(metering.bytes_per_line(), 2560);
        assert_eq!(metering.expected_bytes(), 1_088_000);
        assert!(metering.resolution_is_supported());
        let _ = capabilities;
    }

    /// Every mode names the division it actually is, and the device will do all of them
    #[test]
    fn every_dpi_mode_is_a_division_the_device_offers() {
        let range = settings(Dpi::_4000, false).capabilities.x_resolution;
        for mode in Dpi::ALL {
            assert_eq!(mode.to_dpi(), (4000 / mode.divisor()) as u16);
            assert!(
                (range.min..=range.max).contains(&mode.to_dpi()),
                "{mode:?} is outside the reported range"
            );
        }
    }

    /// The window is the adapter's full boundary, not one dot short of it, and the descriptor
    /// carries it unrounded
    #[test]
    fn a_whole_frame_spans_the_reported_boundary() {
        let capabilities = super::super::capabilities::fixture::capabilities();
        let s = settings(Dpi::_4000, false);
        assert_eq!(s.native_dims(), (3946, 5959));
        assert_eq!(
            s.native_dims(),
            (capabilities.boundary_x, capabilities.boundary_y)
        );
        assert_eq!(s.output_dims(), (3946, 5959));
    }

    #[test]
    fn geometry_divides_by_the_resolution() {
        let s = settings(Dpi::_1000, false);
        assert_eq!(s.res(), 1000);
        // 3946/4 truncates to 986; 5959/4 rounds up to 1490
        assert_eq!(s.output_dims(), (986, 1490));
        // 3 planes * 986 * 2 bytes = 5916, padded to 6144
        assert_eq!(s.bytes_per_line(), 6144);
        assert_eq!(s.expected_bytes(), 6144 * 1490);
    }

    #[test]
    fn infrared_adds_a_fourth_plane() {
        let s = settings(Dpi::_1000, true);
        assert_eq!(s.n_colors(), 4);
        // 4 * 986 * 2 = 7888, padded to 8192
        assert_eq!(s.bytes_per_line(), 8192);
        assert_eq!(s.expected_bytes(), 8192 * 1490);
    }

    /// A pass that left its last line unread is what the rounding guards against, so the line
    /// count must cover the window at every resolution the device offers
    #[test]
    fn the_line_count_never_truncates() {
        let capabilities = super::super::capabilities::fixture::capabilities();
        let optical = u32::from(capabilities.x_resolution.optical);
        for resolution in [90u16, 285, 500, 1000, 1333, 2000, 4000] {
            let s = ScanSettings {
                resolution,
                ..settings(Dpi::_4000, false)
            };
            let (_, height) = s.output_dims();
            assert!(
                height * optical >= s.window.y_size * u32::from(resolution),
                "{resolution} DPI declared {height} lines, short of the window"
            );
        }
    }

    #[test]
    fn geometry_follows_the_reported_capabilities() {
        // A device claiming half the scan area, so nothing here can be a constant
        let capabilities = DeviceLimits {
            boundary_x: 2000,
            boundary_y: 4000,
            ..super::super::capabilities::fixture::capabilities()
        };
        let s = ScanSettings {
            capabilities,
            window: whole_frame(0, capabilities),
            ..settings(Dpi::_4000, false)
        };
        assert_eq!(s.output_dims(), (2000, 4000));
        assert_eq!(s.native_dims(), (2000, 4000));
    }

    /// The window selects the frame, so its size stays put as it moves
    #[test]
    fn a_window_further_along_the_roll_scans_the_same_size() {
        let capabilities = super::super::capabilities::fixture::capabilities();
        let s = ScanSettings {
            window: whole_frame(31_206, capabilities),
            ..settings(Dpi::_1000, false)
        };
        assert_eq!(s.output_dims(), settings(Dpi::_1000, false).output_dims());
        assert_eq!(s.window.y_pos, 31_206);
    }

    /// Tracks the window, so a frame further along the roll focuses on itself
    #[test]
    fn the_center_moves_with_the_window() {
        let capabilities = super::super::capabilities::fixture::capabilities();
        assert_eq!(settings(Dpi::_4000, false).center(), (1973, 2979));

        let further = ScanSettings {
            window: whole_frame(31_206, capabilities),
            ..settings(Dpi::_4000, false)
        };
        assert_eq!(further.center(), (1973, 31_206 + 2979));
    }

    /// Only the counts the firmware's nibble can carry
    #[test]
    fn the_sample_count_is_restricted_to_what_the_nibble_holds() {
        for count in Samples::ALL {
            assert_eq!(Samples::new(count).unwrap().count(), count);
        }
        for bad in [0u8, 3, 5, 32, 255] {
            assert!(Samples::new(bad).is_none(), "{bad} should be refused");
        }
        assert!(!Samples::new(1).unwrap().is_multi());
        assert!(Samples::new(2).unwrap().is_multi());
    }

    /// One inch is the unit divisor by definition, which is what fixes the rest
    #[test]
    fn millimeters_convert_to_native_dots() {
        assert_eq!(native_dots(25.4), 4000);
        assert_eq!(native_dots(0.0), 0);
        assert_eq!(native_dots(1.0), 157);
        assert_eq!(native_dots(-1.0), 0);
    }
}
