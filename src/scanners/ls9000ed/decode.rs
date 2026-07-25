//! Decoding the LS-9000's raw scan stream into an image.

use super::{ScanSettings, Window};
use crate::decode::StreamDecoder;
use image::{ImageBuffer, Luma, Rgb};

/// Sensor pixels processed per inner tile, chosen so both the input runs and the output tile stay in L2 during the transpose
const CHUNK: usize = 256;

// Output iamge types
// The LS-9000 always sends BE u16 over the wire
pub type Image = ImageBuffer<Rgb<u16>, Vec<u16>>;
pub type IrMask = ImageBuffer<Luma<u16>, Vec<u16>>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("scan window does not divide evenly at this resolution")]
    IndivisibleWindow,

    #[error(
        "stage extent gives {stages} positions, not a multiple of the {block}-position CCD block"
    )]
    UnalignedStageExtent { stages: u32, block: u32 },

    #[error("received {got} bytes, expected {expected}")]
    LengthMismatch { got: u64, expected: u64 },
}

/// A decoded frame that borrows the decoder's buffers
pub struct FrameView<'a> {
    /// The image data read out from the scanner
    pub rgb: ImageBuffer<Rgb<u16>, &'a [u16]>,
    /// The optional IR mask for dust removal
    pub ir: Option<ImageBuffer<Luma<u16>, &'a [u16]>>,
}

impl FrameView<'_> {
    /// Copy into owned buffers, so the frame outlives the decoder's reuse
    pub fn to_owned(&self) -> Frame {
        Frame {
            rgb: Image::from_raw(self.rgb.width(), self.rgb.height(), self.rgb.to_vec())
                .expect("view is well formed"),
            ir: self.ir.as_ref().map(|ir| {
                IrMask::from_raw(ir.width(), ir.height(), ir.to_vec()).expect("view is well formed")
            }),
        }
    }
}

/// An owned decoded frame
pub struct Frame {
    /// The image data read out from the scanner
    pub rgb: Image,
    /// The optional IR mask for dust removal
    pub ir: Option<IrMask>,
}

/// Decoder for the 83-DPI overview pass
///
/// Values seem to come back in linear ADC counts.
/// So, you'd need to apply some gamma curve to make it "look" right
pub struct OverviewDecoder {
    width: usize,
    height: usize,
    rgb: Vec<u16>,
    /// Bytes of a row that arrived split across chunks
    partial: Vec<u8>,
    /// Rows decoded so far
    rows: usize,
    received: u64,
}

impl Default for OverviewDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl OverviewDecoder {
    pub fn new() -> Self {
        let (width, height) = Window::overview_dims();
        let (width, height) = (width as usize, height as usize);
        Self {
            width,
            height,
            rgb: vec![0; width * height * 3],
            partial: Vec::with_capacity(width * 6),
            rows: 0,
            received: 0,
        }
    }

    fn row_bytes(&self) -> usize {
        self.width * 3 * 2
    }

    /// One row is three consecutive planes of `width` samples, one per channel, which the output wants interleaved per pixel instead
    fn emit_row(&mut self, row: &[u8]) {
        let y = self.rows;
        for (channel, plane) in row.chunks_exact(self.width * 2).enumerate() {
            for (x, sample) in plane.chunks_exact(2).enumerate() {
                self.rgb[(y * self.width + x) * 3 + channel] =
                    u16::from_be_bytes([sample[0], sample[1]]);
            }
        }
        self.rows += 1;
    }
}

impl StreamDecoder for OverviewDecoder {
    type Output<'a> = ImageBuffer<Rgb<u16>, &'a [u16]>;
    type Error = Error;

    fn expected_bytes(&self) -> u64 {
        (self.row_bytes() * self.height) as u64
    }

    fn push(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.received += bytes.len() as u64;
        if self.received > self.expected_bytes() {
            return Err(Error::LengthMismatch {
                got: self.received,
                expected: self.expected_bytes(),
            });
        }

        let row_bytes = self.row_bytes();
        let mut rest = bytes;

        // Top up a row left half-finished by the previous chunk
        if !self.partial.is_empty() {
            let take = (row_bytes - self.partial.len()).min(rest.len());
            self.partial.extend_from_slice(&rest[..take]);
            rest = &rest[take..];
            if self.partial.len() == row_bytes {
                let row = std::mem::take(&mut self.partial);
                self.emit_row(&row);
                self.partial = row;
                self.partial.clear();
            }
        }

        // Whole rows straight out of the caller's buffer, then stash the tail
        let mut rows = rest.chunks_exact(row_bytes);
        for row in &mut rows {
            self.emit_row(row);
        }
        self.partial.extend_from_slice(rows.remainder());
        Ok(())
    }

