//! Decoding the LS-9000's raw scan stream into an image.

use super::geometry::ScanSettings;
use crate::decode::{
    BlockSink, Blocked, ImageView, LengthMismatch, Rgb16, StreamDecoder, be_u16_at,
};
use crate::scanners::ScanArea;
use image::{ImageBuffer, Rgb};

// Shared with every other scanner here, and re-exported so callers of this module need not
// know that

/// Sensor pixels processed per inner tile, chosen so both the input runs and the output tile stay in L2 during the transpose
const CHUNK: usize = 256;

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("scan window does not divide evenly at this resolution")]
    IndivisibleWindow,

    #[error(
        "stage extent gives {stages} positions, not a multiple of the {block}-position CCD block"
    )]
    UnalignedStageExtent { stages: u32, block: u32 },

    #[error(transparent)]
    LengthMismatch(#[from] LengthMismatch),
}

/// Decoder for the 83-DPI overview pass, one row of pixels at a time
///
/// Values seem to come back in linear ADC counts
/// So, you'd need to apply some gamma curve to make it "look" right
pub struct OverviewRows {
    width: usize,
    height: usize,
    rgb: Vec<u16>,
    /// Rows decoded so far
    rows: usize,
}

/// The 83-DPI overview pass
pub type OverviewDecoder = Blocked<OverviewRows>;

impl Default for OverviewDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl OverviewDecoder {
    pub fn new() -> Self {
        Self::wrap(OverviewRows::new())
    }
}

impl OverviewRows {
    fn new() -> Self {
        let (width, height) = ScanArea::overview_dims();
        let (width, height) = (width as usize, height as usize);
        Self {
            width,
            height,
            rgb: vec![0; width * height * 3],
            rows: 0,
        }
    }
}

impl BlockSink for OverviewRows {
    type Output<'a> = ImageBuffer<Rgb<u16>, &'a [u16]>;
    type Error = DecodeError;

    fn block_len(&self) -> usize {
        self.width * 3 * 2
    }

    fn blocks(&self) -> u64 {
        self.height as u64
    }

    /// One row is three consecutive planes of `width` samples, one per channel, which the output wants interleaved per pixel instead
    fn emit(&mut self, row: &[u8]) {
        let y = self.rows;
        for (channel, plane) in row.chunks_exact(self.width * 2).enumerate() {
            for (x, sample) in plane.chunks_exact(2).enumerate() {
                self.rgb[(y * self.width + x) * 3 + channel] = be_u16_at(sample, 0);
            }
        }
        self.rows += 1;
    }

    fn finish(&mut self) -> Result<Self::Output<'_>, DecodeError> {
        Ok(
            ImageBuffer::from_raw(self.width as u32, self.height as u32, &self.rgb[..])
                .expect("buffer is sized from the dims"),
        )
    }
}

/// Decode a complete overview pass already held in memory
pub fn decode_overview(bytes: &[u8]) -> Result<Rgb16, DecodeError> {
    let mut decoder = OverviewDecoder::new();
    decoder.push(bytes)?;
    let view = decoder.finish()?;
    Ok(Rgb16::from_raw(view.width(), view.height(), view.to_vec()).expect("view is well formed"))
}

/// Unscrambles a full-resolution frame, one CCD interleave block at a time
///
/// A *sample* is one 16-bit value: one channel, at one sensor pixel, from one CCD line, in one readout
pub struct FrameTranspose {
    // --- output geometry ---
    /// Output columns (stage positions x CCD lines)
    width: usize,
    /// Output rows (active sensor pixels)
    height: usize,

    // --- resolved acquisition parameters ---
    /// CCD lines per readout: 3, or 1 in single-line mode
    lines: usize,
    /// Stage positions per interleave block; equivalently the CCD line spacing in output columns, 12 dots over the stage divisor. 1 in single-line mode, and 1 in a preview, where the stage steps exactly one line spacing per column
    block: usize,
    /// Multi-sample repeats per stage position
    multisample: usize,
    /// Whether an infrared readout is present
    ir: bool,
    /// Samples in one readout: one sweep of the sensor bar across all lines
    readout_samples: usize,
    /// Samples per stage position (`readouts * readout_samples`)
    stage_stride: usize,
    /// Interleave blocks the pass will deliver
    total_blocks: u64,

    // --- unscrambled output ---
    rgb: Vec<u16>,
    ir_plane: Vec<u16>,
    /// Blocks emitted so far; the left edge of the current output strip
    block_index: usize,
}

/// A full-resolution frame
pub type FrameDecoder = Blocked<FrameTranspose>;

impl FrameDecoder {
    pub fn new(settings: &ScanSettings) -> Result<Self, DecodeError> {
        Ok(Self::wrap(FrameTranspose::new(settings)?))
    }
}

