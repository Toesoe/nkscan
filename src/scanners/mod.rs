//! Scanners supported by this library

use crate::scsi;

pub mod ls9000ed;

/// Either half of a streamed read can fail: the transport, or the decoder consuming it
#[derive(Debug, thiserror::Error)]
pub enum ReadError<E> {
    #[error(transparent)]
    Scsi(#[from] scsi::Error),
    #[error(transparent)]
    Decode(E),
}
