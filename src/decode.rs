//! A trait to encode the behavior of a streaming image decoder

use image::{ImageBuffer, Luma, Rgb};

// What a decoded pass comes out as. Every scanner here sends BE u16 over the wire.
pub type Rgb16 = ImageBuffer<Rgb<u16>, Vec<u16>>;
pub type Luma16 = ImageBuffer<Luma<u16>, Vec<u16>>;

/// A decoded frame that borrows the decoder's buffers
///
/// Borrowed rather than owned so a multi-hundred-MB image needn't be copied out to be looked at.
pub struct ImageView<'a> {
    /// The image data read out from the scanner
    pub rgb: ImageBuffer<Rgb<u16>, &'a [u16]>,
    /// The optional IR mask for dust removal
    pub ir: Option<ImageBuffer<Luma<u16>, &'a [u16]>>,
}

impl ImageView<'_> {
    /// Copy out of the decoder, so the frame outlives it
    pub fn to_owned(&self) -> Image {
        Image {
            rgb: Rgb16::from_raw(self.rgb.width(), self.rgb.height(), self.rgb.to_vec())
                .expect("view is well formed"),
            ir: self.ir.as_ref().map(|ir| {
                Luma16::from_raw(ir.width(), ir.height(), ir.to_vec()).expect("view is well formed")
            }),
        }
    }
}

/// A decoded frame that owns its buffers
pub struct Image {
    /// The image data read out from the scanner
    pub rgb: Rgb16,
    /// The IR mask for dust removal, when the pass asked for one
    pub ir: Option<Luma16>,
}

/// Read sample `index` from a buffer of big-endian u16s
#[inline(always)]
pub fn be_u16_at(buf: &[u8], index: usize) -> u16 {
    u16::from_be_bytes([buf[2 * index], buf[2 * index + 1]])
}

/// The stream was not the length the decoder was expecting
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("received {got} bytes, expected {expected}")]
pub struct LengthMismatch {
    pub got: u64,
    pub expected: u64,
}

/// A decoder fed a scanner's byte stream as it arrives
///
/// Implementors take chunks in arrival order and unscramble as they go
pub trait StreamDecoder {
    /// Borrows the decoder's buffers, so a multi-hundred-MB image needn't be copied out
    type Output<'a>
    where
        Self: 'a;

    /// What this decoder rejects a stream with
    type Error;

    /// Total bytes the scanner will send for this pass
    fn expected_bytes(&self) -> u64;

    /// Feed the next chunk, in arrival order
    fn push(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;

    /// Complete the pass and borrow the result
    fn finish(&mut self) -> Result<Self::Output<'_>, Self::Error>;
}

/// A decoder that consumes the stream one fixed-size unit at a time
///
/// A unit might be a row of pixels or a CCD interleave block. [`Blocked`] wraps one of these
/// into a [`StreamDecoder`].
pub trait BlockSink {
    type Output<'a>
    where
        Self: 'a;

    type Error: From<LengthMismatch>;

    /// Bytes in one unit
    fn block_len(&self) -> usize;

    /// Units the pass will deliver
    fn blocks(&self) -> u64;

    /// Unscramble one whole unit into the output
    fn emit(&mut self, block: &[u8]);

    /// Complete the pass and borrow the result
    fn finish(&mut self) -> Result<Self::Output<'_>, Self::Error>;
}

/// Feeds a [`BlockSink`] from arbitrarily-chunked reads
pub struct Blocked<S> {
    sink: S,
    /// Holds a unit split across chunks
    staging: Vec<u8>,
    filled: usize,
    received: u64,
}

impl<S: BlockSink> Blocked<S> {
    /// Wrap a sink. Concrete decoders give this a `new` of their own.
    pub fn wrap(sink: S) -> Self {
        let staging = vec![0; sink.block_len()];
        Self {
            sink,
            staging,
            filled: 0,
            received: 0,
        }
    }

    /// The wrapped sink, for the geometry it was built with
    pub fn sink(&self) -> &S {
        &self.sink
    }

    fn mismatch(&self) -> LengthMismatch {
        LengthMismatch {
            got: self.received,
            expected: self.expected_bytes(),
        }
    }
}

impl<S: BlockSink> StreamDecoder for Blocked<S> {
    type Output<'a>
        = S::Output<'a>
    where
        Self: 'a;
    type Error = S::Error;

    fn expected_bytes(&self) -> u64 {
        self.sink.block_len() as u64 * self.sink.blocks()
    }

    fn push(&mut self, mut bytes: &[u8]) -> Result<(), Self::Error> {
        self.received += bytes.len() as u64;
        if self.received > self.expected_bytes() {
            return Err(self.mismatch().into());
        }

        let block_len = self.sink.block_len();

        // Top up a unit the previous chunk left half-finished
        if self.filled > 0 {
            let take = (block_len - self.filled).min(bytes.len());
            self.staging[self.filled..self.filled + take].copy_from_slice(&bytes[..take]);
            self.filled += take;
            bytes = &bytes[take..];
            if self.filled < block_len {
                // This chunk didn't finish the unit, and it's all we have
                return Ok(());
            }
            let block = std::mem::take(&mut self.staging);
            self.sink.emit(&block);
            self.staging = block;
            self.filled = 0;
        }

        // Whole units straight out of the caller's buffer, then stash the tail
        let mut blocks = bytes.chunks_exact(block_len);
        for block in &mut blocks {
            self.sink.emit(block);
        }
        let tail = blocks.remainder();
        self.staging[..tail.len()].copy_from_slice(tail);
        self.filled = tail.len();

        Ok(())
    }

    fn finish(&mut self) -> Result<Self::Output<'_>, Self::Error> {
        if self.received != self.expected_bytes() {
            return Err(self.mismatch().into());
        }
        self.sink.finish()
    }
}

/// A decoder that keeps the raw bytes, for dumping a stream to disk
pub struct Collect {
    expected: u64,
    bytes: Vec<u8>,
}

impl Collect {
    pub fn new(expected: u64) -> Self {
        Self {
            expected,
            bytes: Vec::with_capacity(expected as usize),
        }
    }
}

impl StreamDecoder for Collect {
    type Output<'a> = &'a [u8];
    type Error = LengthMismatch;

    fn expected_bytes(&self) -> u64 {
        self.expected
    }

    fn push(&mut self, bytes: &[u8]) -> Result<(), LengthMismatch> {
        self.bytes.extend_from_slice(bytes);
        if self.bytes.len() as u64 > self.expected {
            return Err(LengthMismatch {
                got: self.bytes.len() as u64,
                expected: self.expected,
            });
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<&[u8], LengthMismatch> {
        if self.bytes.len() as u64 != self.expected {
            return Err(LengthMismatch {
                got: self.bytes.len() as u64,
                expected: self.expected,
            });
        }
        Ok(&self.bytes)
    }
}
