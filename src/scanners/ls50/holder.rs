//! Film adapter detection via the VPD supported-pages list
//!
//! Inferred from what the firmware advertises on page 0x00, keyed off page codes rather than
//! Nikon's labels, which confuse MA-21 with SA-21. Adapters with page 0x46 also name
//! themselves, see [`adapter_name`](Ls50::adapter_name).

use super::Ls50;
use crate::{
    scanners::Scanner,
    scsi::{
        Transport,
        cdbs::{VendorPage, VpdPage},
    },
};

/// Carries the adapter's name on the adapters that have this page
const ADAPTER_NAME_PAGE: u8 = 0x46;

/// Film adapter currently loaded
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Holder {
    /// No adapter-specific page. 0xF8/0xFB/0xFC are advertised with an adapter loaded
    /// too, so this is a fallback rather than a positive match.
    None,
    /// 0x46 without 0xE2
    Mount,
    /// 0x43/0x44: SA-21 strip feeder or the FH-3 holder
    Strip,
    /// 0x47
    SixStrip,
    /// 0x45/0xF1, APS / IX240
    Format240,
    /// 0x46 with 0xE2. Confirmed against an SA-21.
    Feeder,
    /// 36-strip mode lands here: its page 0x10 collides with the standard device-id
    /// page, so it can't be told apart.
    Unknown,
}

impl VendorPage for Holder {
    const PAGE_CODE: u8 = 0x00;
    /// What Nikon Scan asks for
    const ALLOCATION_LENGTH: u8 = 0xFF;

    fn from_page(page: &VpdPage) -> Option<Self> {
        if page.page_code != Self::PAGE_CODE {
            return None;
        }
        let has = |code: u8| page.data.contains(&code);
        Some(if has(0x43) || has(0x44) {
            Holder::Strip
        } else if has(0x47) {
            Holder::SixStrip
        } else if has(0x45) || has(0xF1) {
            Holder::Format240
        } else if has(0x46) {
            if has(0xE2) {
                Holder::Feeder
            } else {
                Holder::Mount
            }
        } else if has(0xF8) || has(0xFA) || has(0xFB) || has(0xFC) {
            Holder::None
        } else {
            Holder::Unknown
        })
    }
}

use crate::scanners::nikon::page_name;

impl<T> Ls50<T>
where
    T: Transport,
{
    /// What the adapter calls itself, `None` without page 0x46
    ///
    /// The strip adapter answers `36SA_OBJECT`: a positive identification, unlike the guess
    /// [`holder`](crate::scanners::FilmHolder::holder) makes from the page list.
    pub fn adapter_name(&mut self) -> Option<String> {
        page_name(&self.vpd_page(ADAPTER_NAME_PAGE).ok()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Standard pages, always present
    const STANDARD: [u8; 8] = [0x00, 0x01, 0x10, 0x40, 0x41, 0x50, 0x51, 0x52];

    fn page(data: Vec<u8>) -> VpdPage {
        VpdPage {
            page_code: Holder::PAGE_CODE,
            data,
        }
    }

    fn list(extra: &[u8]) -> VpdPage {
        let mut data = STANDARD.to_vec();
        data.extend_from_slice(extra);
        page(data)
    }

    #[test]
    fn recognizes_each_adapter() {
        for (extra, expected) in [
            (vec![0xF8, 0xFA, 0xFB, 0xFC], Holder::None),
            (vec![0x43, 0x44, 0xE2], Holder::Strip),
            (vec![0x47, 0xE2], Holder::SixStrip),
            (vec![0x45, 0xF1], Holder::Format240),
            (vec![0x46], Holder::Mount),
            (vec![0x46, 0xE2], Holder::Feeder),
            (vec![], Holder::Unknown),
        ] {
            assert_eq!(
                Holder::from_page(&list(&extra)),
                Some(expected),
                "pages {extra:02x?}"
            );
        }
    }

    #[test]
    fn sa21_strip_feeder_real_capture() {
        // Real page 0x00 list off an LS-50 with the SA-21 feeder
        let real = page(vec![
            0x00, 0x01, 0x40, 0x41, 0x46, 0x50, 0x51, 0x60, 0x61, 0xC1, 0xD1, 0xE1, 0xF0, 0xF8,
            0xE2, 0xFB, 0xFC,
        ]);
        assert_eq!(Holder::from_page(&real), Some(Holder::Feeder));
    }

    /// Page 0x46 exactly as the strip adapter answers it
    #[test]
    fn reads_the_adapter_name_off_the_captured_page() {
        let captured = [
            0x0C, 0x33, 0x36, 0x53, 0x41, 0x5F, 0x4F, 0x42, 0x4A, 0x45, 0x43, 0x54, 0x00,
        ];
        assert_eq!(page_name(&captured).as_deref(), Some("36SA_OBJECT"));
    }

    /// Pages 0x60 and 0x61 carry parameter names in the same shape
    #[test]
    fn reads_a_parameter_name_in_the_same_shape() {
        let exp_time = [0x09, 0x45, 0x58, 0x50, 0x5F, 0x54, 0x49, 0x4D, 0x45, 0x00];
        assert_eq!(page_name(&exp_time).as_deref(), Some("EXP_TIME"));
    }

    #[test]
    fn a_page_with_no_name_in_it_reads_none() {
        assert_eq!(page_name(&[]), None);
        assert_eq!(page_name(&[0x00]), None);
        // A count reaching past what arrived
        assert_eq!(page_name(&[0x0C, 0x33, 0x36]), None);
    }

    #[test]
    fn another_page_is_not_the_supported_pages_list() {
        let mut other = list(&[0x43]);
        other.page_code = 0x01;
        assert_eq!(Holder::from_page(&other), None);
    }
}
