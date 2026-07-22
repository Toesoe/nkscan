//! INQUIRY (SPC-4 6.6)

use crate::scsi::{Cdb, Command, DataDirection, Error};

#[derive(Debug)]
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

/// A Vital Product Data page, as returned by INQUIRY with EVPD=1.
///
/// The page's contents are vendor- or device-specific (page 0x00 is the one
/// exception: it's the list of page codes the device supports), so this just
/// carries the raw bytes. Interpreting them is up to the caller.
#[derive(Debug)]
pub struct VpdPage {
    pub page_code: u8,
    pub data: Vec<u8>,
}

/// INQUIRY with EVPD=1 (SPC-4 6.6.2): requests a Vital Product Data page
/// instead of standard inquiry data.
#[derive(Debug)]
pub struct VpdInquiry {
    pub page_code: u8,
    pub allocation_length: u8,
}

impl VpdInquiry {
    pub fn new(page_code: u8, allocation_length: u8) -> Self {
        VpdInquiry {
            page_code,
            allocation_length,
        }
    }
}

impl Command for VpdInquiry {
    type Response = VpdPage;
    type Cdb = Cdb<6>;

    fn cdb(&self) -> Self::Cdb {
        Cdb([0x12, 0x01, self.page_code, 0x00, self.allocation_length, 0x00])
    }

    fn direction(&self) -> DataDirection {
        DataDirection::Read
    }

    fn data_length(&self) -> usize {
        self.allocation_length as usize
    }

    fn decode(&self, data: &[u8]) -> Result<VpdPage, Error> {
        if data.len() < 4 {
            return Err(Error::InvalidResponse(
                "VPD page response shorter than the 4-byte header",
            ));
        }
        let page_code = data[1];
        let page_length = u16::from_be_bytes([data[2], data[3]]) as usize;
        // The device may have reported a page_length longer than what
        // actually fit in `allocation_length` bytes; don't index past what
        // we actually have.
        let end = (4 + page_length).min(data.len());
        Ok(VpdPage {
            page_code,
            data: data[4..end].to_vec(),
        })
    }
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

    #[test]
    fn vpd_cdb_sets_evpd_bit_and_page_code() {
        let vpd = VpdInquiry::new(0xC8, 64);
        assert_eq!(vpd.cdb().0, [0x12, 0x01, 0xC8, 0x00, 64, 0x00]);
    }

    #[test]
    fn vpd_direction_is_read() {
        assert_eq!(VpdInquiry::new(0xC8, 64).direction(), DataDirection::Read);
    }

    #[test]
    fn vpd_data_length_matches_allocation_length() {
        assert_eq!(VpdInquiry::new(0xC8, 64).data_length(), 64);
    }

    #[test]
    fn vpd_decode_parses_page_code_and_trims_to_page_length() {
        // peripheral=0x00, page_code=0xC8, page_length=2, then payload + padding
        let data = [0x00, 0xC8, 0x00, 0x02, 0xAA, 0xBB, 0x00, 0x00];
        let page = VpdInquiry::new(0xC8, 8).decode(&data).unwrap();
        assert_eq!(page.page_code, 0xC8);
        assert_eq!(page.data, vec![0xAA, 0xBB]);
    }

    #[test]
    fn vpd_decode_clamps_to_available_data() {
        // page_length claims 10 bytes but only 4 are actually present
        let data = [0x00, 0xC8, 0x00, 0x0A, 0xAA, 0xBB, 0xCC, 0xDD];
        let page = VpdInquiry::new(0xC8, 8).decode(&data).unwrap();
        assert_eq!(page.data, vec![0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn vpd_decode_rejects_short_response() {
        let data = [0u8; 3];
        let err = VpdInquiry::new(0xC8, 3).decode(&data).unwrap_err();
        assert!(matches!(err, Error::InvalidResponse(_)));
    }
}
