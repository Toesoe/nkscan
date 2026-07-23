//! MODE SENSE has two variants, a 6 and 10 byte
//! It seems the scanners we've used only use the 6 byte form

use crate::scsi::{Cdb, Command, CommandData, Error};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Page control (PC) field: which set of mode parameter values to return
pub enum PageControl {
    Current,
    Changeable,
    Default,
    Saved,
}

impl PageControl {
    fn to_bits(self) -> u8 {
        match self {
            PageControl::Current => 0b00,
            PageControl::Changeable => 0b01,
            PageControl::Default => 0b10,
            PageControl::Saved => 0b11,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Which mode page(s) to return
pub enum PageCode {
    /// Return all mode pages implemented by the target (3Fh)
    AllPages,
    /// A specific page code: 00h vendor-specific, 01h-1Fh device-type-specific,
    /// 20h-3Eh vendor-specific (page format required)
    Page(u8),
}

impl PageCode {
    fn to_byte(self) -> u8 {
        match self {
            PageCode::AllPages => 0x3F,
            PageCode::Page(code) => code & 0x3F,
        }
    }
}

#[derive(Debug, Copy, Clone)]
/// MODE SENSE(6) command provides a means for a target to report medium,
/// logical unit, or peripheral device parameters to the initiator.
/// Initiators should issue MODE SENSE prior to each MODE SELECT to determine
/// supported pages, page lengths, and other parameters.
pub struct ModeSense {
    /// Logical unit number
    lun: u8,
    /// Disable block descriptors.
    /// True specifies that the target shall not return any block descriptors.
    dbd: bool,
    /// Page control
    pc: PageControl,
    /// Page code
    page_code: PageCode,
    /// Allocation length
    allocation_length: u8,
    /// Control byte
    control: u8,
}

impl ModeSense {
    pub fn new(
        lun: u8,
        dbd: bool,
        pc: PageControl,
        page_code: PageCode,
        allocation_length: u8,
        control: u8,
    ) -> Self {
        Self {
            lun,
            dbd,
            pc,
            page_code,
            allocation_length,
            control,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Mode parameter list header, common to MODE SENSE(6) and MODE SELECT(6)
pub struct ModeParameterHeader {
    /// Length in bytes of the mode data that follows, not including this byte.
    /// Reserved (should be 0) when used with MODE SELECT.
    pub mode_data_length: u8,
    pub medium_type: u8,
    pub device_specific: u8,
    /// Length in bytes of the block descriptor(s) that follow the header
    pub block_descriptor_length: u8,
}

impl ModeParameterHeader {
    pub fn to_bytes(self) -> [u8; 4] {
        [
            self.mode_data_length,
            self.medium_type,
            self.device_specific,
            self.block_descriptor_length,
        ]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Mode parameter block descriptor, common to MODE SENSE(6) and MODE
/// SELECT(6) (SPC-2 8.3.4)
pub struct BlockDescriptor {
    pub density_code: u8,
    /// 24-bit field. Zero means the descriptor doesn't specify a block count.
    pub number_of_blocks: u32,
    /// 24-bit field
    pub block_length: u32,
}

impl BlockDescriptor {
    pub fn to_bytes(self) -> [u8; 8] {
        [
            self.density_code,
            ((self.number_of_blocks & 0xFF0000) >> 16) as u8,
            ((self.number_of_blocks & 0x00FF00) >> 8) as u8,
            (self.number_of_blocks & 0x0000FF) as u8,
            0x00, // reserved
            ((self.block_length & 0xFF0000) >> 16) as u8,
            ((self.block_length & 0x00FF00) >> 8) as u8,
            (self.block_length & 0x0000FF) as u8,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeSenseResponse {
    pub header: ModeParameterHeader,
    /// Block descriptor(s) followed by mode page(s), left undecoded since
    /// their contents are page- and device-specific
    pub data: Vec<u8>,
}

impl Command for ModeSense {
    type Response = ModeSenseResponse;
    type Cdb = Cdb<6>;

    fn cdb(&self) -> Self::Cdb {
        Cdb([
            0x1A, // opcode
            ((self.lun & 0b111) << 5) | ((self.dbd as u8) << 3),
            (self.pc.to_bits() << 6) | self.page_code.to_byte(),
            0x00, // reserved
            self.allocation_length,
            self.control,
        ])
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::Read(self.allocation_length as usize)
    }

    fn decode(&self, data: &[u8]) -> Result<ModeSenseResponse, Error> {
        if data.len() < 4 {
            return Err(Error::InvalidResponse(
                "MODE SENSE(6) response shorter than the 4-byte header",
            ));
        }

        Ok(ModeSenseResponse {
            header: ModeParameterHeader {
                mode_data_length: data[0],
                medium_type: data[1],
                device_specific: data[2],
                block_descriptor_length: data[3],
            },
            data: data[4..].to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cdb_encodes_opcode_lun_and_dbd() {
        let mode_sense = ModeSense::new(3, true, PageControl::Current, PageCode::AllPages, 0, 0);
        let cdb = mode_sense.cdb().0;
        assert_eq!(cdb[0], 0x1A);
        assert_eq!(cdb[1], (3 << 5) | (1 << 3));
    }

    #[test]
    fn cdb_encodes_pc_and_page_code() {
        let mode_sense = ModeSense::new(
            0,
            false,
            PageControl::Changeable,
            PageCode::Page(0x02),
            0,
            0,
        );
        assert_eq!(mode_sense.cdb().0[2], (0b01 << 6) | 0x02);
    }

    #[test]
    fn cdb_encodes_all_pages_page_code() {
        let mode_sense = ModeSense::new(0, false, PageControl::Default, PageCode::AllPages, 0, 0);
        assert_eq!(mode_sense.cdb().0[2], (0b10 << 6) | 0x3F);
    }

    #[test]
    fn cdb_encodes_allocation_length_and_control() {
        let mode_sense = ModeSense::new(0, false, PageControl::Saved, PageCode::Page(0), 96, 0x80);
        let cdb = mode_sense.cdb().0;
        assert_eq!(cdb[3], 0x00);
        assert_eq!(cdb[4], 96);
        assert_eq!(cdb[5], 0x80);
    }

    #[test]
    fn data_is_read_with_allocation_length() {
        let mode_sense = ModeSense::new(0, false, PageControl::Current, PageCode::AllPages, 64, 0);
        assert!(matches!(mode_sense.data(), CommandData::Read(64)));
    }

    #[test]
    fn decode_parses_header_and_leaves_remainder_raw() {
        let mode_sense = ModeSense::new(0, false, PageControl::Current, PageCode::AllPages, 0, 0);
        let data = [0x0A, 0x00, 0x00, 0x08, 0xAA, 0xBB, 0xCC];
        let response = mode_sense.decode(&data).unwrap();
        assert_eq!(
            response.header,
            ModeParameterHeader {
                mode_data_length: 0x0A,
                medium_type: 0x00,
                device_specific: 0x00,
                block_descriptor_length: 0x08,
            }
        );
        assert_eq!(response.data, vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn decode_rejects_short_response() {
        let mode_sense = ModeSense::new(0, false, PageControl::Current, PageCode::AllPages, 0, 0);
        let data = [0u8; 3];
        let err = mode_sense.decode(&data).unwrap_err();
        assert!(matches!(err, Error::InvalidResponse(_)));
    }

    #[test]
    fn header_to_bytes_matches_field_order() {
        let header = ModeParameterHeader {
            mode_data_length: 0x0A,
            medium_type: 0x01,
            device_specific: 0x02,
            block_descriptor_length: 0x08,
        };
        assert_eq!(header.to_bytes(), [0x0A, 0x01, 0x02, 0x08]);
    }

    #[test]
    fn block_descriptor_to_bytes_encodes_24_bit_fields_big_endian() {
        let descriptor = BlockDescriptor {
            density_code: 0x00,
            number_of_blocks: 0x00,
            block_length: 0x01,
        };
        assert_eq!(
            descriptor.to_bytes(),
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]
        );
    }
}
