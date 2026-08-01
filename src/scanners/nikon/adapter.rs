//! Working out which adapter is loaded, from either of the two schemes the bodies use
//!
//! The 35 mm bodies advertise an adapter-specific page code on the supported-pages list; the
//! medium format bodies answer a dedicated page with a class byte. Both are read here and both
//! land in the one [`Adapter`] vocabulary, so nothing above this has to know which scheme its
//! model uses.
//!
//! Parsing is kept apart from interpretation on purpose. [`SupportedPages`] and [`HolderReading`]
//! say what the bytes were; their `adapter` methods say what that means. Captured bytes are the
//! only evidence there is for any of this, so the tests pin the parse and the meaning apart.

use crate::adapter::Adapter;
use crate::scsi::cdbs::{VendorPage, VpdPage};

/// The list of pages a 35 mm body advertises, which is how it names its adapter
///
/// The adapter-specific codes sit near the end, so the whole page is asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportedPages(pub Vec<u8>);

impl VendorPage for SupportedPages {
    const PAGE_CODE: u8 = 0x00;
    const ALLOCATION_LENGTH: u8 = 0xFF;

    fn from_page(page: &VpdPage) -> Option<Self> {
        (page.page_code == Self::PAGE_CODE).then(|| SupportedPages(page.data.clone()))
    }
}

impl SupportedPages {
    fn has(&self, code: u8) -> bool {
        self.0.contains(&code)
    }

    /// The adapter these page codes name
    ///
    /// **0x46 versus 0x47 is the discriminator.** The real SA-21 capture advertises 0x46 and the
    /// real SA-30 capture advertises 0x47, and *both* carry 0xE2 — so 0xE2 separates nothing, and
    /// reading it as the roll-feeder marker is what conflated the two adapters. See the captured
    /// pages in the tests below.
    ///
    /// 0x43 and 0x44 were previously read as a strip adapter. With 0x46 accounted for they are
    /// most likely the mounted-slide adapter and the slide feeder, but no capture says which, so
    /// they stay unrecognized rather than becoming another guess.
    pub fn adapter(&self) -> Adapter {
        if self.has(0x46) {
            Adapter::StripFilm
        } else if self.has(0x47) {
            Adapter::RollFilm
        } else if self.has(0x45) || self.has(0xF1) {
            Adapter::Ix240
        } else if self.has(0x43) {
            Adapter::Unknown(0x43)
        } else if self.has(0x44) {
            Adapter::Unknown(0x44)
        } else if self.has(0xF8) || self.has(0xFA) || self.has(0xFB) || self.has(0xFC) {
            // Advertised with an adapter loaded too, so this is a fallback rather than a match
            Adapter::None
        } else {
            Adapter::Unknown(0x00)
        }
    }
}

/// The class of holder a medium format body reports, as the firmware names them
///
/// One namespace across the product line rather than one per body: `Format240` and `Feeder` are
/// 35 mm ideas that no medium format holder is, which is why a class does not pin a part number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HolderClass {
    Mount,
    Strip,
    Format240,
    Feeder,
    SixStrip,
    ThirtySixStrip,
    Other(u8),
}

impl HolderClass {
    fn from_byte(byte: u8) -> Self {
        match byte {
            1 => HolderClass::Mount,
            2 => HolderClass::Strip,
            3 => HolderClass::Format240,
            4 => HolderClass::Feeder,
            5 => HolderClass::SixStrip,
            6 => HolderClass::ThirtySixStrip,
            other => HolderClass::Other(other),
        }
    }

    /// The byte the firmware answered with
    pub fn code(self) -> u8 {
        match self {
            HolderClass::Mount => 1,
            HolderClass::Strip => 2,
            HolderClass::Format240 => 3,
            HolderClass::Feeder => 4,
            HolderClass::SixStrip => 5,
            HolderClass::ThirtySixStrip => 6,
            HolderClass::Other(byte) => byte,
        }
    }
}

/// What a medium format body answers on page 0xC8
///
/// Three signals rather than one, and only two of them are understood.
///
/// `class` is the coarse kind. `width_dots` is how wide the holder is, which is what separates a
/// 120 holder from a 35 mm carrier on the same body, and it agrees with `boundary_x` on page 0xC1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HolderReading {
    /// `None` when the body reports nothing loaded
    pub class: Option<HolderClass>,
    /// Byte 4, meaning unestablished
    ///
    /// It was read as an aperture count, which the hardware contradicts: an FH-869S holds one
    /// continuous strip and has no apertures at all, and it answers 6 here. It may be a format
    /// (every medium format size is 6 by something), a default frame count, or something else
    /// again. Named for its offset rather than for a guess until a capture settles it.
    pub byte_4: u8,
    /// How wide the holder is, in the scanner's own dots
    pub width_dots: u16,
}

