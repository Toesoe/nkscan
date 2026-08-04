//! The vendor CDBs every Nikon Coolscan here shares
//!
//! A write stages a value in a vendor register and nothing takes effect until
//! [`VendorTrigger`] commits it. The framing below is common; what a given subcode's payload
//! *means* is not, so each driver keeps its own payload type and implements [`VendorRegister`]
//! to say how it encodes.

use crate::scsi::{Cdb, Command, CommandData, Error, fields::be_u24};

/// Which vendor register a read or write addresses
///
/// Not every unit answers to every one of them.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Subcode {
    /// Written to park the focus motor, read for its position
    Focus,
    AutoFocus,
    Lamp,
    Eject,
    /// Something we haven't characterized, for probing the firmware's other registers
    Other(u8),
}

impl Subcode {
    pub fn to_byte(self) -> u8 {
        match self {
            Subcode::Focus => 0xC1,
            Subcode::AutoFocus => 0xA0,
            Subcode::Lamp => 0x80,
            Subcode::Eject => 0xD0,
            Subcode::Other(code) => code,
        }
    }
}

/// A driver's payload type, which knows which register it addresses and how it serializes
///
/// Kept per driver because the encodings genuinely differ: field offsets and payload lengths
/// are not the same for the same subcode everywhere.
pub trait VendorRegister {
    fn subcode(&self) -> Subcode;
    fn to_bytes(&self) -> Vec<u8>;
}

/// The two vendor opcodes share a CDB layout, differing only in direction
pub fn vendor_cdb(opcode: u8, subcode: Subcode, length: u32) -> Cdb<10> {
    let [length_hi, length_mid, length_lo] = be_u24(length);
    Cdb([
        opcode,
        0x00,
        subcode.to_byte(),
        0x00,
        0x00,
        0x00,
        length_hi,
        length_mid,
        length_lo,
        0x00,
    ])
}

/// Nikon vendor WRITE(10), opcode 0xE0
#[derive(Debug, Clone)]
pub struct VendorWrite {
    subcode: Subcode,
    bytes: Vec<u8>,
}

impl VendorWrite {
    pub fn new<P: VendorRegister>(payload: P) -> Self {
        Self {
            subcode: payload.subcode(),
            bytes: payload.to_bytes(),
        }
    }

    /// The staged bytes, for tests that pin them against a capture
    pub fn payload(&self) -> &[u8] {
        &self.bytes
    }
}

impl Command for VendorWrite {
    type Response = ();
    type Cdb = Cdb<10>;

    fn cdb(&self) -> Self::Cdb {
        vendor_cdb(0xE0, self.subcode, self.bytes.len() as u32)
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::Write(&self.bytes)
    }

    fn parse_response(&self, _data: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}

/// Nikon vendor TRIGGER(6)
///
/// Commits whatever a preceding [`VendorWrite`] staged. Opcode only, no data phase.
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

    struct Nine;
    impl VendorRegister for Nine {
        fn subcode(&self) -> Subcode {
            Subcode::Eject
        }
        fn to_bytes(&self) -> Vec<u8> {
            vec![0u8; 9]
        }
    }

    /// The transfer length in the CDB is the payload's own, which is what lets one write serve
    /// payloads of different lengths
    #[test]
    fn the_cdb_carries_the_payload_length() {
        let write = VendorWrite::new(Nine);
        assert_eq!(
            write.cdb().0,
            [0xE0, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00]
        );
        assert_eq!(write.payload(), &[0u8; 9]);
    }

    #[test]
    fn trigger_is_a_bare_opcode() {
        assert_eq!(VendorTrigger.cdb().0, [0xC1, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }
}