impl FrameTranspose {
    fn new(settings: &ScanSettings) -> Result<Self, DecodeError> {
        let (width, height) = settings
            .output_dims()
            .ok_or(DecodeError::IndivisibleWindow)?;
        let (stages, block) = (
            settings.stages().ok_or(DecodeError::IndivisibleWindow)?,
            settings.ccd_block(),
        );
        if stages % block != 0 {
            return Err(DecodeError::UnalignedStageExtent { stages, block });
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
            total_blocks: u64::from(stages / block),
            rgb: vec![0; px * 3],
            ir_plane: if settings.ir { vec![0; px] } else { Vec::new() },
            block_index: 0,
        })
    }

    /// Which readout slot holds channel `c` on repeat `s`
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

    /// Transpose the freshly filled block into its output strip
    ///
    /// The CCD's lines sit [`block`](Self::block) output columns apart, so a block of that many
    /// stage positions is what it takes to tile a contiguous run of columns: the stage advances
    /// one column per position, and line `l` lays its samples down `l * block` columns ahead of
    /// line 0. A block's `block * lines` columns therefore run `[line 0 x block][line 1 x
    /// block][line 2 x block]`, and the strip column splits back into a stage position and a
    /// line. Iterating column-outer, sensor-inner keeps a chunk of the output column in cache
    /// while the input is read sequentially down the bar.
    fn emit_block(&mut self, staging: &[u8]) {
        let first_col = self.block_index * self.block * self.lines;
        let strip_cols = self.block * self.lines;
        let rsamp = self.readout_samples;

        let mut p0 = 0;
        while p0 < self.height {
            let p_end = (p0 + CHUNK).min(self.height);

            for col in 0..strip_cols {
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
                            be_u16_at(staging, base),
                            be_u16_at(staging, base + rsamp),
                            be_u16_at(staging, base + 2 * rsamp),
                        ]
                    } else {
                        let m = self.multisample as u32;
                        let mut t = [0u16; 3];
                        for (channel, out) in t.iter_mut().enumerate() {
                            let mut acc = 0u32;
                            for rep in 0..self.multisample {
                                let idx = base + self.readout_of(channel, rep) * rsamp;
                                acc += u32::from(be_u16_at(staging, idx));
                            }
                            *out = (acc / m) as u16;
                        }
                        t
                    };
                    self.rgb[out3..out3 + 3].copy_from_slice(&rgb);

                    if self.ir {
                        // IR is readout slot 3, present only on repeat 0.
                        self.ir_plane[y * self.width + x] = be_u16_at(staging, base + 3 * rsamp);
                    }
                }
            }
            p0 = p_end;
        }
    }
}

impl BlockSink for FrameTranspose {
    type Output<'a> = ImageView<'a>;
    type Error = DecodeError;

    fn block_len(&self) -> usize {
        self.block * self.stage_stride * 2
    }

    fn blocks(&self) -> u64 {
        self.total_blocks
    }

    fn emit(&mut self, block: &[u8]) {
        self.emit_block(block);
        self.block_index += 1;
    }

    /// `new()` guarantees the stage count is a whole number of blocks, so there is never a
    /// trailing partial block to flush here
    fn finish(&mut self) -> Result<ImageView<'_>, DecodeError> {
        let (w, h) = (self.width as u32, self.height as u32);
        Ok(ImageView {
            rgb: ImageBuffer::from_raw(w, h, self.rgb.as_slice()).expect("buffer sized in new"),
            ir: self.ir.then(|| {
                ImageBuffer::from_raw(w, h, self.ir_plane.as_slice()).expect("sized in new")
            }),
        })
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
        let (width, height) = ScanArea::overview_dims();
        assert_eq!((width, height), (186, 721));
        // The exact byte count read back off the scanner
        assert_eq!(2 * 3 * width * height, 804_636);
    }

    #[test]
    fn deinterleaves_line_sequential_rows() {
        let (width, height) = ScanArea::overview_dims();
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
        let (width, height) = ScanArea::overview_dims();
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
        let (width, height) = ScanArea::overview_dims();
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
            Err(DecodeError::LengthMismatch(LengthMismatch {
                got: 1116,
                ..
            }))
        ));
    }

    #[test]
    fn wrong_length_is_an_error() {
        assert!(matches!(
            decode_overview(&[0u8; 100]),
            Err(DecodeError::LengthMismatch(LengthMismatch {
                got: 100,
                expected: 804_636
            }))
        ));
    }
}

#[cfg(test)]
mod frame_tests {
    use super::*;
    use crate::scanners::ls9000::{
        geometry::{CcdMode, Dpi, Multisample},
        window::BaseQuality,
    };

