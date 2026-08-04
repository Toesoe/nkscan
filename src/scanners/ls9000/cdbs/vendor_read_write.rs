//! What the LS-9000 stages in the shared vendor registers
//!
//! The CDB framing, the subcode values and [`VendorWrite`] itself live in
//! [`nikon::cdbs`](crate::scanners::nikon::cdbs). What lives here is how this model encodes a
//! payload and decodes a read: focus is a two-byte field at offset 3.

use crate::{
    scanners::nikon::cdbs::{Subcode, VendorRegister, vendor_cdb},
    scsi::{Cdb, Command, CommandData, Error},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VendorPayload {
    /// Focus motor target/position, arbitrary firmware units. Used as both
    /// the write payload (set) and the read response (get).
    Focus(u16),
    /// Where on the film to focus, in 1/4000-in dots. Nikon Scan always aims this at the
    /// center of the frame it is about to scan.
    AutoFocus { x: u32, y: u32 },
    /// Send the film holder back out. Carries no parameters: Nikon Scan leaves the nine bytes
    /// as whatever was in the buffer, and they differ every time, so nothing reads them.
    Eject,
    /// Bytes off an uncharacterized subcode, left for the caller to make sense of
    Raw(Vec<u8>),
}

impl VendorRegister for VendorPayload {
    fn subcode(&self) -> Subcode {
        match self {
            VendorPayload::Focus(_) => Subcode::Focus,
            VendorPayload::AutoFocus { .. } => Subcode::AutoFocus,
            VendorPayload::Eject => Subcode::Eject,
            // Nothing to write back, so the subcode has to come from the caller instead
            VendorPayload::Raw(_) => Subcode::Other(0x00),
        }
    }

    /// Always nine bytes on this model, whatever the subcode
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0u8; 9];
        match *self {
            VendorPayload::Focus(focus) => bytes[3..5].copy_from_slice(&focus.to_be_bytes()),
            VendorPayload::AutoFocus { x, y } => {
                // X runs along the sensor bar, so it never needs more than two bytes
                debug_assert!(x <= u32::from(u16::MAX));
                bytes[3..5].copy_from_slice(&(x as u16).to_be_bytes());
                bytes[5..9].copy_from_slice(&y.to_be_bytes());
            }
            VendorPayload::Eject | VendorPayload::Raw(_) => {}
        }
        bytes
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
        let short = |at: usize| u16::from_be_bytes(data[at..at + 2].try_into().expect("checked"));
        let long = |at: usize| u32::from_be_bytes(data[at..at + 4].try_into().expect("checked"));
        Ok(match self.subcode {
            Subcode::Focus => VendorPayload::Focus(short(3)),
            Subcode::AutoFocus => VendorPayload::AutoFocus {
                x: u32::from(short(3)),
                y: long(5),
            },
            // Write-only here, or not a register this model has, so a read is just bytes
            Subcode::Eject | Subcode::Lamp | Subcode::Other(_) => {
                VendorPayload::Raw(data.to_vec())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanners::nikon::cdbs::VendorWrite;

    /// The autofocus point Nikon Scan wrote before scanning frame 2 of a 6x9 strip:
    /// the sensor center, and the middle of that frame's boundary rectangle
    #[test]
    fn autofocus_matches_a_captured_point() {
        let write = VendorWrite::new(VendorPayload::AutoFocus { x: 5000, y: 25260 });
        assert_eq!(
            write.cdb().0,
            [0xE0, 0x00, 0xA0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00]
        );
        assert_eq!(
            write.payload(),
            [0x00, 0x00, 0x00, 0x13, 0x88, 0x00, 0x00, 0x62, 0xAC]
        );
    }

    /// Focus leaves the four trailing bytes clear, which is what every capture shows
    #[test]
    fn focus_only_fills_the_short_field() {
        let write = VendorWrite::new(VendorPayload::Focus(0x00B2));
        assert_eq!(write.cdb().0[2], 0xC1);
        assert_eq!(
            write.payload(),
            [0x00, 0x00, 0x00, 0x00, 0xB2, 0x00, 0x00, 0x00, 0x00]
        );
    }

    /// Nikon Scan sends nine bytes of whatever was in the buffer, different every session, so
    /// only the subcode carries meaning. Zeros are the honest version of that.
    #[test]
    fn eject_is_a_bare_subcode() {
        let write = VendorWrite::new(VendorPayload::Eject);
        assert_eq!(
            write.cdb().0,
            [0xE0, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00]
        );
        assert_eq!(write.payload(), [0x00; 9]);
    }

    #[test]
    fn payloads_round_trip_through_the_wire() {
        for payload in [
            VendorPayload::Focus(0x00C3),
            VendorPayload::AutoFocus { x: 5000, y: 29508 },
        ] {
            let bytes = payload.to_bytes();
            let read = VendorRead::new(payload.subcode(), bytes.len() as u32);
            assert_eq!(read.parse_response(&bytes).unwrap(), payload);
        }
    }
}
