//! Turning the scan stream into an image. Section 2-11-3
//!
//! The stream carries no header, so [`Layout`] is the only description of it.
//! Bytes arrive in whatever lengths the transport allows, so a decoder holds
//! whatever part of a line the last chunk left and writes complete lines only.
//!
//! Output is one sample per channel per pixel, channels in the order `Layout`
//! lists them, so a caller can wrap it as `(lines, pixels, channels)` without
//! copying.

use crate::{error::Error, protocol::caps::set_window::ColorInterleaving, protocol::image::Layout};

/// A stream we cannot make an image of
fn bad(reason: String) -> Error {
    Error::Unsupported {
        op: "decode",
        reason,
    }
}

/// How a stream is scrambled, and where one block of it belongs
///
/// Orderings differ in the size of a block and in where its samples land. They
/// do not differ in how bytes arrive, so [`Decoder`] owns that part.
enum Ordering {
    /// 2-11-3-1 format 1, one wire line per output line
    ///
    /// Every sample comes off a single CCD row, so there is no inter-line
    /// mismatch here and nothing for the CCD curves to correct
    Lines(Lines),
}

impl Ordering {
    /// Bytes one [`emit`](Self::emit) consumes
    fn block_bytes(&self) -> usize {
        match self {
            Self::Lines(l) => l.pixels * l.channels * l.bytes_per_sample,
        }
    }

    /// Blocks the layout promises
    fn blocks(&self) -> usize {
        match self {
            Self::Lines(l) => l.lines,
        }
    }

    /// Samples the output holds
    fn samples(&self) -> usize {
        match self {
            Self::Lines(l) => l.lines * l.pixels * l.channels,
        }
    }

    /// Put block `n` where it belongs
    fn emit(&self, n: usize, block: &[u8], out: &mut [u16]) {
        match self {
            Self::Lines(l) => l.emit(n, block, out),
        }
    }
}

/// 2-11-3-1 format 1: a line holds each channel's row end to end, and the
/// output wants them interleaved per pixel
struct Lines {
    pixels: usize,
    lines: usize,
    channels: usize,
    bytes_per_sample: usize,
}

impl Lines {
    fn emit(&self, n: usize, line: &[u8], out: &mut [u16]) {
        let row = n * self.pixels * self.channels;
        for (channel, plane) in line
            .chunks_exact(self.pixels * self.bytes_per_sample)
            .enumerate()
        {
            for (x, sample) in plane.chunks_exact(self.bytes_per_sample).enumerate() {
                out[row + x * self.channels + channel] = sample_at(sample);
            }
        }
    }
}

/// One big-endian sample, whichever width 2-11-3 gave it
fn sample_at(sample: &[u8]) -> u16 {
    match sample {
        [b] => u16::from(*b),
        [hi, lo] => u16::from_be_bytes([*hi, *lo]),
        _ => unreachable!("the width is checked when the decoder is built"),
    }
}

/// Unscrambles a scan into a caller-owned buffer
///
/// A chunk can end anywhere, so partial blocks are held until the rest arrives
pub struct Decoder {
    ordering: Ordering,
    /// A block the last chunk ended part-way through
    carry: Vec<u8>,
    /// Blocks emitted so far
    done: usize,
}