    /// A deliberately tiny frame: single-line CCD, no multi-sample, no IR
    /// 4x8 output, 3 readouts of 4 samples per stage, 8 stages, 192 bytes, 24 per block.
    fn settings() -> ScanSettings {
        ScanSettings {
            ccd_mode: CcdMode::SingleLine,
            ir: false,
            dpi: Dpi::_4000,
            quality: BaseQuality::Scan,
            multisample: Multisample::X1,
            window: ScanArea {
                x_pos: 0,
                y_pos: 0,
                x_size: 4,
                y_size: 8,
            },
        }
    }

    fn tagged(len: usize) -> Vec<u8> {
        (0..len / 2)
            .flat_map(|i| ((i as u16) & 0x3FFF).to_be_bytes())
            .collect()
    }

    fn decode_in_chunks(settings: &ScanSettings, stream: &[u8], size: usize) -> Vec<u16> {
        let mut decoder = FrameDecoder::new(settings).unwrap();
        for chunk in stream.chunks(size) {
            decoder.push(chunk).unwrap();
        }
        decoder.finish().unwrap().rgb.iter().copied().collect()
    }

    /// The scanner's chunks have no reason to land on interleave-block boundaries
    #[test]
    fn chunking_does_not_change_the_result() {
        let settings = settings();
        let stream = tagged(settings.expected_bytes().unwrap() as usize);
        let reference = decode_in_chunks(&settings, &stream, stream.len());

        // A block is 24 bytes: sizes that divide it, straddle it, and dwarf it
        for size in [1, 7, 23, 24, 25, 100, 191, 1000] {
            assert_eq!(
                decode_in_chunks(&settings, &stream, size),
                reference,
                "chunk size {size}"
            );
        }
    }

    /// The three-line CCD lays its lines down `ccd_block` output columns apart, so a block of
    /// that many stage positions tiles `block * 3` columns as `[line 0 x block][line 1 x
    /// block][line 2 x block]`.
    ///
    /// Measured off a 2000-DPI three-line capture with the seam probe: at the right block size
    /// the adjacent-column correlation is flat, and every other size leaves a periodic dip.
    /// Single-line settings can't catch a regression here, since their block is 1.
    #[test]
    fn three_line_columns_are_block_interleaved() {
        // 2000 DPI puts the lines 6 columns apart, so one block is 6 stage positions and 18
        // columns: exactly one block wide, 2 sensor pixels tall.
        let settings = ScanSettings {
            ccd_mode: CcdMode::ThreeLine,
            dpi: Dpi::_2000,
            window: ScanArea {
                x_pos: 0,
                y_pos: 0,
                x_size: 4,
                y_size: 36,
            },
            ..settings()
        };
        let (block, lines) = (settings.ccd_block() as usize, 3);
        assert_eq!((block, settings.stages()), (6, Some(6)));
        let (width, height) = settings.output_dims().unwrap();
        let (width, height) = (width as usize, height as usize);
        assert_eq!((width, height), (18, 2));

        let stream = tagged(settings.expected_bytes().unwrap() as usize);
        let got = decode_in_chunks(&settings, &stream, stream.len());

        let (rsamp, stage_stride) = (height * lines, 3 * height * lines);
        for x in 0..width {
            // The inverse of the layout above: a column names its line and stage position.
            let (b, c) = (x / (block * lines), x % (block * lines));
            let (line, stage) = (c / block, c % block);
            let g = b * block + stage;
            for y in 0..height {
                // The sensor bar reads out opposite to increasing y.
                let p = height - 1 - y;
                for channel in 0..3 {
                    let sample = g * stage_stride + channel * rsamp + p * lines + line;
                    assert_eq!(
                        got[(y * width + x) * 3 + channel],
                        (sample as u16) & 0x3FFF,
                        "channel {channel} at {x},{y}"
                    );
                }
            }
        }
    }

    #[test]
    fn over_long_stream_is_an_error() {
        let settings = settings();
        let mut decoder = FrameDecoder::new(&settings).unwrap();
        let too_much = tagged(settings.expected_bytes().unwrap() as usize + 2);
        assert!(matches!(
            decoder.push(&too_much),
            Err(DecodeError::LengthMismatch(_))
        ));
    }

    #[test]
    fn finish_before_the_stream_completes_is_an_error() {
        let settings = settings();
        let mut decoder = FrameDecoder::new(&settings).unwrap();
        decoder.push(&tagged(24)).unwrap();
        assert!(matches!(
            decoder.finish(),
            Err(DecodeError::LengthMismatch(LengthMismatch { got: 24, .. }))
        ));
    }
}
