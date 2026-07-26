//! Nikon vendor TRIGGER(6)
//!
//! Commits whatever a preceding [`VendorWrite`](super::VendorWrite) staged. Opcode only,
//! no data phase.

use crate::scsi::{Cdb, Command, CommandData, Error};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cdb_is_opcode_only() {
        assert_eq!(VendorTrigger.cdb().0, [0xC1, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert!(matches!(VendorTrigger.data(), CommandData::None));
    }
}
