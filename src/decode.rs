//! A trait to encode the behavior of a streaming image decoder

/// A decoder fed a scanner's byte stream as it arrives
///
/// Implementors take chunks in arrival order and unscramble as they go.
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
