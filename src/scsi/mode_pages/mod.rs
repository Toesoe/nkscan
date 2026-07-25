//! Decoders for standardized SCSI mode pages (as returned by MODE SENSE),
//! as opposed to vendor-specific pages, which belong under the scanner
//! that defines them.

use crate::scsi::cdbs::ModeSenseResponse;

mod measurement_units;

pub use measurement_units::*;

/// One mode page, as MODE SENSE returns it and MODE SELECT takes it back
pub trait ModePage: Sized {
    const PAGE_CODE: u8;
    /// The page's own length byte: everything after the page code and this byte
    const BODY_LEN: u8;

    /// 4-byte mode parameter header, 2-byte page header, then the body
    fn allocation_length() -> u8 {
        4 + 2 + Self::BODY_LEN
    }

    /// Decode the page body, the bytes after the page code and length
    fn decode_body(body: &[u8]) -> Option<Self>;

    /// Encode the page body
    fn encode_body(&self) -> Vec<u8>;

    /// Pull this page out of a MODE SENSE response, skipping any block descriptors
    fn from_response(response: &ModeSenseResponse) -> Option<Self> {
        let page = response
            .data
            .get(response.header.block_descriptor_length as usize..)?;
        if page.len() < 2 + Self::BODY_LEN as usize || page[0] & 0x3F != Self::PAGE_CODE {
            return None;
        }
        Self::decode_body(&page[2..])
    }

    /// The page as it goes into a MODE SELECT parameter list
    fn page_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![Self::PAGE_CODE, Self::BODY_LEN];
        bytes.extend_from_slice(&self.encode_body());
        bytes
    }
}
