//! Film holder detection via VPD page 0xC8

use crate::scsi::cdbs::VpdPage;

/// Film holder currently loaded in the scanner
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Holder {
    /// No film holder
    None,
    /// Strip film
    Strip,
    // The rest of these are documented in the binary but I don't know what they are
    Mount,
    Format240,
    Feeder,
    SixStrip,
    ThirtySixStrip,
    /// A holder is present, but its type byte isn't one we've seen before.
    Unknown(u8),
}

impl Holder {
    /// Holder page code probe
    pub const PAGE_CODE: u8 = 0xC8;
    /// Comfortably covers both observed response sizes (5 and 21 bytes total).
    pub const ALLOCATION_LENGTH: u8 = 64;

    /// Decode a VPD page 0xC8 response. Returns `None` if `page` isn't
    /// actually page 0xC8, or if the present flag is set but the payload is
    /// too short to hold a type byte - both cases the caller should treat as
    /// a real error, not a holder state.
    pub fn from_page(page: &VpdPage) -> Option<Self> {
        if page.page_code != Self::PAGE_CODE {
            return None;
        }
        if page.data.first().copied().unwrap_or(0) == 0 {
            return Some(Holder::None);
        }
        Some(match *page.data.get(3)? {
            1 => Holder::Mount,
            2 => Holder::Strip,
            3 => Holder::Format240,
            4 => Holder::Feeder,
            5 => Holder::SixStrip,
            6 => Holder::ThirtySixStrip,
            other => Holder::Unknown(other),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(data: Vec<u8>) -> VpdPage {
        VpdPage {
            page_code: Holder::PAGE_CODE,
            data,
        }
    }

    #[test]
    fn no_holder() {
        // Real capture: `06 c8 00 01 00` -> payload `00`.
        assert_eq!(Holder::from_page(&page(vec![0x00])), Some(Holder::None));
    }

    #[test]
    fn strip_holder() {
        // Real capture: `06 c8 00 11 01 00 00 02 06 00 00 08 bc 00 00 23 04 00 00 00 00`
        let data = vec![
            0x01, 0x00, 0x00, 0x02, 0x06, 0x00, 0x00, 0x08, 0xbc, 0x00, 0x00, 0x23, 0x04, 0x00,
            0x00, 0x00, 0x00,
        ];
        assert_eq!(Holder::from_page(&page(data)), Some(Holder::Strip));
    }

    #[test]
    fn unknown_holder_type_is_preserved() {
        let data = vec![0x01, 0x00, 0x00, 0x7F];
        assert_eq!(Holder::from_page(&page(data)), Some(Holder::Unknown(0x7F)));
    }

    #[test]
    fn wrong_page_code_is_not_holder_data() {
        let mut p = page(vec![0x00]);
        p.page_code = 0xD1;
        assert_eq!(Holder::from_page(&p), None);
    }

    #[test]
    fn present_flag_set_but_payload_too_short_is_not_decodable() {
        let data = vec![0x01, 0x00];
        assert_eq!(Holder::from_page(&page(data)), None);
    }
}
