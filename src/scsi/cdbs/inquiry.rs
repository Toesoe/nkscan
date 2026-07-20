//! INQUIRY (SPC-4 6.6)

use crate::scsi::{Cdb, Command, DataDirection, Error};

pub struct Inquiry {
    pub allocation_length: u8,
}

impl Inquiry {
    pub fn new() -> Self {
        Inquiry {
            allocation_length: 36,
        }
    }
}

impl Default for Inquiry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct InquiryResponse {
    pub peripheral: u8,
    pub vendor: String,
    pub product: String,
    pub revision: String,
}

impl Command for Inquiry {
    type Response = InquiryResponse;
    type Cdb = Cdb<6>;

    fn cdb(&self) -> Self::Cdb {
        Cdb([0x12, 0x00, 0x00, 0x00, self.allocation_length, 0x00])
    }

    fn direction(&self) -> DataDirection {
        DataDirection::Read
    }

    fn data_length(&self) -> usize {
        self.allocation_length as usize
    }

    fn decode(&self, data: &[u8]) -> Result<InquiryResponse, Error> {
        if data.len() < 36 {
            return Err(Error::InvalidResponse(
                "standard INQUIRY response shorter than 36 bytes",
            ));
        }

        Ok(InquiryResponse {
            peripheral: data[0],
            vendor: ascii_field(&data[8..16]),
            product: ascii_field(&data[16..32]),
            revision: ascii_field(&data[32..36]),
        })
    }
}

/// INQUIRY text fields are ASCII, space-padded to a fixed width
fn ascii_field(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_defaults_to_standard_allocation_length() {
        assert_eq!(Inquiry::new().allocation_length, 36);
    }

    #[test]
    fn cdb_encodes_opcode_and_allocation_length() {
        let inquiry = Inquiry {
            allocation_length: 96,
        };
        assert_eq!(inquiry.cdb().0, [0x12, 0x00, 0x00, 0x00, 96, 0x00]);
    }

    #[test]
    fn direction_is_read() {
        assert_eq!(Inquiry::new().direction(), DataDirection::Read);
    }

    #[test]
    fn data_length_matches_allocation_length() {
        let inquiry = Inquiry {
            allocation_length: 200,
        };
        assert_eq!(inquiry.data_length(), 200);
    }

    #[test]
    fn decode_parses_fields_and_trims_trailing_padding() {
        let mut data = [0x20u8; 36]; // space-fill vendor/product/revision
        data[0] = 0x05; // peripheral qualifier/device type byte
        data[8..14].copy_from_slice(b"NIKON ");
        data[16..24].copy_from_slice(b"COOLSCAN");
        data[32..36].copy_from_slice(b"1.00");

        let response = Inquiry::new().decode(&data).unwrap();

        assert_eq!(response.peripheral, 0x05);
        assert_eq!(response.vendor, "NIKON");
        assert_eq!(response.product, "COOLSCAN");
        assert_eq!(response.revision, "1.00");
    }

    #[test]
    fn decode_rejects_short_response() {
        let data = [0u8; 35];
        let err = Inquiry::new().decode(&data).unwrap_err();
        assert!(matches!(err, Error::InvalidResponse(_)));
    }
}