    fn finish(&mut self) -> Result<Self::Output<'_>, Error> {
        if self.received != self.expected_bytes() {
            return Err(Error::LengthMismatch {
                got: self.received,
                expected: self.expected_bytes(),
            });
        }
        Ok(
            ImageBuffer::from_raw(self.width as u32, self.height as u32, &self.rgb[..])
                .expect("buffer is sized from the dims"),
        )
    }
}

/// Decode a complete overview pass already held in memory
pub fn decode_overview(bytes: &[u8]) -> Result<Image, Error> {
    let mut decoder = OverviewDecoder::new();
    decoder.push(bytes)?;
    let view = decoder.finish()?;
    Ok(Image::from_raw(view.width(), view.height(), view.to_vec()).expect("view is well formed"))
}

/// Streaming decoder.
/// Feed `READ(10)` payloads to [`push`](Self::push) in arrival order, then call [`finish`](Self::finish)
///
/// A *sample* is one 16-bit value: one channel, at one sensor pixel, from one CCD line, in one readout
pub struct FrameDecoder {
    // --- output geometry ---
    /// Output columns (stage positions x CCD lines)
    width: usize,
    /// Output rows (active sensor pixels)
    height: usize,

    // --- resolved acquisition parameters ---
    /// CCD lines per readout: 3, or 1 in single-line mode
    lines: usize,
    /// Stage positions per interleave block; equivalently the CCD line spacing in output columns (`N = 12/k`). 1 in single-line mode
    block: usize,
    /// Multi-sample repeats per stage position
    multisample: usize,
    /// Whether an infrared readout is present
    ir: bool,
    /// Samples in one readout: one sweep of the sensor bar across all lines
    readout_samples: usize,
    /// Samples per stage position (`readouts * readout_samples`)
    stage_stride: usize,
    /// Total stream length in bytes; the transfer is complete at this count
    expected: u64,

    // --- unscrambled output ---
    rgb: Vec<u16>,
    ir_plane: Vec<u16>,

    // --- streaming state ---
    /// One interleave block of raw bytes, refilled as the stream arrives.
    staging: Vec<u8>,
    /// Bytes currently in `staging`.
    filled: usize,
    /// Blocks emitted so far; the left edge of the current output strip.
    block_index: usize,
    /// Bytes received across all `push` calls.
    received: u64,
}

impl FrameDecoder {
    pub fn new(settings: &ScanSettings) -> Result<Self, Error> {
        let (width, height) = settings.output_dims().ok_or(Error::IndivisibleWindow)?;
        let (stages, block) = (
            settings.stages().ok_or(Error::IndivisibleWindow)?,
            settings.ccd_block(),
        );
        if stages % block != 0 {
            return Err(Error::UnalignedStageExtent { stages, block });
        }

        let (width, height) = (width as usize, height as usize);
        let lines = settings.lines() as usize;
        let readout_samples = height * lines;
        let stage_stride = settings.readouts() as usize * readout_samples;
        let px = width * height;

        Ok(Self {
            width,
            height,
            lines,
            block: block as usize,
            multisample: settings.multisample.count() as usize,
            ir: settings.ir,
            readout_samples,
            stage_stride,
            expected: settings.expected_bytes().ok_or(Error::IndivisibleWindow)?,
            rgb: vec![0; px * 3],
            ir_plane: if settings.ir { vec![0; px] } else { Vec::new() },
            staging: vec![0; block as usize * stage_stride * 2],
            filled: 0,
            block_index: 0,
            received: 0,
        })
    }

