//! READ(10) and SEND(10) for SCSI-2 scanner devices, from 15.2.4 / 15.2.8
//!
//! Mirror images of each other: same DTC/DTQ addressing, opposite data direction.

use crate::scsi::{
    Cdb, Command, CommandData, Error,
    fields::{be_u24, lun_byte},
};

/// SCSI-2 scanner READ(10)
///
/// DTC/DTQ key what kind of read operation this really is and is vendor-specific
pub struct Read {
    /// Logical unit number (3 bits)
    lun: u8,
    /// Data-type code
    dtc: DataTypeCode,
    /// Data-type qualifier
    dtq: u16,
    /// Transfer length in bytes (really u24)
    transfer_length: u32,
    /// Control byte. Bits 7-6 are vendor-specific.
    control: u8,
}

impl Read {
    pub fn new(lun: u8, dtc: DataTypeCode, dtq: u16, transfer_length: u32, control: u8) -> Self {
        Read {
            lun,
            dtc,
            dtq,
            transfer_length,
            control,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DataTypeCode {
    Image,
    HalftoneMask,
    GammaFunction,
    Vendor(u8),
}

impl DataTypeCode {
    fn to_byte(self) -> u8 {
        match self {
            DataTypeCode::Image => 0x00,
            DataTypeCode::HalftoneMask => 0x02,
            DataTypeCode::GammaFunction => 0x03,
            DataTypeCode::Vendor(x) => x,
        }
    }
}

impl Command for Read {
    type Response = Vec<u8>;
    type Cdb = Cdb<10>;

    fn cdb(&self) -> Self::Cdb {
        let [dtq_hi, dtq_lo] = self.dtq.to_be_bytes();
        let [length_hi, length_mid, length_lo] = be_u24(self.transfer_length);
        Cdb([
            0x28, // opcode
            lun_byte(self.lun),
            self.dtc.to_byte(),
            0x00, // reserved
            dtq_hi,
            dtq_lo,
            length_hi,
            length_mid,
            length_lo,
            self.control,
        ])
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::Read(self.transfer_length as usize)
    }

    fn parse_response(&self, data: &[u8]) -> Result<Self::Response, Error> {
        Ok(data.to_vec())
    }
}

/// SCSI-2 scanner SEND(10), the data-out counterpart to [`Read`]
pub struct Send {
    /// Logical unit number (3 bits)
    lun: u8,
    /// Data-type code
    dtc: DataTypeCode,
    /// Data-type qualifier
    dtq: u16,
    /// The parameter list to send
    parameters: Vec<u8>,
    /// Control byte. Bits 7-6 are vendor-specific.
    control: u8,
}

impl Send {
    pub fn new(lun: u8, dtc: DataTypeCode, dtq: u16, parameters: Vec<u8>, control: u8) -> Self {
        Send {
            lun,
            dtc,
            dtq,
            parameters,
            control,
        }
    }
}

impl Command for Send {
    type Response = ();
    type Cdb = Cdb<10>;

    fn cdb(&self) -> Self::Cdb {
        let [dtq_hi, dtq_lo] = self.dtq.to_be_bytes();
        let [length_hi, length_mid, length_lo] = be_u24(self.parameters.len() as u32);
        Cdb([
            0x2A, // opcode
            lun_byte(self.lun),
            self.dtc.to_byte(),
            0x00, // reserved
            dtq_hi,
            dtq_lo,
            length_hi,
            length_mid,
            length_lo,
            self.control,
        ])
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::Write(&self.parameters)
    }

    fn parse_response(&self, _data: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
mod send_tests {
    use super::*;

    #[test]
    fn cdb_matches_real_capture() {
        // LS-9000ED frame-boundary write: `2A 00 88 00 00 03 00 00 44 00`, 68-byte payload
        let send = Send::new(0, DataTypeCode::Vendor(0x88), 0x0003, vec![0u8; 68], 0x00);
        assert_eq!(
            send.cdb().0,
            [0x2A, 0x00, 0x88, 0x00, 0x00, 0x03, 0x00, 0x00, 0x44, 0x00]
        );
    }

    #[test]
    fn cdb_encodes_parameter_list_length_as_big_endian_u24() {
        let send = Send::new(0, DataTypeCode::Image, 0, vec![0u8; 0x01_2345], 0x00);
        let cdb = send.cdb().0;
        assert_eq!([cdb[6], cdb[7], cdb[8]], [0x01, 0x23, 0x45]);
    }

    #[test]
    fn cdb_encodes_lun_dtc_and_dtq() {
        let send = Send::new(2, DataTypeCode::GammaFunction, 0x0103, vec![], 0x80);
        let cdb = send.cdb().0;
        assert_eq!(cdb[1], 2 << 5);
        assert_eq!(cdb[2], 0x03);
        assert_eq!([cdb[4], cdb[5]], [0x01, 0x03]);
        assert_eq!(cdb[9], 0x80);
    }

    #[test]
    fn data_is_write_with_parameters() {
        let send = Send::new(0, DataTypeCode::Image, 0, vec![1, 2, 3], 0);
        assert!(matches!(send.data(), CommandData::Write(p) if p == [1, 2, 3]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cdb_encodes_opcode_lun_and_dtc() {
        let read = Read::new(2, DataTypeCode::HalftoneMask, 0, 0, 0x00);
        let cdb = read.cdb().0;
        assert_eq!(cdb[0], 0x28);
        assert_eq!(cdb[1], 2 << 5);
        assert_eq!(cdb[2], 0x02);
        assert_eq!(cdb[3], 0x00);
    }

    /// Byte 1 is nothing but the LUN here, so an over-large one used to shift straight
    /// through it and corrupt the reserved bits
    #[test]
    fn out_of_range_lun_is_masked() {
        assert_eq!(Read::new(9, DataTypeCode::Image, 0, 0, 0).cdb().0[1], 0x20);
        assert_eq!(
            Send::new(9, DataTypeCode::Image, 0, vec![], 0).cdb().0[1],
            0x20
        );
    }

    #[test]
    fn cdb_encodes_dtq_big_endian() {
        let read = Read::new(0, DataTypeCode::Image, 0x1234, 0, 0x00);
        let cdb = read.cdb().0;
        assert_eq!(cdb[4], 0x12);
        assert_eq!(cdb[5], 0x34);
    }

    #[test]
    fn cdb_encodes_transfer_length_as_big_endian_u24() {
        let read = Read::new(0, DataTypeCode::Image, 0, 0x01_2345, 0x00);
        let cdb = read.cdb().0;
        assert_eq!(cdb[6], 0x01);
        assert_eq!(cdb[7], 0x23);
        assert_eq!(cdb[8], 0x45);
    }

    #[test]
    fn cdb_encodes_control_byte_verbatim() {
        let read = Read::new(0, DataTypeCode::Image, 0, 0, 0x80);
        assert_eq!(read.cdb().0[9], 0x80);
    }

    #[test]
    fn cdb_matches_real_capture() {
        // Real capture of the LS-9000 driver's own image READ: lun=0,
        // dtc=Image, dtq=0x0000, transfer_length=0x0C471C (804636 bytes),
        // control=0x80 (vendor-specific bit set, not spec's 0x00).
        let read = Read::new(0, DataTypeCode::Image, 0x0000, 804_636, 0x80);
        assert_eq!(
            read.cdb().0,
            [0x28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x47, 0x1C, 0x80]
        );
    }

    #[test]
    fn data_is_read_with_transfer_length() {
        let read = Read::new(0, DataTypeCode::Image, 0, 512, 0x00);
        assert!(matches!(read.data(), CommandData::Read(512)));
    }

    #[test]
    fn decode_returns_owned_copy_of_data() {
        let data = [0xAA, 0xBB, 0xCC];
        let response = Read::new(0, DataTypeCode::Image, 0, 3, 0x00)
            .parse_response(&data)
            .unwrap();
        assert_eq!(response, vec![0xAA, 0xBB, 0xCC]);
    }
}
