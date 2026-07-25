//! Nikon vendor-specific WRITE(10)/READ(10)
//!
//! These writes send a value to RAM but don't actually take effect until the
//! vendor TRIGGER is applied.

use crate::scsi::{Cdb, Command, CommandData, Error, fields::be_u24};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Subcode {
    Focus,
    Preheat,
}

impl Subcode {
    fn to_byte(self) -> u8 {
        match self {
            Subcode::Focus => 0xC1,
            Subcode::Preheat => 0xA0,
        }
    }
}

pub enum VendorPayload {
    /// Focus motor target/position, arbitrary firmware units. Used as both
    /// the write payload (set) and the read response (get).
    Focus(u16),
    Preheat,
}

impl VendorPayload {
    fn subcode(&self) -> Subcode {
        match self {
            VendorPayload::Focus(_) => Subcode::Focus,
            VendorPayload::Preheat => Subcode::Preheat,
        }
    }

    fn to_bytes(&self) -> [u8; 9] {
        // TODO: Will this need to be dynamically-sized?
        let mut bytes = [0u8; 9];
        if let VendorPayload::Focus(x) = self {
            bytes[3..5].copy_from_slice(&x.to_be_bytes());
        }
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct VendorWrite {
    subcode: Subcode,
    bytes: [u8; 9],
}

impl VendorWrite {
    pub fn new(payload: VendorPayload) -> Self {
        Self {
            subcode: payload.subcode(),
            bytes: payload.to_bytes(),
        }
    }
}

/// The two vendor opcodes share a CDB layout, differing only in direction
fn vendor_cdb(opcode: u8, subcode: Subcode, length: u32) -> Cdb<10> {
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

#[derive(Debug, Copy, Clone)]
pub struct VendorRead {
    subcode: Subcode,
    transfer_length: u32,
}

impl VendorRead {
    pub fn new(subcode: Subcode, transfer_length: u32) -> Self {
        Self {
            subcode,
            transfer_length,
        }
    }
}

impl Command for VendorRead {
    type Response = VendorPayload;
    type Cdb = Cdb<10>;

    fn cdb(&self) -> Self::Cdb {
        vendor_cdb(0xE1, self.subcode, self.transfer_length)
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::Read(self.transfer_length as usize)
    }

    fn parse_response(&self, data: &[u8]) -> Result<Self::Response, Error> {
        if data.len() != self.transfer_length as usize {
            return Err(Error::InvalidResponse(
                "We didn't get all the bytes we expected",
            ));
        }
        Ok(match self.subcode {
            Subcode::Focus => VendorPayload::Focus(u16::from_be_bytes(
                data[3..5].try_into().expect("we checked the byte length"),
            )),
            // We've never needed to read this one back, so the payload stays uninterpreted
            Subcode::Preheat => VendorPayload::Preheat,
        })
    }
}