    /// Feed the streaming decoder
    pub fn push(&mut self, mut bytes: &[u8]) -> Result<(), Error> {
        self.received += bytes.len() as u64;
        if self.received > self.expected {
            return Err(Error::LengthMismatch {
                got: self.received,
                expected: self.expected,
            });
        }
        while !bytes.is_empty() {
            let take = (self.staging.len() - self.filled).min(bytes.len());
            self.staging[self.filled..self.filled + take].copy_from_slice(&bytes[..take]);
            self.filled += take;
            bytes = &bytes[take..];
            if self.filled == self.staging.len() {
                self.emit();
                self.filled = 0;
                self.block_index += 1;
            }
        }
        Ok(())
    }

    /// Complete the current frame and borrow the decoded buffers.
    ///
    /// The buffers stay in the decoder, so the view is only valid until
    /// [`reset`](Self::reset) or the next [`push`](Self::push). Call
    /// [`FrameView::to_owned`] to keep it. `new()` guarantees the stage count
    /// is a whole number of blocks, so there is never a trailing partial
    /// block to flush here.
    pub fn finish(&mut self) -> Result<FrameView<'_>, Error> {
        if self.received != self.expected {
            return Err(Error::LengthMismatch {
                got: self.received,
                expected: self.expected,
            });
        }
        let (w, h) = (self.width as u32, self.height as u32);
        Ok(FrameView {
            rgb: ImageBuffer::from_raw(w, h, self.rgb.as_slice()).expect("buffer sized in new"),
            ir: self.ir.then(|| {
                ImageBuffer::from_raw(w, h, self.ir_plane.as_slice()).expect("sized in new")
            }),
        })
    }

    /// Prepare to decode another frame with the same settings, reusing every buffer
    pub fn reset(&mut self) {
        self.filled = 0;
        self.block_index = 0;
        self.received = 0;
    }

    /// Which readout slot holds channel `c` on repeat `s`.
    ///
    /// Channels are `0=R, 1=G, 2=B, 3=IR`. Infrared exists only on repeat 0.
    #[inline]
    fn readout_of(&self, c: usize, s: usize) -> usize {
        if c >= 3 {
            3
        } else if s == 0 {
            c
        } else {
            3 + usize::from(self.ir) + (s - 1) * 3 + c
        }
    }

    /// Transpose the freshly filled block into its output strip.
    ///
    /// One block covers `self.block` stage positions, which in three-line mode
    /// map to a contiguous run of `self.block * lines` output columns.
    /// Iterating column-outer, sensor-inner keeps a chunk of the output column
    /// in cache while the input is read sequentially down the bar.
    fn emit(&mut self) {
        let first_col = self.block_index * self.block * self.lines;
        let strip_cols = self.block * self.lines;
        let rsamp = self.readout_samples;

        let mut p0 = 0;
        while p0 < self.height {
            let p_end = (p0 + CHUNK).min(self.height);

            for col in 0..strip_cols {
                // A block's columns run [line 0 x N][line 1 x N][line 2 x N],
                // so the strip column splits into a stage position and a line.
                let (stage, line) = if self.lines == 3 {
                    (col % self.block, col / self.block)
                } else {
                    (col, 0)
                };
                let x = first_col + col;
                // Invariant across the whole sensor sweep for this column.
                let col_base = stage * self.stage_stride + line;

                for p in p0..p_end {
                    // The sensor bar reads out opposite to increasing y.
                    let y = self.height - 1 - p;
                    let out3 = (y * self.width + x) * 3;
                    // Readout 0, channel 0 (= red) of this pixel; other
                    // readouts follow at multiples of `rsamp`.
                    let base = col_base + p * self.lines;

                    // Gather the pixel into a stack triple, then write it in one shot
                    // RGB is interleaved in the output, so the triple is contiguous and this is a single bounds check.
                    let rgb = if self.multisample == 1 {
                        // Readout slot for channel c is just c.
                        [
                            sample_at(&self.staging, base),
                            sample_at(&self.staging, base + rsamp),
                            sample_at(&self.staging, base + 2 * rsamp),
                        ]
                    } else {
                        let m = self.multisample as u32;
                        let mut t = [0u16; 3];
                        for (channel, out) in t.iter_mut().enumerate() {
                            let mut acc = 0u32;
                            for rep in 0..self.multisample {
                                let idx = base + self.readout_of(channel, rep) * rsamp;
                                acc += u32::from(sample_at(&self.staging, idx));
                            }
                            *out = (acc / m) as u16;
                        }
                        t
                    };
                    self.rgb[out3..out3 + 3].copy_from_slice(&rgb);

                    if self.ir {
                        // IR is readout slot 3, present only on repeat 0.
                        self.ir_plane[y * self.width + x] =
                            sample_at(&self.staging, base + 3 * rsamp);
                    }
                }
            }
            p0 = p_end;
        }
    }
}

