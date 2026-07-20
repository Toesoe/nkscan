//! Scanners supported by this library
use crate::scsi::{Error as ScsiError, Transport, cdbs::*};

// TODO: Scanner traits that will be implemented by the various devices

/// The LS-9000 ED
pub struct Ls9k<T> {
    transport: T,
}

/// The coolscan 9000 is SCSI-only, so we can gate here on scsi backends
impl<T> Ls9k<T>
where
    T: Transport,
{
    pub fn new(transport: T) -> Self {
        Ls9k { transport }
    }

    // TODO: Remove
    pub fn inquiry(&mut self) -> Result<InquiryResponse, ScsiError> {
        self.transport.send(&Inquiry::new())
    }
}
