//! Nikon vendor-specific WRITE(10)/READ(10)
//!
//! A write stages a value in RAM; nothing takes effect until
//! [`VendorTrigger`](super::VendorTrigger) commits it.
//!
//! The CDB layout is shared with the other Coolscans, the subcodes are not: 0xA0
//! autofocuses here and preheats on the 9000.

use crate::scsi::{Cdb, Command, CommandData, Error, fields::be_u24};

/// Which vendor register a read or write addresses
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Subcode {
    /// Written to park the motor, read for the position
    Focus,
    AutoFocus,
    Lamp,
    Eject,
    /// Something we haven't characterized, for probing the firmware's other registers
    Other(u8),
}

impl Subcode {
    fn to_byte(self) -> u8 {
        match self {
            Subcode::Focus => 0xC1,
            Subcode::AutoFocus => 0xA0,
            Subcode::Lamp => 0x80,
            Subcode::Eject => 0xD0,
            Subcode::Other(code) => code,
        }
    }
}

/// What a [`VendorWrite`] stages
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VendorPayload {
    /// Focus setpoint, big-endian at bytes 1..5. Probed against hardware: written
    /// setpoints read back exactly through [`VendorRead::focus`], and the firmware takes
    /// this payload at either 5 or 9 bytes.
    Focus(u32),
    /// Autofocus at `(x, y)` in native pixels (SANE's `cs3_autofocus`)
    AutoFocus {
        x: u32,
        y: u32,
    },
    Lamp,
    Eject,
    /// Bytes off an uncharacterized subcode, left for the caller to make sense of
    Raw(Vec<u8>),
}

impl VendorPayload {
    fn subcode(&self) -> Subcode {
        match self {
            VendorPayload::Focus(_) => Subcode::Focus,
            VendorPayload::AutoFocus { .. } => Subcode::AutoFocus,
            VendorPayload::Lamp => Subcode::Lamp,
            VendorPayload::Eject => Subcode::Eject,
            // Nothing to write back, so the subcode has to come from the caller instead
            VendorPayload::Raw(_) => Subcode::Other(0x00),
        }
    }

    /// Payload lengths are per subcode, so this is a `Vec` rather than one array
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            VendorPayload::Focus(setpoint) => {
                let mut bytes = vec![0u8; 9];
                bytes[1..5].copy_from_slice(&setpoint.to_be_bytes());
                bytes
            }
            VendorPayload::AutoFocus { x, y } => {
                let mut bytes = vec![0u8; 9];
                bytes[1..5].copy_from_slice(&x.to_be_bytes());
                bytes[5..9].copy_from_slice(&y.to_be_bytes());
                bytes
            }
            VendorPayload::Lamp => Vec::new(),
            VendorPayload::Eject => vec![0u8; 13],
            VendorPayload::Raw(_) => Vec::new(),
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

pub struct VendorWrite {
    subcode: Subcode,
    bytes: Vec<u8>,
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
        vendor_cdb(0xE0, self.subcode, self.bytes.len() as u32)
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::Write(&self.bytes)
    }

    fn parse_response(&self, _data: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}

/// The firmware rejects a transfer length above 13 bytes here
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

    /// The focus position, 13 bytes (SANE's `cs3_read_focus`)
    pub fn focus() -> Self {
        Self::new(Subcode::Focus, 13)
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

    /// Focus sits at bytes 1..5, big-endian. Everything else comes back uninterpreted: the
    /// lamp, eject and autofocus registers have only ever been written.
    fn parse_response(&self, data: &[u8]) -> Result<VendorPayload, Error> {
        match self.subcode {
            Subcode::Focus => data
                .get(1..5)
                .map(|b| VendorPayload::Focus(u32::from_be_bytes([b[0], b[1], b[2], b[3]])))
                .ok_or(Error::InvalidResponse(
                    "vendor read shorter than its value field",
                )),
            _ => Ok(VendorPayload::Raw(data.to_vec())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autofocus_cdb_and_payload_match_the_capture() {
        // Focus at (0x07B4, 0x0BA3): `E0 00 A0 00 00 00 00 00 09 00`, then
        // 00 + x BE32 + y BE32
        let write = VendorWrite::new(VendorPayload::AutoFocus {
            x: 0x0000_07B4,
            y: 0x0000_0BA3,
        });
        assert_eq!(
            write.cdb().0,
            [0xE0, 0x00, 0xA0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00]
        );
        assert!(matches!(
            write.data(),
            CommandData::Write([0x00, 0x00, 0x00, 0x07, 0xB4, 0x00, 0x00, 0x0B, 0xA3])
        ));
    }

    #[test]
    fn eject_cdb_matches_the_capture() {
        // Subcode 0xD0 with 13 zero bytes. The load counterpart 0xD1 is rejected
        // 05/24 on this unit, so eject is the only working form.
        let write = VendorWrite::new(VendorPayload::Eject);
        assert_eq!(
            write.cdb().0,
            [0xE0, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0D, 0x00]
        );
        assert!(matches!(write.data(), CommandData::Write(p) if p == [0u8; 13]));
    }

    #[test]
    fn lamp_carries_no_payload() {
        let write = VendorWrite::new(VendorPayload::Lamp);
        let cdb = write.cdb().0;
        assert_eq!(cdb[2], 0x80);
        assert_eq!([cdb[6], cdb[7], cdb[8]], [0, 0, 0]);
        assert!(matches!(write.data(), CommandData::Write(p) if p.is_empty()));
    }

    #[test]
    fn focus_setpoint_matches_the_probed_layout() {
        // Probed on hardware: 320 written here reads back as 320
        let write = VendorWrite::new(VendorPayload::Focus(320));
        assert_eq!(
            write.cdb().0,
            [0xE0, 0x00, 0xC1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00]
        );
        assert!(matches!(
            write.data(),
            CommandData::Write([0x00, 0x00, 0x00, 0x01, 0x40, 0x00, 0x00, 0x00, 0x00])
        ));
    }

    #[test]
    fn parking_is_a_zero_setpoint() {
        let write = VendorWrite::new(VendorPayload::Focus(0));
        assert!(matches!(write.data(), CommandData::Write(p) if p == [0u8; 9]));
    }

    #[test]
    fn focus_read_asks_for_thirteen_bytes_and_decodes_the_position() {
        let read = VendorRead::focus();
        assert_eq!(
            read.cdb().0,
            [0xE1, 0x00, 0xC1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0D, 0x00]
        );
        assert!(matches!(read.data(), CommandData::Read(13)));

        let mut response = [0u8; 13];
        response[1..5].copy_from_slice(&228u32.to_be_bytes());
        assert_eq!(
            read.parse_response(&response).unwrap(),
            VendorPayload::Focus(228)
        );
    }

    #[test]
    fn focus_read_rejects_a_short_response() {
        assert!(VendorRead::focus().parse_response(&[0u8; 3]).is_err());
    }

    /// An uncharacterized register keeps its subcode and hands the bytes back untouched
    #[test]
    fn a_probe_read_is_uninterpreted() {
        let read = VendorRead::new(Subcode::Other(0x42), 11);
        assert_eq!(
            read.cdb().0,
            [0xE1, 0x00, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x00]
        );
        assert_eq!(
            read.parse_response(&[1, 2, 3]).unwrap(),
            VendorPayload::Raw(vec![1, 2, 3])
        );
    }
}
