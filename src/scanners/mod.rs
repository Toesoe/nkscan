//! Scanners supported by this library

use crate::{decode::StreamDecoder, scsi};

pub mod ls50ed;
pub mod ls9000ed;

/// Either half of a streamed read can fail: the transport, or the decoder consuming it
#[derive(Debug, thiserror::Error)]
pub enum ReadError<E> {
    #[error(transparent)]
    Scsi(#[from] scsi::Error),
    #[error(transparent)]
    Decode(E),
}

/// What every scanner can do, whatever it is on the other end of the transport
pub trait Scanner {
    /// How this scanner reports readiness
    type Status;

    /// Vendor, product and revision
    fn identify(&mut self) -> Result<scsi::cdbs::InquiryResponse, scsi::Error>;

    /// Current readiness, with transient not-ready states folded in rather than raised
    fn status(&mut self) -> Result<Self::Status, scsi::Error>;

    /// Take exclusive access
    fn reserve(&mut self) -> Result<(), scsi::Error>;

    /// Give exclusive access back
    fn release(&mut self) -> Result<(), scsi::Error>;

    /// Pull the next slice of the pending pass. An empty return means the scanner stopped early.
    fn read_chunk(&mut self, want: u32) -> Result<Vec<u8>, scsi::Error>;

    /// Stream a pass into a decoder, `chunk` bytes at a time
    ///
    /// The decoder says how much to read, so the geometry lives in one place
    fn read_into<D>(&mut self, decoder: &mut D, chunk: u32) -> Result<(), ReadError<D::Error>>
    where
        D: StreamDecoder,
        Self: Sized,
    {
        self.read_into_with(decoder, chunk, |_, _| {})
    }

    /// [`read_into`](Self::read_into), calling `progress` with (received, expected) per chunk
    fn read_into_with<D, F>(
        &mut self,
        decoder: &mut D,
        chunk: u32,
        mut progress: F,
    ) -> Result<(), ReadError<D::Error>>
    where
        D: StreamDecoder,
        F: FnMut(u64, u64),
        Self: Sized,
    {
        let expected = decoder.expected_bytes();
        let mut received = 0u64;

        while received < expected {
            let want = u64::from(chunk).min(expected - received) as u32;
            let bytes = self.read_chunk(want)?;
            if bytes.is_empty() {
                return Err(scsi::Error::InvalidResponse(
                    "image read returned nothing before the expected length",
                )
                .into());
            }
            received += bytes.len() as u64;
            decoder.push(&bytes).map_err(ReadError::Decode)?;
            progress(received, expected);
        }
        Ok(())
    }
}

/// A scanner with removable film holders
pub trait FilmHolder {
    /// What holders this scanner recognizes
    type Holder;

    /// Which holder, if any, is currently loaded
    fn holder(&mut self) -> Result<Self::Holder, scsi::Error>;
}

/// A scanner with a movable focus mechanism
pub trait Focus {
    /// The focus value currently staged in firmware
    fn focus(&mut self) -> Result<u16, scsi::Error>;

    /// Stage a focus target and commit it
    fn set_focus(&mut self, focus: u16) -> Result<(), scsi::Error>;
}
