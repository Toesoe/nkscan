//! Nikon vendor TRIGGER(6)

use crate::scsi::{Cdb, Command, CommandData, Error};

/// Nikon vendor TRIGGER(6)
///
/// Commits whatever was last staged by a preceding [`VendorWrite`].
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

    fn decode(&self, _data: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}
