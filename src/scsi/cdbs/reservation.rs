//! RESERVE UNIT(6) and RELEASE UNIT(6)
//!
//! Mirror images of each other: identical CDB layout, opposite effect.

use crate::scsi::{Cdb, Command, CommandData, Error, fields::lun_byte};

/// Both commands lay byte 1 out the same way: LUN, a third-party flag, and the device id
fn reservation_cdb(opcode: u8, lun: u8, third_party: Option<u8>, control: u8) -> Cdb<6> {
    Cdb([
        opcode,
        lun_byte(lun)
            | ((third_party.is_some() as u8) << 4)
            | ((third_party.unwrap_or(0) & 0b111) << 1),
        0x00,
        0x00,
        0x00,
        control,
    ])
}

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
        reservation_cdb(0x16, self.lun, self.third_party, self.control)
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::None
    }

    fn parse_response(&self, _data: &[u8]) -> Result<Self::Response, Error> {
        Ok(())
    }
}

#[derive(Debug, Default)]
/// RELEASE(6) - release previously reserved exclusive control
/// Default of zeros/None is standard plain, non-extent, non-3rd-party reservation
pub struct ReleaseUnit {
    /// Logical unit number
    lun: u8,
    /// Optional third party device ID
    third_party: Option<u8>,
    /// Control byte
    control: u8,
}

impl ReleaseUnit {
    pub fn new(lun: u8, third_party: Option<u8>, control: u8) -> Self {
        Self {
            lun,
            third_party,
            control,
        }
    }
}

impl Command for ReleaseUnit {
    type Response = ();
    type Cdb = Cdb<6>;

    fn cdb(&self) -> Self::Cdb {
        reservation_cdb(0x17, self.lun, self.third_party, self.control)
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::None
    }

    fn parse_response(&self, _data: &[u8]) -> Result<Self::Response, Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcodes_are_the_only_difference() {
        let reserve = ReserveUnit::new(2, Some(5), 0x80).cdb().0;
        let release = ReleaseUnit::new(2, Some(5), 0x80).cdb().0;
        assert_eq!(reserve[0], 0x16);
        assert_eq!(release[0], 0x17);
        assert_eq!(reserve[1..], release[1..]);
    }

    #[test]
    fn plain_reservation_is_all_zeros_past_the_opcode() {
        assert_eq!(
            ReserveUnit::default().cdb().0,
            [0x16, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn third_party_sets_the_flag_and_device_id() {
        // bit 4 is the third-party flag, bits 3-1 the device id
        assert_eq!(ReserveUnit::new(0, Some(3), 0).cdb().0[1], 0b0001_0110);
    }

    #[test]
    fn no_third_party_leaves_both_fields_clear() {
        assert_eq!(ReserveUnit::new(1, None, 0).cdb().0[1], 0x20);
    }
}
