//! What the scanner says it can do, from vital product data page 0xC1
//!
//! An LS-50 with an SA-21 reports 14-bit samples, 4000 DPI optical, boundaries of 3946 by 5959
//! dots, a 5959-dot frame pitch and focus 0 to 323: everything the driver would else hardcode.

use crate::scanners::nikon::capabilities::{self as nikon, Capabilities};
use crate::scsi::{self as scsi};
use crate::scsi::{Transport, TransportExt, cdbs::VendorPage, cdbs::VpdInquiry, cdbs::VpdPage};

const PAGE: u8 = 0xC1;
/// Covers the last field we read, at offset 78. The unit answers with 83.
const ALLOCATION_LENGTH: u8 = 87;

/// Ask the device, before there is a scanner to ask
///
/// An unreadable page is an error, not a cue to fall back on constants: page 0x00 advertises
/// 0xC1 and no healthy unit has been seen to withhold it.
pub(super) fn read<T: Transport + ?Sized>(transport: &mut T) -> Result<Capabilities, scsi::Error> {
    nikon::read(transport, ALLOCATION_LENGTH)
}

impl VendorPage for Capabilities {
    const PAGE_CODE: u8 = PAGE;
    const ALLOCATION_LENGTH: u8 = ALLOCATION_LENGTH;

    fn from_page(page: &VpdPage) -> Option<Self> {
        if page.page_code != PAGE {
            return None;
        }
        Self::parse(&page.data).ok()
    }
}

/// Frames sensed on the loaded strip, 0 for none
///
/// Not cached with [`Capabilities`]: it describes the film, not the adapter. A six-frame strip
/// reads 6 freshly loaded and 1 after a pass, so only trust it before the first.
pub(super) fn read_sensed_frames<T: Transport + ?Sized>(transport: &mut T) -> u32 {
    transport
        .send(&VpdInquiry::new(PAGE, ALLOCATION_LENGTH))
        .ok()
        .and_then(|page| frames_in(&page))
        .unwrap_or(0)
}

fn frames_in(page: &VpdPage) -> Option<u32> {
    if page.page_code != PAGE {
        return None;
    }
    page.data.get(71).map(|&n| u32::from(n))
}

#[cfg(test)]
pub(super) mod fixture {
    use crate::scanners::nikon::capabilities::Capabilities;
    use crate::scsi::cdbs::{VendorPage, VpdPage};

    /// What the captured page parses to, for anything that needs geometry to test against
    pub fn capabilities() -> Capabilities {
        Capabilities::from_page(&captured()).expect("the captured page parses")
    }

    /// The whole response, header included, as a mock transport has to answer it
    pub fn raw_page() -> Vec<u8> {
        let body = captured().data;
        let mut raw = vec![0x06, super::PAGE];
        raw.extend_from_slice(&(body.len() as u16).to_be_bytes());
        raw.extend_from_slice(&body);
        raw
    }

    /// The page as an LS-50 ED with an SA-21 and a six-frame strip loaded reports it
    ///
    /// The header the device sends is `06 C1 00 53`, so the body below is 83 bytes.
    pub fn captured() -> VpdPage {
        let hex = "03 00 3A 00 0F 00 00 00 40 01 01 00 01 31 0F A0 0F A0 00 5A 00 00 00 00 \
                   00 00 00 00 00 00 00 00 00 00 0F 6A 0F A0 0F A0 00 5A 00 00 BA 38 00 00 \
                   00 00 00 00 00 00 00 00 17 47 00 00 17 47 00 00 00 00 00 61 00 61 06 06 \
                   00 00 01 43 00 00 0E 0F 6A 00 01";
        VpdPage {
            page_code: 0xC1,
            data: hex
                .split_whitespace()
                .map(|b| u8::from_str_radix(b, 16).unwrap())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{fixture::captured, *};
    use crate::scanners::nikon::capabilities::ResolutionRange;

    /// `max_x`/`max_y` subtract one from these, so a zero would wrap into a window the size of
    /// the address space rather than failing anywhere near the bad page
    #[test]
    fn a_zero_boundary_is_not_a_usable_page() {
        for offset in [32, 54] {
            let mut page = captured();
            page.data[offset..offset + 4].fill(0);
            assert!(
                Capabilities::from_page(&page).is_none(),
                "a zero boundary at {offset} should be rejected"
            );
        }
    }

    #[test]
    fn parses_the_captured_page() {
        let caps = Capabilities::from_page(&captured()).unwrap();

        assert_eq!(caps.max_bits, 14);
        assert_eq!(caps.boundary_x, 3946);
        assert_eq!(caps.boundary_y, 5959);
        assert_eq!(caps.frame_pitch, 5959);
        assert_eq!(
            caps.x_resolution,
            ResolutionRange {
                optical: 4000,
                min: 90,
                max: 4000
            }
        );
        assert_eq!(caps.y_resolution, caps.x_resolution);
    }

    /// Offset 58 is its own field, not a second copy of the boundary
    ///
    /// This feeder answers 5959 at both, which alone would not tell them apart; a holder
    /// adapter answers 13176 at 54 and 0 at 58.
    #[test]
    fn the_frame_pitch_is_device_reported_and_distinct_from_the_boundary() {
        let caps = Capabilities::from_page(&captured()).unwrap();
        assert_eq!(caps.frame_pitch, 5959);

        let mut holder = captured();
        holder.data[54..58].copy_from_slice(&13176u32.to_be_bytes());
        holder.data[58..62].copy_from_slice(&0u32.to_be_bytes());
        let caps = Capabilities::from_page(&holder).unwrap();
        assert_eq!(caps.boundary_y, 13176);
        assert_eq!(caps.frame_pitch, 0);
    }

    /// 14-bit samples, delivered as big-endian u16, and the window descriptor carries it
    #[test]
    fn the_sample_depth_is_device_reported() {
        let caps = Capabilities::from_page(&captured()).unwrap();
        assert_eq!(caps.max_bits, 0x0E);
    }

    /// 320 is the setpoint that was probed on hardware and read back exactly
    #[test]
    fn the_probed_focus_setpoint_is_in_the_reported_range() {
        let caps = Capabilities::from_page(&captured()).unwrap();
        assert_eq!(caps.focus, (0, 323));
        assert!(caps.focus.0 <= 320 && 320 <= caps.focus.1);
    }

    /// The count lives on the same page but is deliberately not part of the geometry
    #[test]
    fn reads_the_sensed_frame_count() {
        assert_eq!(frames_in(&captured()), Some(6));
    }

    #[test]
    fn a_short_page_is_not_this_page() {
        let page = VpdPage {
            page_code: 0xC1,
            data: vec![0u8; 10],
        };
        assert_eq!(Capabilities::from_page(&page), None);
        assert_eq!(frames_in(&page), None);
    }

    #[test]
    fn rejects_another_page() {
        let mut other = captured();
        other.page_code = 0xD1;
        assert_eq!(Capabilities::from_page(&other), None);
        assert_eq!(frames_in(&other), None);
    }
}
