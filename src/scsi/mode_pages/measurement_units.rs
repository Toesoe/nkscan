//! Measurement Units mode page, SCSI-2 15.3.3.1, mode page 03h.
//!
//! Reported via MODE SENSE(6); tells the initiator what unit and divisor a
//! scanner device's dimension-bearing commands (e.g. the scan window) are in.

use crate::scsi::cdbs::ModeSenseResponse;

/// Basic measurement unit, mode page 03h byte 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasicUnit {
    Inches,
    Millimetres,
    Points,
    /// A code byte the spec reserves and we haven't seen used.
    Unknown(u8),
}

impl BasicUnit {
    fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => BasicUnit::Inches,
            0x01 => BasicUnit::Millimetres,
            0x02 => BasicUnit::Points,
            other => BasicUnit::Unknown(other),
        }
    }

    fn to_byte(self) -> u8 {
        match self {
            BasicUnit::Inches => 0x00,
            BasicUnit::Millimetres => 0x01,
            BasicUnit::Points => 0x02,
            BasicUnit::Unknown(other) => other,
        }
    }
}

/// Decoded contents of the Measurement Units mode page (page code 0x03).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasurementUnits {
    pub basic_unit: BasicUnit,
    /// Number of units needed to equal one basic measurement unit
    /// (default 1200, i.e. 1/1200 in). Zero is a device error condition.
    pub divisor: u16,
}

impl MeasurementUnits {
    pub const PAGE_CODE: u8 = 0x03;
    /// 4-byte mode parameter header + this page's fixed 8-byte body, with no
    /// block descriptors (DBD set) getting in between.
    pub const ALLOCATION_LENGTH: u8 = 12;

    /// Decode a MODE SENSE(6) response for page 0x03. Returns `None` if the
    /// page data is missing, too short, or doesn't start with page code 0x03
    /// - the caller should treat that as a real error, not a units state.
    pub fn from_response(response: &ModeSenseResponse) -> Option<Self> {
        let page = response
            .data
            .get(response.header.block_descriptor_length as usize..)?;
        if page.len() < 8 || page[0] & 0x3F != Self::PAGE_CODE {
            return None;
        }
        Some(Self {
            basic_unit: BasicUnit::from_byte(page[2]),
            divisor: u16::from_be_bytes([page[4], page[5]]),
        })
    }

    /// Encode this page's fixed 8-byte body (page code/PS, length, basic
    /// unit, divisor), for use in a MODE SELECT parameter list.
    pub fn page_bytes(&self) -> [u8; 8] {
        let [divisor_msb, divisor_lsb] = self.divisor.to_be_bytes();
        [
            Self::PAGE_CODE,
            0x06,
            self.basic_unit.to_byte(),
            0x00,
            divisor_msb,
            divisor_lsb,
            0x00,
            0x00,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scsi::cdbs::ModeParameterHeader;

    fn response(block_descriptor_length: u8, data: Vec<u8>) -> ModeSenseResponse {
        ModeSenseResponse {
            header: ModeParameterHeader {
                mode_data_length: 0,
                medium_type: 0,
                device_specific: 0,
                block_descriptor_length,
            },
            data,
        }
    }

    #[test]
    fn decodes_inches_at_default_divisor() {
        // page code 0x03, length 0x06, unit=inches, reserved, divisor=1200, reserved
        let data = vec![0x03, 0x06, 0x00, 0x00, 0x04, 0xB0, 0x00, 0x00];
        let units = MeasurementUnits::from_response(&response(0, data)).unwrap();
        assert_eq!(units.basic_unit, BasicUnit::Inches);
        assert_eq!(units.divisor, 1200);
    }

    #[test]
    fn decodes_millimetres() {
        let data = vec![0x03, 0x06, 0x01, 0x00, 0x00, 0x0A, 0x00, 0x00];
        let units = MeasurementUnits::from_response(&response(0, data)).unwrap();
        assert_eq!(units.basic_unit, BasicUnit::Millimetres);
        assert_eq!(units.divisor, 10);
    }

    #[test]
    fn unrecognized_unit_byte_is_preserved() {
        let data = vec![0x03, 0x06, 0x7F, 0x00, 0x00, 0x01, 0x00, 0x00];
        let units = MeasurementUnits::from_response(&response(0, data)).unwrap();
        assert_eq!(units.basic_unit, BasicUnit::Unknown(0x7F));
    }

    #[test]
    fn skips_block_descriptor_before_page_data() {
        let mut data = vec![0xAA; 8]; // stand-in block descriptor
        data.extend_from_slice(&[0x03, 0x06, 0x00, 0x00, 0x04, 0xB0, 0x00, 0x00]);
        let units = MeasurementUnits::from_response(&response(8, data)).unwrap();
        assert_eq!(units.basic_unit, BasicUnit::Inches);
        assert_eq!(units.divisor, 1200);
    }

    #[test]
    fn wrong_page_code_is_not_measurement_units_data() {
        let data = vec![0x02, 0x06, 0x00, 0x00, 0x04, 0xB0, 0x00, 0x00];
        assert_eq!(MeasurementUnits::from_response(&response(0, data)), None);
    }

    #[test]
    fn page_shorter_than_fixed_body_is_not_decodable() {
        let data = vec![0x03, 0x06, 0x00, 0x00, 0x04, 0xB0];
        assert_eq!(MeasurementUnits::from_response(&response(0, data)), None);
    }

    #[test]
    fn page_bytes_matches_real_capture() {
        // Nikon Scan's own MODE SELECT page 0x03 payload for the LS-9000ED
        // (Windows capture seq 724, see RE_FINDINGS.md): inches, divisor
        // 0x0FA0 = 4000 (the scanner's native DPI).
        let units = MeasurementUnits {
            basic_unit: BasicUnit::Inches,
            divisor: 4000,
        };
        assert_eq!(
            units.page_bytes(),
            [0x03, 0x06, 0x00, 0x00, 0x0F, 0xA0, 0x00, 0x00]
        );
    }
}