impl Decoder {
    /// A decoder for a stream shaped like `layout`
    ///
    /// Only [`LINE_WITHOUT_DISTANCE`](ColorInterleaving::LINE_WITHOUT_DISTANCE)
    /// so far. The three-line ordering puts the sensor bar on the output's Y
    /// axis and reads it out backwards, tiles stage positions against CCD lines
    /// in blocks of the line gap, and gives each channel and multi-sample repeat
    /// its own readout slot. It is another [`Ordering`], and the one the CCD
    /// correction belongs in; `FrameTranspose` on the pre-rewrite `main` is a
    /// working implementation of the geometry.
    pub fn new(layout: &Layout) -> Result<Self, Error> {
        if !layout
            .interleaving
            .contains(ColorInterleaving::LINE_WITHOUT_DISTANCE)
        {
            return Err(bad(format!(
                "{:?} is not an ordering this decodes yet",
                layout.interleaving
            )));
        }
        // Repeats have no place to sit in a format that is one line per line,
        // and no capture has ever paired the two, so there is nothing to copy
        if layout.readings_per_line > 1 {
            return Err(bad(format!(
                "{} readings a line is not an ordering this decodes yet",
                layout.readings_per_line
            )));
        }
        if !matches!(layout.bytes_per_sample, 1 | 2) {
            return Err(bad(format!(
                "{} bytes a sample is neither of the widths 2-11-3 defines",
                layout.bytes_per_sample
            )));
        }

        let ordering = Ordering::Lines(Lines {
            pixels: layout.pixels as usize,
            lines: layout.lines as usize,
            channels: layout.channels.len(),
            bytes_per_sample: usize::from(layout.bytes_per_sample),
        });
        Ok(Self {
            carry: Vec::with_capacity(ordering.block_bytes()),
            ordering,
            done: 0,
        })
    }

    /// Samples the output buffer has to hold
    pub fn samples(&self) -> usize {
        self.ordering.samples()
    }

    /// Blocks emitted so far, of the [`Layout`]'s total
    pub fn decoded(&self) -> usize {
        self.done
    }

    /// Whether every block the layout promised arrived
    pub fn complete(&self) -> bool {
        self.done == self.ordering.blocks()
    }

    /// Feed the next chunk, writing whatever blocks it completes into `out`
    ///
    /// Bytes past the last block the layout promised are dropped: the unit pads
    /// a short read rather than truncating one.
    pub fn push(&mut self, chunk: &[u8], out: &mut [u16]) -> Result<(), Error> {
        if out.len() < self.samples() {
            return Err(bad(format!(
                "the output holds {} samples and this stream needs {}",
                out.len(),
                self.samples()
            )));
        }

        let width = self.ordering.block_bytes();
        let mut rest = chunk;

        // Finish the block the last chunk ended part-way through
        if !self.carry.is_empty() {
            let take = (width - self.carry.len()).min(rest.len());
            self.carry.extend_from_slice(&rest[..take]);
            rest = &rest[take..];
            if self.carry.len() < width {
                return Ok(());
            }
            let block = std::mem::take(&mut self.carry);
            self.take(&block, out);
            self.carry = block;
            self.carry.clear();
        }

        let mut blocks = rest.chunks_exact(width);
        for block in &mut blocks {
            self.take(block, out);
        }
        self.carry.extend_from_slice(blocks.remainder());
        Ok(())
    }

