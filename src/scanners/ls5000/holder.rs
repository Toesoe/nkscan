//! Film adapter detection via the VPD supported-pages list
//!
//! Inferred from what the firmware advertises on page 0x00, keyed off page codes rather than
//! Nikon's labels. The roll feeder advertises 0x47 and 0xE2 together, and names itself on
//! page 0x01.

use super::Ls5000;
use crate::{
    scanners::{Scanner, nikon::page_name},
    scsi::{
        Transport,
        cdbs::{VendorPage, VpdPage},
    },
};

/// Carries the adapter's name on this model
const ADAPTER_NAME_PAGE: u8 = 0x01;

/// Film adapter currently loaded
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Holder {
    /// No adapter-specific page. 0xF8/0xFB/0xFC are advertised with an adapter loaded
    /// too, so this is a fallback rather than a positive match.
    None,
    /// 0x47 with 0xE2, the motorized roll feeder.
    RollFeeder,
    /// 0x47 without 0xE2
    SixStrip,
    /// 0x43/0x44: strip adapter or a fixed holder
    Strip,
    /// 0x45/0xF1, APS / IX240
    Format240,
    /// 0x46
    Mount,
    Unknown,
}

impl VendorPage for Holder {
    const PAGE_CODE: u8 = 0x00;
    /// The whole page, since the adapter markers sit near its end
    const ALLOCATION_LENGTH: u8 = 0xFF;

    fn from_page(page: &VpdPage) -> Option<Self> {
        if page.page_code != Self::PAGE_CODE {
            return None;
        }
        let has = |code: u8| page.data.contains(&code);
        Some(if has(0x47) {
            // 0xE2 separates the motorized feeder from a six-frame strip holder
            if has(0xE2) {
                Holder::RollFeeder
            } else {
                Holder::SixStrip
            }
        } else if has(0x43) || has(0x44) {
            Holder::Strip
        } else if has(0x45) || has(0xF1) {
            Holder::Format240
        } else if has(0x46) {
            Holder::Mount
        } else if has(0xF8) || has(0xFA) || has(0xFB) || has(0xFC) {
            Holder::None
        } else {
            Holder::Unknown
        })
    }
}

impl Holder {
    /// Whether the feeder senses frames along a roll and reports them in the transport table
    pub fn is_roll(self) -> bool {
        matches!(self, Holder::RollFeeder)
    }
}

impl<T> Ls5000<T>
where
    T: Transport,
{
    /// What the adapter calls itself, `None` without page 0x01
    ///
    /// The roll feeder answers `36Strip`, which is a positive identification rather than the
    /// guess [`holder`](crate::scanners::FilmHolder::holder) makes from the page list.
    pub fn adapter_name(&mut self) -> Option<String> {
        page_name(&self.vpd_page(ADAPTER_NAME_PAGE).ok()?)
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

    /// Page 0x00 as a unit answers it with the roll feeder loaded
    fn captured() -> VpdPage {
        page(vec![
            0x00, 0x01, 0x40, 0x41, 0x47, 0x50, 0x51, 0x60, 0x61, 0x62, 0xC1, 0xD1, 0xE1, 0xE3,
            0xF0, 0xF8, 0xE2, 0xFB, 0xFC,
        ])
    }

    #[test]
    fn the_captured_page_list_is_the_roll_feeder() {
        let holder = Holder::from_page(&captured()).unwrap();
        assert_eq!(holder, Holder::RollFeeder);
        assert!(holder.is_roll());
    }

    /// The list also carries 0xF8/0xFB/0xFC, which the fallback arm would otherwise claim
    #[test]
    fn the_feeder_marker_is_what_separates_it_from_a_six_strip_holder() {
        let mut without = captured();
        without.data.retain(|&code| code != 0xE2);
        let holder = Holder::from_page(&without).unwrap();
        assert_eq!(holder, Holder::SixStrip);
        assert!(!holder.is_roll());
    }

    #[test]
    fn recognizes_each_adapter() {
        const STANDARD: [u8; 6] = [0x00, 0x01, 0x40, 0x41, 0x50, 0x51];
        for (extra, expected) in [
            (vec![0xF8, 0xFA, 0xFB, 0xFC], Holder::None),
            (vec![0x47, 0xE2], Holder::RollFeeder),
            (vec![0x47], Holder::SixStrip),
            (vec![0x43, 0x44], Holder::Strip),
            (vec![0x45, 0xF1], Holder::Format240),
            (vec![0x46], Holder::Mount),
            (vec![], Holder::Unknown),
        ] {
            let mut data = STANDARD.to_vec();
            data.extend_from_slice(&extra);
            assert_eq!(
                Holder::from_page(&page(data)),
                Some(expected),
                "pages {extra:02x?}"
            );
        }
    }

    /// Page 0x01 as the feeder answers it, name and all
    #[test]
    fn reads_the_adapter_name_off_the_captured_page() {
        let captured = [0x08, 0x33, 0x36, 0x53, 0x74, 0x72, 0x69, 0x70, 0x00];
        assert_eq!(page_name(&captured).as_deref(), Some("36Strip"));
    }

    #[test]
    fn another_page_is_not_the_supported_pages_list() {
        let mut other = captured();
        other.page_code = 0x01;
        assert_eq!(Holder::from_page(&other), None);
    }
}
