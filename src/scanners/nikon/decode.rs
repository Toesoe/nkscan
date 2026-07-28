//! The planar line stream the USB Coolscans produce
//!
//! One plane per channel back to back, `R G B` or `R G B I`, top line first. Each plane is
//! `width` samples padded to an even count and the line padded to a 512-byte multiple, so
//! decoding is a per-line de-interleave.
//!
//! Shared because the layout is the same wire format on every model that streams this way;
//! what differs is only the geometry each driver computes for it. The LS-9000's stream is a
//! different shape entirely and keeps its own decoder.

use crate::decode::{BlockSink, Blocked, ImageView, LengthMismatch, be_u16_at};
use image::ImageBuffer;

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error(transparent)]
    LengthMismatch(#[from] LengthMismatch),
}

/// De-interleaves the planar stream one line at a time
pub struct PlanarLines {
    width: usize,
    height: usize,
    /// 3, or 4 with infrared
    n_colors: usize,
    /// One plane in samples: `width` padded to an even count
    plane: usize,
    /// On-wire bytes per line, planes plus the block padding
    stride: usize,
    rgb: Vec<u16>,
    /// Empty when the pass is RGB only
    ir_plane: Vec<u16>,
    /// Lines emitted so far, which is the current output row
    rows: usize,
}

pub type FrameDecoder = Blocked<PlanarLines>;

impl PlanarLines {
    pub fn new(width: usize, height: usize, n_colors: usize, stride: usize) -> Self {
        Self {
            width,
            height,
            n_colors,
            plane: width + (width & 1),
            stride,
            rgb: vec![0; width * height * 3],
            ir_plane: if n_colors == 4 {
                vec![0; width * height]
            } else {
                Vec::new()
            },
            rows: 0,
        }
    }
}

/// A decoder over a planar pass of these dimensions
///
/// Each driver wraps this with its own geometry, since what `stride` and `n_colors` are is a
/// property of the scan settings rather than of the stream format.
pub fn planar_decoder(
    width: usize,
    height: usize,
    n_colors: usize,
    stride: usize,
) -> FrameDecoder {
    Blocked::wrap(PlanarLines::new(width, height, n_colors, stride))
}

impl BlockSink for PlanarLines {
    type Output<'a> = ImageView<'a>;
    type Error = DecodeError;

    fn block_len(&self) -> usize {
        self.stride
    }

    fn blocks(&self) -> u64 {
        self.height as u64
    }

    /// Planes 0/1/2 interleave into RGB, plane 3 becomes the IR mask. Reading only
    /// `width` samples per plane skips both paddings.
    fn emit(&mut self, line: &[u8]) {
        let y = self.rows;
        let rgb_base = y * self.width * 3;
        for channel in 0..3 {
            let plane_base = channel * self.plane;
            for x in 0..self.width {
                self.rgb[rgb_base + x * 3 + channel] = be_u16_at(line, plane_base + x);
            }
        }
        if self.n_colors == 4 {
            let ir_base = y * self.width;
            let plane_base = 3 * self.plane;
            for x in 0..self.width {
                self.ir_plane[ir_base + x] = be_u16_at(line, plane_base + x);
            }
        }
        self.rows += 1;
    }

