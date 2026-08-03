//! What the USB bodies stage in the shared vendor registers
//!
//! The CDB framing, the subcode values and [`VendorWrite`] itself live in
//! [`nikon::cdbs`](crate::scanners::nikon::cdbs). What lives here is how a payload is encoded
//! and a read decoded, which the LS-50 and the LS-5000 do identically apart from the focus read
//! length. That one is a parameter rather than a second copy of this file.

use crate::{
    scanners::nikon::cdbs::{Subcode, VendorRegister, vendor_cdb},
    scsi::{Cdb, Command, CommandData, Error},
};

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
    ExtendedConfig(Vec<u8>),
    /// Bytes off an uncharacterized subcode, left for the caller to make sense of
    Raw(Vec<u8>),
}

impl VendorRegister for VendorPayload {
    fn subcode(&self) -> Subcode {
        match self {
            VendorPayload::Focus(_) => Subcode::Focus,
            VendorPayload::AutoFocus { .. } => Subcode::AutoFocus,
            VendorPayload::Lamp => Subcode::Lamp,
            VendorPayload::Eject => Subcode::Eject,
            VendorPayload::ExtendedConfig(_) => Subcode::ExtendedConfig,
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
            VendorPayload::ExtendedConfig(_) => vec![0u8; 9],
            VendorPayload::Raw(_) => Vec::new(),
        }
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

    /// The focus position
    ///
    /// `length` is the one thing the two USB bodies disagree about: the LS-50 asks for 13 bytes,
    /// saying the firmware rejects a longer transfer, and the LS-5000 asks for 9, saying 9 is
    /// the payload length rather than a truncation. Nobody has read one at both lengths, so it
    /// is the caller's constant rather than a choice made here.
    /// See docs/OPEN_QUESTIONS.md section 18.
    pub fn focus(length: u32) -> Self {
        Self::new(Subcode::Focus, length)
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
    use crate::scanners::nikon::cdbs::VendorWrite;

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
        let read = VendorRead::focus(13);
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
        assert!(VendorRead::focus(13).parse_response(&[0u8; 3]).is_err());
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
