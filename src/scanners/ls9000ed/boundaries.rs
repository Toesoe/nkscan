//! Frame boundary rectangles, DTC 0x88
//!
//! This is the strip's frame table, where the frames sit on the loaded film.
//!
//! Nikon Scan writes it twice: nominal, evenly-spaced rectangles during calibration,
//! then the real per-frame positions once the overview scan has actually located the frames.

use super::{
    Ls9000ed,
    dtc::{self, Dtc},
};
use crate::scsi::{Error as ScsiError, Transport};

/// One frame's extent, in the same 1/4000-in dots as [`Window`](super::Window).
///
/// Y is along stage travel (which frame), X is along the sensor bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRect {
    pub y_top: u32,
    pub x_left: u32,
    pub y_bottom: u32,
    pub x_right: u32,
}

impl FrameRect {
    /// A frame `height` dots tall at `y_top`, spanning the full 56 mm medium format width.
    ///
    /// Only correct for holders that lay film out in a single row, which is all we have captures for.
    /// A holder carrying two rows of 35 mm side by side would put each row in its own X band.
    pub fn full_width(y_top: u32, height: u32) -> Self {
        Self {
            y_top,
            x_left: Self::X_LEFT,
            y_bottom: y_top + height,
            x_right: Self::X_RIGHT,
        }
    }

    /// The 8964-dot width used throughout the scan path, centred on the 10000-dot sensor exactly as [`Window::centred`](super::Window::centred) does.
    const X_LEFT: u32 = 518;
    const X_RIGHT: u32 = 9482;

    fn to_bytes(self) -> [u8; 16] {
        let mut buf = [0u8; 16];
        buf[0..4].copy_from_slice(&self.y_top.to_be_bytes());
        buf[4..8].copy_from_slice(&self.x_left.to_be_bytes());
        buf[8..12].copy_from_slice(&self.y_bottom.to_be_bytes());
        buf[12..16].copy_from_slice(&self.x_right.to_be_bytes());
        buf
    }
}

/// The DTC 0x88 parameter list: a short header plus one rectangle per frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameBoundaries(pub Vec<FrameRect>);

impl FrameBoundaries {
    /// Data-type code this is read and written under
    pub const DTC: u8 = 0x88;
    /// Data-type qualifier the driver uses for every 0x88 access
    pub const DTQ: u16 = 0x0003;

    /// The nominal boundaries Nikon Scan writes during calibration, before any
    /// frame has actually been located: four 6696-dot (6x4.5) frames butted
    /// together from y=2236, for some reason
    pub fn nominal() -> Self {
        Self(
            (0..4)
                .map(|i| FrameRect::full_width(2236 + i * 6696, 6696))
                .collect(),
        )
    }

    /// `count` frames of `height` dots each, butted together from `y_top`.
    /// Single-row holders only - see [`FrameRect::full_width`].
    pub fn evenly_spaced(y_top: u32, height: u32, count: u32) -> Self {
        Self(
            (0..count)
                .map(|i| FrameRect::full_width(y_top + i * height, height))
                .collect(),
        )
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4 + 16 * self.0.len());
        // Total length includes the 4-byte header itself
        buf.extend_from_slice(&(4 + 16 * self.0.len() as u16).to_be_bytes());
        buf.push(self.0.len() as u8);
        buf.push(0x00); // reserved
        for rect in &self.0 {
            buf.extend_from_slice(&rect.to_bytes());
        }
        buf
    }
}

impl<T> Ls9000ed<T>
where
    T: Transport,
{
    /// Where the frames sit on the loaded film, as the scanner currently has it
    pub fn frame_boundaries(&mut self) -> Result<Vec<u8>, ScsiError> {
        self.read_framed_dtc(Dtc::FrameBoundaries, None, dtc::HEADER_LEN)
    }

    /// Tell the scanner where the frames sit on the loaded film
    pub fn set_frame_boundaries(&mut self, boundaries: &FrameBoundaries) -> Result<(), ScsiError> {
        self.write_dtc(Dtc::FrameBoundaries, None, boundaries.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wire(hex: &[&str]) -> Vec<u8> {
        hex.join(" ")
            .split_whitespace()
            .map(|b| u8::from_str_radix(b, 16).unwrap())
            .collect()
    }

    /// The nominal write from full_session_cold_start
    #[test]
    fn nominal_matches_real_capture() {
        let expected = wire(&[
            "00 44 04 00",
            "00 00 08 BC 00 00 02 06 00 00 22 E4 00 00 25 0A",
            "00 00 22 E4 00 00 02 06 00 00 3D 0C 00 00 25 0A",
            "00 00 3D 0C 00 00 02 06 00 00 57 34 00 00 25 0A",
            "00 00 57 34 00 00 02 06 00 00 71 5C 00 00 25 0A",
        ]);

        assert_eq!(FrameBoundaries::nominal().to_bytes(), expected);
    }

    /// The 6x9 write from 16x_multisample
    #[test]
    fn six_by_nine_matches_real_capture() {
        let expected = wire(&[
            "00 24 02 00",
            "00 00 08 BC 00 00 02 06 00 00 3C 34 00 00 25 0A",
            "00 00 3C 34 00 00 02 06 00 00 6F AC 00 00 25 0A",
        ]);

        assert_eq!(
            FrameBoundaries::evenly_spaced(2236, 13176, 2).to_bytes(),
            expected
        );
    }

    #[test]
    fn header_length_covers_itself_and_every_rectangle() {
        for count in 1..=8u32 {
            let bytes = FrameBoundaries::evenly_spaced(0, 100, count).to_bytes();
            assert_eq!(bytes.len(), 4 + 16 * count as usize);
            assert_eq!(
                u16::from_be_bytes([bytes[0], bytes[1]]) as usize,
                bytes.len()
            );
            assert_eq!(bytes[2] as u32, count);
        }
    }

    #[test]
    fn frames_are_butted_together() {
        let FrameBoundaries(rects) = FrameBoundaries::evenly_spaced(2236, 6696, 4);
        for pair in rects.windows(2) {
            assert_eq!(pair[0].y_bottom, pair[1].y_top);
        }
        assert!(rects.iter().all(|r| r.x_right - r.x_left == 8964));
    }
}
