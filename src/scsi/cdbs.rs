//! Standard CBDs that are part of SCSI

use super::*;

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