impl VendorPage for HolderReading {
    const PAGE_CODE: u8 = 0xC8;
    /// Comfortably covers both observed response sizes, 5 and 21 bytes total
    const ALLOCATION_LENGTH: u8 = 64;

    /// `None` if this is not page 0xC8, or the present flag is set over a body too short to hold
    /// the fields. Both are malformed answers rather than holder states.
    fn from_page(page: &VpdPage) -> Option<Self> {
        if page.page_code != Self::PAGE_CODE {
            return None;
        }
        if page.data.first().copied().unwrap_or(0) == 0 {
            return Some(HolderReading {
                class: None,
                byte_4: 0,
                width_dots: 0,
            });
        }
        let at = |i: usize| page.data.get(i).copied();
        Some(HolderReading {
            class: Some(HolderClass::from_byte(at(3)?)),
            byte_4: at(4).unwrap_or(0),
            // Absent on the shorter answer, which is not an error: only the class is load-bearing
            width_dots: match (at(11), at(12)) {
                (Some(high), Some(low)) => u16::from_be_bytes([high, low]),
                _ => 0,
            },
        })
    }
}

impl HolderReading {
    /// The adapter this reading names
    ///
    /// Only [`Adapter::None`] is settled. Every loaded medium format holder comes back as
    /// [`Adapter::Unknown`] carrying its class byte, because a class does not pin a part: class 2
    /// is a strip holder, but nothing here says whether that is an FH-869S, an FH-869G or a
    /// 35 mm FH-835S, and the capability table needs the part to answer for the FH-869GR.
    ///
    /// The three signals are all preserved on this type so that filling the mapping in is a
    /// table here rather than a reparse. What it needs is a capture per holder, which is what
    /// tagged test holders would produce.
    pub fn adapter(&self) -> Adapter {
        match self.class {
            None => Adapter::None,
            Some(class) => Adapter::Unknown(class.code()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pages(codes: &[u8]) -> SupportedPages {
        SupportedPages(codes.to_vec())
    }

    /// Page 0x00 off an LS-50 with a genuine SA-21 loaded
    fn sa21_capture() -> SupportedPages {
        pages(&[
            0x00, 0x01, 0x40, 0x41, 0x46, 0x50, 0x51, 0x60, 0x61, 0xC1, 0xD1, 0xE1, 0xF0, 0xF8,
            0xE2, 0xFB, 0xFC,
        ])
    }

    /// Page 0x00 off an LS-5000 with an SA-30 loaded
    fn sa30_capture() -> SupportedPages {
        pages(&[
            0x00, 0x01, 0x40, 0x41, 0x47, 0x50, 0x51, 0x60, 0x61, 0x62, 0xC1, 0xD1, 0xE1, 0xE3,
            0xF0, 0xF8, 0xE2, 0xFB, 0xFC,
        ])
    }

    /// The regression this whole module exists to fix
    ///
    /// The SA-21 is a strip adapter, not a roll adapter and not a mounted-slide adapter. Reading
    /// it as either is what made `warm_up` run the carriage motion that ejects its strip.
    #[test]
    fn the_sa21_capture_is_a_strip_adapter() {
        assert_eq!(sa21_capture().adapter(), Adapter::StripFilm);
    }

    #[test]
    fn the_sa30_capture_is_a_roll_adapter() {
        assert_eq!(sa30_capture().adapter(), Adapter::RollFilm);
    }

    /// Both real adapters advertise 0xE2, so it cannot be what tells them apart
    #[test]
    fn the_e2_marker_is_present_on_both_adapters() {
        assert!(sa21_capture().has(0xE2));
        assert!(sa30_capture().has(0xE2));
    }

    /// Taking 0xE2 away changes nothing, because 0x46 and 0x47 are what carry the answer
    #[test]
    fn dropping_the_e2_marker_does_not_change_either_reading() {
        for (capture, expected) in [
            (sa21_capture(), Adapter::StripFilm),
            (sa30_capture(), Adapter::RollFilm),
        ] {
            let mut without = capture;
            without.0.retain(|&code| code != 0xE2);
            assert_eq!(without.adapter(), expected);
        }
    }

    /// 0xF8/0xFA/0xFB/0xFC are advertised with an adapter loaded too, so they only answer last
    #[test]
    fn an_empty_body_still_advertises_the_common_pages() {
        assert_eq!(
            pages(&[0x00, 0x01, 0xF8, 0xFA, 0xFB, 0xFC]).adapter(),
            Adapter::None
        );
    }

    /// Neither code has a capture behind it, so neither may name an adapter
    #[test]
    fn the_unclaimed_codes_stay_unrecognized() {
        assert_eq!(pages(&[0x00, 0x43]).adapter(), Adapter::Unknown(0x43));
        assert_eq!(pages(&[0x00, 0x44]).adapter(), Adapter::Unknown(0x44));
    }

    #[test]
    fn the_ix240_codes_are_recognized() {
        assert_eq!(pages(&[0x00, 0x45]).adapter(), Adapter::Ix240);
        assert_eq!(pages(&[0x00, 0xF1]).adapter(), Adapter::Ix240);
    }

    #[test]
    fn another_page_is_not_the_supported_pages_list() {
        let page = VpdPage {
            page_code: 0x01,
            data: vec![0x46],
        };
        assert_eq!(SupportedPages::from_page(&page), None);
    }

    fn holder_page(data: Vec<u8>) -> VpdPage {
        VpdPage {
            page_code: 0xC8,
            data,
        }
    }

    /// Page 0xC8 with nothing loaded. Real capture: `06 c8 00 01 00`.
    #[test]
    fn an_empty_body_reports_no_class() {
        let reading = HolderReading::from_page(&holder_page(vec![0x00])).unwrap();
        assert_eq!(reading.class, None);
        assert_eq!(reading.adapter(), Adapter::None);
    }

    /// Real capture off an LS-9000 with a strip holder loaded
    ///
    /// Pins all three fields, since the width is what a later mapping will need in order to tell
    /// a 120 holder from a 35 mm carrier: 8964 dots is 56.9 mm, the medium format width.
    #[test]
    fn the_captured_holder_page_parses_every_signal() {
        let reading = HolderReading::from_page(&holder_page(vec![
            0x01, 0x00, 0x00, 0x02, 0x06, 0x00, 0x00, 0x08, 0xbc, 0x00, 0x00, 0x23, 0x04, 0x00,
            0x00, 0x00, 0x00,
        ]))
        .unwrap();
        assert_eq!(reading.class, Some(HolderClass::Strip));
        assert_eq!(reading.byte_4, 6);
        assert_eq!(reading.width_dots, 8964);
    }

    /// A class is not a part number, so it may not become one until a capture says so
    #[test]
    fn a_loaded_holder_keeps_its_class_rather_than_guessing_a_part() {
        let reading = HolderReading {
            class: Some(HolderClass::Strip),
            byte_4: 6,
            width_dots: 8964,
        };
        assert_eq!(reading.adapter(), Adapter::Unknown(2));
    }

    #[test]
    fn an_unknown_class_byte_is_preserved() {
        let reading = HolderReading::from_page(&holder_page(vec![0x01, 0x00, 0x00, 0x7F])).unwrap();
        assert_eq!(reading.class, Some(HolderClass::Other(0x7F)));
        assert_eq!(reading.adapter(), Adapter::Unknown(0x7F));
    }

    /// The shorter answer carries a class and no geometry, which is not an error
    #[test]
    fn a_short_answer_still_yields_a_class() {
        let reading = HolderReading::from_page(&holder_page(vec![0x01, 0x00, 0x00, 0x02])).unwrap();
        assert_eq!(reading.class, Some(HolderClass::Strip));
        assert_eq!(reading.width_dots, 0);
    }

    #[test]
    fn present_but_too_short_to_hold_a_class_is_not_decodable() {
        assert_eq!(
            HolderReading::from_page(&holder_page(vec![0x01, 0x00])),
            None
        );
    }

    #[test]
    fn wrong_page_code_is_not_holder_data() {
        assert!(HolderReading::from_page(&holder_page(vec![0x00])).is_some());
        let mut page = holder_page(vec![0x00]);
        page.page_code = 0xD1;
        assert_eq!(HolderReading::from_page(&page), None);
    }
}