    fn finish(&mut self) -> Result<Self::Output<'_>, DecodeError> {
        let (width, height) = (self.width as u32, self.height as u32);
        Ok(ImageView {
            rgb: ImageBuffer::from_raw(width, height, &self.rgb[..])
                .expect("buffer is sized from the dims"),
            ir: (self.n_colors == 4).then(|| {
                ImageBuffer::from_raw(width, height, &self.ir_plane[..])
                    .expect("buffer is sized from the dims")
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::Image;
    use crate::decode::StreamDecoder;

    /// Samples as their big-endian wire bytes, the way the scanner sends them
    fn be(samples: &[u16]) -> Vec<u8> {
        samples.iter().flat_map(|s| s.to_be_bytes()).collect()
    }

    fn decode(data: &[u8], width: usize, height: usize, n_colors: usize, stride: usize) -> Image {
        let mut decoder = planar_decoder(width, height, n_colors, stride);
        decoder.push(data).unwrap();
        decoder.finish().unwrap().to_owned()
    }

    #[test]
    fn deinterleaves_planar_lines() {
        // 2x2, three planes back to back per line: R[..] G[..] B[..], six samples a
        // line at 2 bytes each, so stride 12
        let data = be(&[
            10, 11, 20, 21, 30, 31, // line 0
            40, 41, 50, 51, 60, 61, // line 1
        ]);
        let frame = decode(&data, 2, 2, 3, 12);
        assert!(frame.ir.is_none());
        assert_eq!(frame.rgb.get_pixel(0, 0).0, [10u16, 20, 30]);
        assert_eq!(frame.rgb.get_pixel(1, 0).0, [11u16, 21, 31]);
        assert_eq!(frame.rgb.get_pixel(0, 1).0, [40u16, 50, 60]);
        assert_eq!(frame.rgb.get_pixel(1, 1).0, [41u16, 51, 61]);
    }

    #[test]
    fn reads_samples_big_endian() {
        // A sample above 255 has to decode from both wire bytes, not just the low one.
        // Width 1 pads each plane to 2 samples: R at 0, G at 2, B at 4.
        let data = be(&[0x0102, 0, 0x0304, 0, 0x0506, 0]);
        let frame = decode(&data, 1, 1, 3, 12);
        assert_eq!(frame.rgb.get_pixel(0, 0).0, [0x0102u16, 0x0304, 0x0506]);
    }

    #[test]
    fn splits_rgb_and_ir_planes() {
        // 2x2 with four planes a line: R G B I, eight samples, so stride 16
        let data = be(&[
            10, 11, 20, 21, 30, 31, 90, 91, // line 0
            40, 41, 50, 51, 60, 61, 92, 93, // line 1
        ]);
        let frame = decode(&data, 2, 2, 4, 16);
        assert_eq!(frame.rgb.get_pixel(0, 0).0, [10u16, 20, 30]);
        assert_eq!(frame.rgb.get_pixel(1, 1).0, [41u16, 51, 61]);
        let ir = frame.ir.expect("IR plane present");
        assert_eq!(ir.dimensions(), (2, 2));
        assert_eq!(ir.get_pixel(0, 0).0, [90u16]);
        assert_eq!(ir.get_pixel(1, 1).0, [93u16]);
    }

    #[test]
    fn skips_the_pad_sample_of_an_odd_width_plane() {
        // Width 1 makes each plane 2 samples, one real and one pad
        let data = be(&[
            9, 0xAAAA, 8, 0xBBBB, 7, 0xCCCC, // line 0
            1, 0xA0A0, 2, 0xB0B0, 3, 0xC0C0, // line 1
        ]);
        let frame = decode(&data, 1, 2, 3, 12);
        assert_eq!(frame.rgb.get_pixel(0, 0).0, [9u16, 8, 7]);
        assert_eq!(frame.rgb.get_pixel(0, 1).0, [1u16, 2, 3]);
    }

    #[test]
    fn drops_the_block_padding() {
        // 2x1 needs six samples (12 bytes); stride 16 leaves two pad samples
        let data = be(&[5, 6, 15, 16, 25, 26, 0xEEEE, 0xFFFF]);
        let frame = decode(&data, 2, 1, 3, 16);
        assert_eq!(frame.rgb.get_pixel(0, 0).0, [5u16, 15, 25]);
        assert_eq!(frame.rgb.get_pixel(1, 0).0, [6u16, 16, 26]);
    }

    #[test]
    fn reassembles_a_line_split_across_pushes() {
        let mut decoder = planar_decoder(2, 1, 3, 12);
        let line = be(&[5, 6, 15, 16, 25, 26]);
        decoder.push(&line[..5]).unwrap(); // split mid-sample
        decoder.push(&line[5..]).unwrap();
        let frame = decoder.finish().unwrap().to_owned();
        assert_eq!(frame.rgb.get_pixel(0, 0).0, [5u16, 15, 25]);
        assert_eq!(frame.rgb.get_pixel(1, 0).0, [6u16, 16, 26]);
    }

    /// The LS-5000 reads in 512-aligned bulk chunks rather than a line at a time, so a chunk
    /// boundary lands mid-line far more often than it does on the LS-50
    #[test]
    fn bulk_chunks_that_straddle_lines_decode_the_same() {
        let lines = be(&[
            10, 11, 20, 21, 30, 31, // line 0
            40, 41, 50, 51, 60, 61, // line 1
            70, 71, 80, 81, 90, 91, // line 2
        ]);
        let mut decoder = planar_decoder(2, 3, 3, 12);
        for chunk in lines.chunks(7) {
            decoder.push(chunk).unwrap();
        }
        let frame = decoder.finish().unwrap().to_owned();
        assert_eq!(frame.rgb.get_pixel(0, 0).0, [10u16, 20, 30]);
        assert_eq!(frame.rgb.get_pixel(1, 1).0, [41u16, 51, 61]);
        assert_eq!(frame.rgb.get_pixel(0, 2).0, [70u16, 80, 90]);
    }

    /// A pass that stops early is a failed scan, not a short image
    #[test]
    fn short_stream_is_an_error() {
        let mut decoder = planar_decoder(2, 2, 3, 12);
        decoder.push(&be(&[5, 6, 15, 16, 25, 26])).unwrap();
        assert!(matches!(
            decoder.finish(),
            Err(DecodeError::LengthMismatch(LengthMismatch {
                got: 12,
                expected: 24
            }))
        ));
    }

    #[test]
    fn overlong_stream_is_an_error() {
        let mut decoder = planar_decoder(2, 1, 3, 12);
        assert!(matches!(
            decoder.push(&[0; 13]),
            Err(DecodeError::LengthMismatch { .. })
        ));
    }
}
