//! RESERVE UNIT

use crate::scsi::{Cdb, Command, CommandData, Error};

#[derive(Debug, Default)]
/// RESERVE(6) - claim exclusive control
/// Default of zeros/None is standard plain, non-extent, non-3rd-party reservation
pub struct ReserveUnit {
    /// Logical unit number
    lun: u8,
    /// Optional third party device ID
    third_party: Option<u8>,
    /// Control byte
    control: u8,
}

impl ReserveUnit {
    pub fn new(lun: u8, third_party: Option<u8>, control: u8) -> Self {
        Self {
            lun,
            third_party,
            control,
        }
    }
}

impl Command for ReserveUnit {
    type Response = ();
    type Cdb = Cdb<6>;

    fn cdb(&self) -> Self::Cdb {
        Cdb([
            0x16, // opcode
            ((self.lun & 0b111) << 5)
                | ((self.third_party.is_some() as u8) << 4)
                | ((self.third_party.unwrap_or(0) & 0b111) << 1),
            0x00,
            0x00,
            0x00,
            self.control,
        ])
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::None
    }

    fn decode(&self, _data: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}
