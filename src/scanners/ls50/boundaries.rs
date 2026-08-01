//! The frame table: where the frames sit on the loaded film
//!
//! A pass declares every frame, not just the one it scans, or the later frames come back black.
//! Positions are one pitch apart plus the caller's per-frame correction, and a pass takes its
//! window from [`FrameRect::scan_area`] so the two cannot disagree about where a frame is.

use super::{Ls50, dtc::Dtc, geometry::frame_offset};
use crate::scanners::{ScanArea, nikon::limits::DeviceLimits};
use crate::scsi::{self, Transport};

/// One frame's extent in native pixels. Y runs along the feed, X along the sensor bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRect {
    pub y_top: u32,
    pub x_left: u32,
    pub y_bottom: u32,
    pub x_right: u32,
}

impl FrameRect {
    /// A frame one pitch tall at `y_top`, spanning the adapter's full width
    pub fn full_width(y_top: u32, height: u32, native_x: u32) -> Self {
        Self {
            y_top,
            x_left: 0,
            y_bottom: y_top + height - 1,
            x_right: native_x - 1,
        }
    }

    /// The window that scans just this frame
    ///
    /// The firmware serves film from where the table says the frame starts, so a window offset
    /// the table does not know about reads that far *into* the frame instead of moving to it.
    /// The length is [`max_y`](DeviceLimits::max_y), one short of the boundary the firmware
    /// enforces, so every frame comes out the same size.
    pub fn scan_area(self, capabilities: DeviceLimits) -> ScanArea {
        ScanArea {
            x_pos: self.x_left,
            y_pos: self.y_top,
            x_size: capabilities.max_x(),
            y_size: capabilities.max_y(),
        }
    }

    fn to_bytes(self) -> [u8; 16] {
        let mut buf = [0u8; 16];
        buf[0..4].copy_from_slice(&self.y_top.to_be_bytes());
        buf[4..8].copy_from_slice(&self.x_left.to_be_bytes());
        buf[8..12].copy_from_slice(&self.y_bottom.to_be_bytes());
        buf[12..16].copy_from_slice(&self.x_right.to_be_bytes());
        buf
    }
}

/// The DTC 0x88 parameter list: a short header plus one rectangle per frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameBoundaries(pub Vec<FrameRect>);

impl FrameBoundaries {
    /// `count` frames `pitch` apart, each shifted by its own offset in mm
    ///
    /// Last offset repeats, see [`frame_offset`]. They go in the table, not the window: the
    /// table is what the firmware positions film against.
    pub fn evenly_spaced(count: u32, pitch: u32, offsets: &[f32], native_x: u32) -> Self {
        Self(
            (0..count)
                .map(|i| {
                    FrameRect::full_width(i * pitch + frame_offset(offsets, i), pitch, native_x)
                })
                .collect(),
        )
    }

    /// The parameter list, or `None` if it will not fit the wire format
    ///
    /// The header carries the count in one byte and the length in two, so past 255 frames the
    /// header and the rectangles that follow it would disagree about how many there are, and
    /// the scanner would read the table off the end.
    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        let count = u8::try_from(self.0.len()).ok()?;
        let mut buf = Vec::with_capacity(4 + 16 * self.0.len());
        // Total length includes the 4-byte header itself
        buf.extend_from_slice(&(4 + 16 * u16::from(count)).to_be_bytes());
        buf.push(count);
        // Reserved in the spec, but only ever driven with the count repeated
        buf.push(count);
        for rect in &self.0 {
            buf.extend_from_slice(&rect.to_bytes());
        }
        Some(buf)
    }
}

impl<T> Ls50<T>
where
    T: Transport,
{
    /// Tell the scanner where the frames sit on the loaded film
    ///
    /// Re-declared before every pass: a pass that cannot see the whole table leaves the feed
    /// where it is, and the frames after the first come back black.
    pub fn set_frame_boundaries(
        &mut self,
        boundaries: &FrameBoundaries,
    ) -> Result<(), scsi::Error> {
        let parameters = boundaries.to_bytes().ok_or(scsi::Error::Unsupported(
            "more frames than the boundary table's one-byte count can carry",
        ))?;
        self.write_dtc(Dtc::FrameBoundaries, None, parameters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NATIVE_X: u32 = 3945;

    /// A pitch of its own, so the expected bytes do not move when the calibration does
    const PITCH: u32 = 5984;

    fn descriptor(index: u32, offset: u32) -> Vec<u8> {
        let y_top = index * PITCH + offset;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&y_top.to_be_bytes());
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes.extend_from_slice(&(y_top + PITCH - 1).to_be_bytes());
        bytes.extend_from_slice(&(NATIVE_X - 1).to_be_bytes());
        bytes
    }

    #[test]
    fn one_frame_is_a_header_and_a_rectangle() {
        // len = 4 + 16 = 20 (0x14), count 1
        let mut expected = vec![0x00, 0x14, 0x01, 0x01];
        expected.extend(descriptor(0, 0));
        assert_eq!(
            FrameBoundaries::evenly_spaced(1, PITCH, &[0.0], NATIVE_X).to_bytes(),
            Some(expected)
        );
    }

    /// The offsets are per frame and the last repeats, so frame 0 sits on the pitch and every
    /// frame after it carries the correction
    #[test]
    fn every_frame_of_the_strip_is_declared_at_its_own_offset() {
        let shift = frame_offset(&[0.0, 5.6], 1);
        // len = 4 + 96 = 100 (0x64), six rectangles one pitch apart
        let mut expected = vec![0x00, 0x64, 0x06, 0x06];
        expected.extend(descriptor(0, 0));
        for i in 1..6 {
            expected.extend(descriptor(i, shift));
        }
        assert_eq!(
            FrameBoundaries::evenly_spaced(6, PITCH, &[0.0, 5.6], NATIVE_X).to_bytes(),
            Some(expected)
        );
    }

    /// A window asking for film the table did not declare puts that much of the next frame's
    /// gap at the bottom of the image
    #[test]
    fn the_window_starts_where_the_table_says_the_frame_does() {
        let capabilities = super::super::capabilities::fixture::capabilities();
        let boundaries =
            FrameBoundaries::evenly_spaced(3, capabilities.frame_pitch, &[0.0, 5.6], NATIVE_X);
        let shift = frame_offset(&[0.0, 5.6], 1);

        for (index, rect) in boundaries.0.iter().enumerate() {
            let area = rect.scan_area(capabilities);
            assert_eq!(area.y_pos, rect.y_top, "frame {index}");
            // Same size everywhere, and inside the boundary the firmware enforces
            assert_eq!(area.y_size, capabilities.max_y());
            assert!(area.y_size < capabilities.boundary_y);
        }
        assert_eq!(boundaries.0[0].y_top, 0);
        assert_eq!(boundaries.0[1].y_top, capabilities.frame_pitch + shift);
    }

    /// Past 255 the one-byte count would disagree with the rectangles behind it, and the
    /// scanner would read the table off the end rather than reject it
    #[test]
    fn a_table_too_big_for_its_header_is_refused() {
        let rect = FrameRect::full_width(0, PITCH, NATIVE_X);
        assert!(FrameBoundaries(vec![rect; 255]).to_bytes().is_some());
        assert!(FrameBoundaries(vec![rect; 256]).to_bytes().is_none());
    }

    #[test]
    fn rectangles_span_the_adapter_width() {
        let rect = FrameRect::full_width(100, PITCH, NATIVE_X);
        assert_eq!(rect.x_left, 0);
        assert_eq!(rect.x_right, NATIVE_X - 1);
        assert_eq!(rect.y_bottom, 100 + PITCH - 1);
    }
}
