//! Decoding the LS-9000's raw scan stream into an image.

use super::ScanSettings;
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

    /// Sample index of one value in the staging block.
    ///
    /// The block is stage-major: for each stage position come `readouts`
    /// readouts, each a sweep of `height` sensor pixels across `lines` CCD
    /// lines. This is the inverse of that layout.
    #[inline]
    fn sample_index(&self, stage: usize, readout: usize, p: usize, line: usize) -> usize {
        stage * self.stage_stride + readout * self.readout_samples + p * self.lines + line
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

                for p in p0..p_end {
                    // The sensor bar reads out opposite to increasing y
                    let y = self.height - 1 - p;
                    let out = y * self.width + x;

                    // Single-sample is the common case, skip the accumulate and the (runtime, non-power-of-two) divide.
                    // The readout slot for repeat 0 of channel c is just c.
                    if self.multisample == 1 {
                        for channel in 0..3 {
                            let idx = self.sample_index(stage, channel, p, line);
                            self.rgb[out * 3 + channel] = sample_at(&self.staging, idx);
                        }
                    } else {
                        for channel in 0..3 {
                            let mut acc = 0u32;
                            for rep in 0..self.multisample {
                                let idx = self.sample_index(
                                    stage,
                                    self.readout_of(channel, rep),
                                    p,
                                    line,
                                );
                                acc += u32::from(sample_at(&self.staging, idx));
                            }
                            self.rgb[out * 3 + channel] = (acc / self.multisample as u32) as u16;
                        }
                    }

                    if self.ir {
                        let idx = self.sample_index(stage, self.readout_of(3, 0), p, line);
                        self.ir_plane[out] = sample_at(&self.staging, idx);
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
