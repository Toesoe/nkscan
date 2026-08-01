//! Scan geometry and sampling settings
//!
//! Everything is in 1/4000-in dots, the sensor's native pitch, since
//! [`Ls50::new`](super::Ls50::new) pins the measurement units to a 4000 divisor at open.

use crate::scanners::{ScanArea, nikon::limits::DeviceLimits};

/// The measurement units this driver sets at open
///
/// Only for converting millimeters; anything that scales with the device takes it from
/// [`DeviceLimits`] instead.
pub const DOTS_PER_INCH: u32 = 4000;

/// Windows on the film, in 1/4000-in dots. Y runs along the feed and X along the sensor bar,
/// and there is no host feed command, so `y_pos` is what selects a frame.
impl ScanArea {
    /// One whole frame at `y_pos`, spanning the adapter's full scan area
    ///
    /// [`max_y`](DeviceLimits::max_y) rather than the boundary itself, since the firmware
    /// refuses a descriptor whose length reaches it.
    pub fn frame(y_pos: u32, capabilities: DeviceLimits) -> Self {
        Self {
            x_pos: 0,
            y_pos,
            x_size: capabilities.max_x(),
            y_size: capabilities.max_y(),
        }
    }
}

/// DPI mode to read out
///
/// The sensor reads at its optical resolution and the firmware divides down. The divisor is
/// what the scan is actually parameterised by, so these are the resolutions that exist rather
/// than a request the driver rounds off behind the caller's back.
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
    /// `_1333` is 4000/3 named to the nearest whole DPI, as
    /// [`ls9000`](crate::scanners::ls9000::Dpi) names the same division.
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
    /// Nominal, off the LS-50's 4000-DPI sensor. What a window descriptor carries is
    /// [`ScanSettings::res`], which divides the resolution the device reports.
    pub fn to_dpi(self) -> u16 {
        (4000 / self.divisor()) as u16
    }

    /// Every division the firmware offers, lowest divisor first
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

/// 16-bit linear RGB (plus infrared) over one window
#[derive(Debug, Clone, Copy)]
pub struct ScanSettings {
    /// Which of the firmware's divisions of the optical resolution to read out
    pub dpi: Dpi,
    /// Capture channel 0x09 as a 4th planar channel
    pub ir: bool,
    /// Only 1 works: a higher count arms a multi-pass scan that never streams
    pub samples: u8,
    /// What to scan, which is also what picks the frame out of a strip
    pub window: ScanArea,
    pub capabilities: DeviceLimits,
}

impl ScanSettings {
    /// Native dots per output pixel
    pub fn pitch(&self) -> u32 {
        self.dpi.divisor()
    }

    /// The sensor's own resolution, which is what the firmware divides down
    fn optical(&self) -> u16 {
        self.capabilities.x_resolution.optical
    }

    pub fn n_colors(&self) -> usize {
        3 + usize::from(self.ir)
    }

    /// The divided resolution, as the window descriptor carries it
    pub fn res(&self) -> u16 {
        (u32::from(self.optical()) / self.pitch()) as u16
    }

    pub fn output_dims(&self) -> (u32, u32) {
        let pitch = self.pitch();
        (self.window.x_size / pitch, self.window.y_size / pitch)
    }

    /// Window dimensions, which the descriptor takes in native units rather than
    /// output pixels
    pub fn native_dims(&self) -> (u32, u32) {
        let pitch = self.pitch();
        let (w, h) = self.output_dims();
        (w * pitch, h * pitch)
    }

    /// Where [`autofocus`](crate::scanners::nikon::usb::UsbCoolscan::autofocus) wants aiming
    ///
    /// Against [`native_dims`](Self::native_dims), not the window's own size: the descriptor
    /// carries the pitch-rounded extent and that is what the scanner covers.
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

/// The offset for frame `index`, in 1/4000-in dots, the last value repeating
///
/// Per frame because the feed does not place them evenly: the first advance under-travels while
/// the rest match the reported pitch, so frame 0 and the rest want different figures. The
/// numbers are the caller's to measure; where film sits belongs to the load, not the scanner.
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
            dpi,
            ir,
            samples: 1,
            window: ScanArea::frame(0, capabilities),
            capabilities,
        }
    }

    /// The last value repeats, so two figures cover a strip of any length
    #[test]
    fn frame_offsets_repeat_their_last_value() {
        let measured = [0.0, 5.6];
        assert_eq!(frame_offset(&measured, 0), 0);
        assert_eq!(frame_offset(&measured, 1), native_dots(5.6));
        assert_eq!(frame_offset(&measured, 5), native_dots(5.6));
        assert_eq!(frame_offset(&[2.0], 3), native_dots(2.0));
        assert_eq!(frame_offset(&[], 0), 0);
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

    /// The divisor is the whole of it: no rounding, so the mode fixes every figure below
    #[test]
    fn geometry_divides_by_the_dpi_mode() {
        let s = settings(Dpi::_1000, false);
        assert_eq!(s.pitch(), 4);
        assert_eq!(s.res(), 1000);
        // The 3945 by 5958 window, divided down and its remainder dropped
        assert_eq!(s.output_dims(), (986, 1489));
        assert_eq!(s.native_dims(), (3944, 5956));
        // 3 planes * 986 * 2 bytes = 5916, padded to 6144
        assert_eq!(s.bytes_per_line(), 6144);
        assert_eq!(s.expected_bytes(), 6144 * 1489);
    }

    #[test]
    fn infrared_adds_a_fourth_plane() {
        let s = settings(Dpi::_1000, true);
        assert_eq!(s.n_colors(), 4);
        // 4 * 986 * 2 = 7888, padded to 8192
        assert_eq!(s.bytes_per_line(), 8192);
        assert_eq!(s.expected_bytes(), 8192 * 1489);
    }

    #[test]
    fn geometry_follows_the_reported_capabilities() {
        // A device claiming half the scan area, so nothing here can be a constant
        let capabilities = DeviceLimits {
            boundary_x: 2001,
            boundary_y: 4001,
            ..super::super::capabilities::fixture::capabilities()
        };
        let s = ScanSettings {
            capabilities,
            window: ScanArea::frame(0, capabilities),
            ..settings(Dpi::_4000, false)
        };
        assert_eq!(s.pitch(), 1);
        assert_eq!(s.output_dims(), (2000, 4000));
        assert_eq!(s.native_dims(), (2000, 4000));
    }

    /// The window selects the frame, so its size stays put as it moves
    #[test]
    fn a_window_further_down_the_strip_scans_the_same_size() {
        let capabilities = super::super::capabilities::fixture::capabilities();
        let s = ScanSettings {
            window: ScanArea::frame(2 * capabilities.frame_pitch, capabilities),
            ..settings(Dpi::_1000, false)
        };
        assert_eq!(s.output_dims(), settings(Dpi::_1000, false).output_dims());
        assert_eq!(s.window.y_pos, 2 * capabilities.frame_pitch);
    }

    /// Tracks the window and the rounded extent, not the adapter
    #[test]
    fn the_center_moves_with_the_window() {
        let capabilities = super::super::capabilities::fixture::capabilities();
        let at_origin = settings(Dpi::_1000, false);
        // native_dims is (3944, 5956), which is what the descriptor carries
        assert_eq!(at_origin.center(), (1972, 2978));

        let pitch = capabilities.frame_pitch;
        let further_down = ScanSettings {
            window: ScanArea::frame(pitch, capabilities),
            ..at_origin
        };
        assert_eq!(further_down.center(), (1972, pitch + 2978));
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
