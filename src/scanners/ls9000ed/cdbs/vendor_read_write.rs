//! Nikon vendor-specific WRITE(10)/READ(10).
//!
//! Not part of any public SCSI spec — the `0xC0`-`0xFF` opcode range is
//! reserved "Vendor Specific" by T10, and Nikon uses it for a proprietary
//! register read/write interface. Layout reverse-engineered from Nikon's
//! Windows driver + wire captures, not from documentation.
//!
//! These writes send a value to RAM but don't actually take effect until the
//! vendor TRIGGER is applied.

use crate::scsi::{Cdb, Command, CommandData, Error};

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

impl Command for VendorWrite {
    type Response = ();
    type Cdb = Cdb<10>;

    fn cdb(&self) -> Self::Cdb {
        let length_bytes = (self.bytes.len() as u32).to_be_bytes();
        Cdb([
            0xE0,
            0x00,
            self.subcode.to_byte(),
            0x00,
            0x00,
            0x00,
            length_bytes[1],
            length_bytes[2],
            length_bytes[3],
            0x00,
        ])
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::Write(&self.bytes)
    }

    fn decode(&self, _data: &[u8]) -> Result<(), Error> {
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
        let length_bytes = self.transfer_length.to_be_bytes();
        Cdb([
            0xE1,
            0x00,
            self.subcode.to_byte(),
            0x00,
            0x00,
            0x00,
            length_bytes[1],
            length_bytes[2],
            length_bytes[3],
            0x00,
        ])
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::Read(self.transfer_length as usize)
    }

    fn decode(&self, data: &[u8]) -> Result<Self::Response, Error> {
        if data.len() != self.transfer_length as usize {
            return Err(Error::InvalidResponse(
                "We didn't get all the bytes we expected",
            ));
        }
        Ok(match self.subcode {
            Subcode::Focus => VendorPayload::Focus(u16::from_be_bytes(
                data[3..5].try_into().expect("we checked the byte length"),
            )),
            Subcode::Preheat => todo!(),
        })
    }
}