/// Read one big-endian sample.
#[inline(always)]
fn sample_at(buf: &[u8], i: usize) -> u16 {
    u16::from_be_bytes([buf[2 * i], buf[2 * i + 1]])
}

impl StreamDecoder for FrameDecoder {
    type Output<'a> = FrameView<'a>;
    type Error = Error;

    fn expected_bytes(&self) -> u64 {
        self.expected
    }

    fn push(&mut self, bytes: &[u8]) -> Result<(), Error> {
        FrameDecoder::push(self, bytes)
    }

    fn finish(&mut self) -> Result<FrameView<'_>, Error> {
        FrameDecoder::finish(self)
    }
}

#[cfg(test)]
mod overview_tests {
    use super::*;

    /// Build a stream where every sample encodes its own (x, y, channel), so any
    /// transposition shows up as a wrong value rather than a plausible-looking image.
    fn tagged_stream(width: usize, height: usize) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(width * height * 6);
        for y in 0..height {
            for channel in 0..3 {
                for x in 0..width {
                    let sample = ((y * width + x) * 3 + channel) as u16 & 0x3FFF;
                    bytes.extend_from_slice(&sample.to_be_bytes());
                }
            }
        }
        bytes
    }

    #[test]
    fn matches_the_hardware_overview_size() {
        let (width, height) = Window::overview_dims();
        assert_eq!((width, height), (186, 721));
        // The exact byte count read back off the scanner
        assert_eq!(2 * 3 * width * height, 804_636);
    }

    #[test]
    fn deinterleaves_line_sequential_rows() {
        let (width, height) = Window::overview_dims();
        let image = decode_overview(&tagged_stream(width as usize, height as usize)).unwrap();

        assert_eq!(image.dimensions(), (width, height));
        for (x, y, pixel) in image.enumerate_pixels() {
            let base = (y as usize * width as usize + x as usize) * 3;
            let tag = |channel: usize| ((base + channel) as u16) & 0x3FFF;
            assert_eq!(pixel.0, [tag(0), tag(1), tag(2)], "at {x},{y}");
        }
    }

    #[test]
    fn samples_are_big_endian() {
        let (width, height) = Window::overview_dims();
        let mut bytes = vec![0u8; (width * height * 6) as usize];
        // First sample of the first red row
        bytes[0] = 0x12;
        bytes[1] = 0x34;
        let image = decode_overview(&bytes).unwrap();
        assert_eq!(image.get_pixel(0, 0).0[0], 0x1234);
    }

    /// The scanner's chunks have no reason to land on row boundaries, so pushing the same
    /// stream in awkward sizes has to give the same image as one big push
    #[test]
    fn chunking_does_not_change_the_result() {
        let (width, height) = Window::overview_dims();
        let stream = tagged_stream(width as usize, height as usize);
        let expected = decode_overview(&stream).unwrap();

        // A row is 1116 bytes: sizes that divide it, straddle it, and dwarf it
        for size in [1, 7, 1115, 1116, 1117, 32_364, 500_000] {
            let mut decoder = OverviewDecoder::new();
            for chunk in stream.chunks(size) {
                decoder.push(chunk).unwrap();
            }
            let got = decoder.finish().unwrap();
            assert_eq!(got.dimensions(), expected.dimensions(), "chunk size {size}");
            assert!(
                got.iter().eq(expected.iter()),
                "chunk size {size} decoded differently"
            );
        }
    }

    #[test]
    fn finish_before_the_stream_completes_is_an_error() {
        let mut decoder = OverviewDecoder::new();
        decoder.push(&[0u8; 1116]).unwrap();
        assert!(matches!(
            decoder.finish(),
            Err(Error::LengthMismatch { got: 1116, .. })
        ));
    }

    #[test]
    fn wrong_length_is_an_error() {
        assert!(matches!(
            decode_overview(&[0u8; 100]),
            Err(Error::LengthMismatch {
                got: 100,
                expected: 804_636
            })
        ));
    }
}
