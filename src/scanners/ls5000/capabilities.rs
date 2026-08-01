//! What the scanner says it can do, from vital product data page 0xC1
//!
//! 16-bit samples, 4000 DPI optical, boundaries of 3946 by 5959 dots, a 5959-dot frame pitch
//! and focus 0 to 323. Offset 71 counts roll-feeder slots.

use crate::scanners::nikon::limits::{self as nikon, DeviceLimits};
use crate::scsi::{self as scsi};
use crate::scsi::{Transport, TransportExt, cdbs::VpdInquiry, cdbs::VpdPage};

const PAGE: u8 = 0xC1;
/// Covers the last field we read, at offset 78. The unit answers with 83.
const ALLOCATION_LENGTH: u8 = 87;

/// Ask the device, before there is a scanner to ask
pub(super) fn read<T: Transport + ?Sized>(transport: &mut T) -> Result<DeviceLimits, scsi::Error> {
    nikon::read(transport, ALLOCATION_LENGTH)
}

/// Slots the feeder reports for the loaded film, 0 for none
///
/// Describes the film rather than the adapter, so it is read fresh rather than cached with
/// [`DeviceLimits`]. A full roll reads 40; a shorter one reads what it sensed.
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
    use crate::scanners::nikon::limits::DeviceLimits;
    use crate::scsi::cdbs::{VendorPage, VpdPage};

    /// What the fixture page parses to, for anything that needs geometry to test against
    pub fn capabilities() -> DeviceLimits {
        DeviceLimits::from_page(&captured()).expect("the fixture page parses")
    }

    /// The whole response, header included, as a mock transport has to answer it
    pub fn raw_page() -> Vec<u8> {
        let body = captured().data;
        let mut raw = vec![0x06, super::PAGE];
        raw.extend_from_slice(&(body.len() as u16).to_be_bytes());
        raw.extend_from_slice(&body);
        raw
    }

    /// The page a unit answers with a roll feeder loaded and no roll sensed yet
    ///
    /// Assembled from the field values rather than held as a blob, so a change to the offsets
    /// the shared parser reads shows up as a failure here rather than as a fixture that no
    /// longer describes anything.
    pub fn captured() -> VpdPage {
        page(0)
    }

    /// The same page once a 40-slot roll has been sensed
    pub fn captured_with_roll() -> VpdPage {
        page(40)
    }

    /// The device answers 83 bytes of body
    const BODY_LEN: usize = 0x53;

    fn page(slots: u8) -> VpdPage {
        let mut data = vec![0u8; BODY_LEN];
        let mut short = |at: usize, v: u16| data[at..at + 2].copy_from_slice(&v.to_be_bytes());
        // x resolution: optical, max, min
        short(14, 4000);
        short(16, 4000);
        short(18, 90);
        // y resolution, the same
        short(36, 4000);
        short(38, 4000);
        short(40, 90);
        // Focus travel
        short(72, 0);
        short(74, 323);

        let mut long = |at: usize, v: u32| data[at..at + 4].copy_from_slice(&v.to_be_bytes());
        long(32, 3946); // boundary_x
        long(54, 5959); // boundary_y, one frame
        long(58, 5959); // frame pitch

        data[71] = slots;
        data[78] = 16; // sample depth
        VpdPage {
            page_code: 0xC1,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{fixture::*, *};
    use crate::scanners::nikon::limits::ResolutionRange;
    use crate::scsi::cdbs::VendorPage;

    /// The allocation length has to cover the whole body the device answers with
    #[test]
    fn the_allocation_length_covers_the_page() {
        assert!(usize::from(ALLOCATION_LENGTH) >= captured().data.len());
    }

    #[test]
    fn parses_the_captured_page() {
        let caps = DeviceLimits::from_page(&captured()).unwrap();

        assert_eq!(caps.boundary_x, 3946);
        assert_eq!(caps.boundary_y, 5959);
        assert_eq!(caps.frame_pitch, 5959);
        assert_eq!(caps.focus, (0, 323));
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

    /// The window descriptor carries this straight through, so it comes off the device
    #[test]
    fn the_sample_depth_is_sixteen_bits() {
        assert_eq!(DeviceLimits::from_page(&captured()).unwrap().max_bits, 16);
    }

    /// Empty feeder reads 0, a loaded 40-slot roll reads 40
    #[test]
    fn reads_the_sensed_slot_count() {
        assert_eq!(frames_in(&captured()), Some(0));
        assert_eq!(frames_in(&captured_with_roll()), Some(40));
    }

    /// The geometry is per adapter; only the slot count moves when a roll is loaded
    #[test]
    fn loading_a_roll_does_not_change_the_geometry() {
        let empty = DeviceLimits::from_page(&captured()).unwrap();
        let loaded = DeviceLimits::from_page(&captured_with_roll()).unwrap();
        assert_eq!(empty.boundary_x, loaded.boundary_x);
        assert_eq!(empty.boundary_y, loaded.boundary_y);
        assert_eq!(empty.frame_pitch, loaded.frame_pitch);
    }

    /// `max_x`/`max_y` subtract one from these, so a zero would wrap into a window the size of
    /// the address space rather than failing anywhere near the bad page
    #[test]
    fn a_zero_boundary_is_not_a_usable_page() {
        for offset in [32, 54] {
            let mut page = captured();
            page.data[offset..offset + 4].fill(0);
            assert!(
                DeviceLimits::from_page(&page).is_none(),
                "a zero boundary at {offset} should be rejected"
            );
        }
    }

    #[test]
    fn a_short_page_is_not_this_page() {
        let page = VpdPage {
            page_code: 0xC1,
            data: vec![0u8; 10],
        };
        assert_eq!(DeviceLimits::from_page(&page), None);
        assert_eq!(frames_in(&page), None);
    }

    #[test]
    fn rejects_another_page() {
        let mut other = captured();
        other.page_code = 0xD1;
        assert_eq!(DeviceLimits::from_page(&other), None);
        assert_eq!(frames_in(&other), None);
    }
}