    /// Emit one block, unless the layout has already had every block it wanted
    fn take(&mut self, block: &[u8], out: &mut [u16]) {
        if self.done >= self.ordering.blocks() {
            return;
        }
        self.ordering.emit(self.done, block, out);
        self.done += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::image::Layout;

    /// A 1494 x 1494 three-channel 16-bit stream, which is what a 666 dpi
    /// single-line pass over a 6x6 frame produces
    fn layout(pixels: u32, lines: u32, channels: Vec<u8>) -> Layout {
        Layout::single_line(pixels, lines, channels)
    }

    /// Channel `c` of pixel `x` on line `y` carries a value that says so
    fn stream(pixels: usize, lines: usize, channels: usize) -> Vec<u8> {
        let mut out = Vec::new();
        for y in 0..lines {
            for c in 0..channels {
                for x in 0..pixels {
                    let v = (y * 1000 + x * 10 + c) as u16;
                    out.extend_from_slice(&v.to_be_bytes());
                }
            }
        }
        out
    }

    /// The wire puts a whole channel down before the next; the output
    /// interleaves them per pixel
    #[test]
    fn a_line_of_planes_comes_out_interleaved() {
        let l = layout(4, 3, vec![1, 2, 3]);
        let mut d = Decoder::new(&l).unwrap();
        let mut out = vec![0u16; d.samples()];
        d.push(&stream(4, 3, 3), &mut out).unwrap();

        assert!(d.complete());
        for y in 0..3usize {
            for x in 0..4usize {
                for c in 0..3usize {
                    let got = out[(y * 4 + x) * 3 + c];
                    assert_eq!(got, (y * 1000 + x * 10 + c) as u16, "{y},{x},{c}");
                }
            }
        }
    }

    /// The transport splits where it likes, including mid-sample
    #[test]
    fn a_stream_split_anywhere_decodes_the_same() {
        let l = layout(4, 3, vec![1, 2, 3]);
        let whole = stream(4, 3, 3);

        let mut want = vec![0u16; Decoder::new(&l).unwrap().samples()];
        Decoder::new(&l).unwrap().push(&whole, &mut want).unwrap();

        for split in [1usize, 3, 7, 16, 23, 48] {
            let mut d = Decoder::new(&l).unwrap();
            let mut got = vec![0u16; d.samples()];
            for piece in whole.chunks(split) {
                d.push(piece, &mut got).unwrap();
            }
            assert!(d.complete(), "split {split} left {} lines", d.decoded());
            assert_eq!(got, want, "split {split}");
        }
    }

    /// A short read leaves the tail of the image as it found it
    #[test]
    fn a_short_stream_writes_only_what_arrived() {
        let l = layout(4, 3, vec![1, 2, 3]);
        let mut d = Decoder::new(&l).unwrap();
        let mut out = vec![0u16; d.samples()];
        let whole = stream(4, 3, 3);
        d.push(&whole[..whole.len() / 3], &mut out).unwrap();

        assert!(!d.complete());
        assert_eq!(d.decoded(), 1);
        assert_eq!(&out[4 * 3..], [0u16; 4 * 3 * 2]);
    }

    /// The unit pads rather than truncating, so anything past the last line
    /// the layout promised is not ours to write
    #[test]
    fn padding_past_the_last_line_is_dropped() {
        let l = layout(4, 2, vec![1, 2, 3]);
        let mut d = Decoder::new(&l).unwrap();
        let mut out = vec![0u16; d.samples()];
        d.push(&stream(4, 4, 3), &mut out).unwrap();

        assert_eq!(d.decoded(), 2);
        assert!(d.complete());
    }

    /// A real 666 dpi pass off an LS-9000, 1494 x 1494 x 3 at 16 bits, decoded
    /// in the 256 KB pieces the transport hands over
    ///
    /// The capture is not checked in, so this reports and passes without it
    #[test]
    fn a_real_pass_decodes_into_a_photograph() {
        let Ok(raw) = std::fs::read("scan.raw") else {
            eprintln!("no scan.raw, skipping");
            return;
        };
        let (w, h) = (1494usize, 1494usize);
        let l = layout(w as u32, h as u32, vec![1, 2, 3]);
        let mut d = Decoder::new(&l).unwrap();
        assert_eq!(raw.len(), d.samples() * 2);

        let mut out = vec![0u16; d.samples()];
        for piece in raw.chunks(262_144) {
            d.push(piece, &mut out).unwrap();
        }
        assert!(d.complete());

        // A photograph, not noise: neighbouring pixels agree far better than
        // distant ones. Scrambled planes would flatten the difference
        let at = |y: usize, x: usize, c: usize| f64::from(out[(y * w + x) * 3 + c]);
        let (mut near, mut far, mut n) = (0.0, 0.0, 0.0);
        for y in (10..h - 10).step_by(37) {
            for x in (10..w - 800).step_by(37) {
                near += (at(y, x, 1) - at(y, x + 1, 1)).abs();
                far += (at(y, x, 1) - at(y, x + 700, 1)).abs();
                n += 1.0;
            }
        }
        eprintln!("neighbour {:.0}, distant {:.0}", near / n, far / n);
        assert!(near * 4.0 < far, "near {} far {}", near / n, far / n);
    }

    #[test]
    fn an_output_too_small_is_refused() {
        let l = layout(4, 3, vec![1, 2, 3]);
        let mut d = Decoder::new(&l).unwrap();
        assert!(d.push(&[], &mut [0u16; 4]).is_err());
    }
}
