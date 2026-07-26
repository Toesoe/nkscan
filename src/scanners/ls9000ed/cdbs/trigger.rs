//! Nikon vendor TRIGGER(6) and ABORT(6)
//!
//! A pair of parameterless vendor commands. TRIGGER commits whatever a vendor write staged,
//! ABORT throws away a pass in progress.

use crate::scsi::{Cdb, Command, CommandData, Error};

/// Nikon vendor TRIGGER(6)
#[derive(Debug, Default, Copy, Clone)]
pub struct VendorTrigger;

impl Command for VendorTrigger {
    type Response = ();
    type Cdb = Cdb<6>;

    fn cdb(&self) -> Self::Cdb {
        Cdb([0xC1, 0x00, 0x00, 0x00, 0x00, 0x00])
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::None
    }

    fn parse_response(&self, _data: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}

/// Nikon vendor ABORT(6)
#[derive(Debug, Default, Copy, Clone)]
pub struct VendorAbort;

impl Command for VendorAbort {
    type Response = ();
    type Cdb = Cdb<6>;

    fn cdb(&self) -> Self::Cdb {
        Cdb([0xC0, 0x00, 0x00, 0x00, 0x00, 0x00])
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::None
    }

    fn parse_response(&self, _data: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}
