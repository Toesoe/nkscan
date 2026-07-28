//! What a scanner says it can do, from vital product data page 0xC1
//!
//! Reading it means the geometry the library will accept comes from the device rather than from
//! constants, which matters most for [`boundary_y`](Capabilities::boundary_y). Not every field
//! is populated by every unit.

use crate::scsi::{
    self, Transport, TransportExt,
    cdbs::{VendorPage, VpdInquiry, VpdPage},
};

/// The page code this lives on
pub const PAGE: u8 = 0xC1;

/// What every unit seen answers this page with
pub const ALLOCATION_LENGTH: u8 = 87;

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

/// Device-reported limits and geometry
///
/// Field offsets are into the page body, after the four-byte VPD header.
///
/// Lengths are in the measurement units the driver set at open, not in a fixed pitch: the
/// divisor is a MODE SELECT parameter and a unit's optical resolution is whatever
/// [`x_resolution`](Self::x_resolution) reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    /// Bits per sample the scanner produces
    pub max_bits: u8,
    /// Native sensor resolution, and the range it will divide down to
    pub x_resolution: ResolutionRange,
    /// Same along the feed or stage
    pub y_resolution: ResolutionRange,
    /// Widest window along the sensor bar
    pub boundary_x: u32,
    /// Longest window along the feed
    ///
    /// One frame's worth, not the strip. Setting a frame window longer than this can drive the
    /// mechanism into its endstop rather than being refused.
    pub boundary_y: u32,
    /// How far the feed advances between frames, 0 for an adapter that does not advance
    pub frame_pitch: u32,
    /// Frames the device reports for the loaded holder
    ///
    /// Unreliable as a count, and stale as soon as a pass runs. Read it fresh rather than
    /// trusting this copy.
    pub frames: u8,
    /// The range [`focus`](crate::scanners::Focus::focus) is reported and set in
    pub focus: (u16, u16),
}

/// An optical resolution and the range the firmware will divide it into
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolutionRange {
    pub optical: u16,
    pub min: u16,
    pub max: u16,
}

impl Capabilities {
    /// The last scannable dot along the sensor bar
    ///
    /// The boundaries count dots; a window is bounded by the last of them. Never underflows:
    /// [`parse`](Self::parse) rejects a page reporting a zero boundary.
    pub fn max_x(&self) -> u32 {
        self.boundary_x - 1
    }

    /// The last scannable dot along the feed, one frame's worth. See [`max_x`](Self::max_x).
    pub fn max_y(&self) -> u32 {
        self.boundary_y - 1
    }

    pub fn parse(data: &[u8]) -> Result<Self, scsi::Error> {
        // The last field we read sits at 78, so anything shorter is not this page
        if data.len() < 79 {
            return Err(scsi::Error::InvalidResponse(
                "capability page shorter than its known fields",
            ));
        }
        let short = |at: usize| u16::from_be_bytes([data[at], data[at + 1]]);
        let long =
            |at: usize| u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);

        // A scanner with no scannable area is not a page we can work from, and taking it at its
        // word would underflow `max_x`/`max_y` into a window the size of the address space
        let (boundary_x, boundary_y) = (long(32), long(54));
        if boundary_x == 0 || boundary_y == 0 {
            return Err(scsi::Error::InvalidResponse(
                "capability page reports no scannable area",
            ));
        }

        Ok(Self {
            max_bits: data[78],
            x_resolution: ResolutionRange {
                optical: short(14),
                max: short(16),
                min: short(18),
            },
            y_resolution: ResolutionRange {
                optical: short(36),
                max: short(38),
                min: short(40),
            },
            boundary_x,
            boundary_y,
            frame_pitch: long(58),
            frames: data[71],
            focus: (short(72), short(74)),
        })
    }
}

/// Ask the device, before there is a scanner to ask
///
/// `allocation_length` is the caller's, since how much a unit answers with varies and a wire
/// change here is not something the tests would catch.
pub fn read<T: Transport + ?Sized>(
    transport: &mut T,
    allocation_length: u8,
) -> Result<Capabilities, scsi::Error> {
    let page = transport.send(&VpdInquiry::new(PAGE, allocation_length))?;
    if page.page_code != PAGE {
        return Err(scsi::Error::InvalidResponse(
            "device answered the capability request with a different page",
        ));
    }
    Capabilities::parse(&page.data)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Captured pages from two different units, so the shared offsets stay pinned to real
    /// hardware rather than to whichever caller was refactored last
    fn ls9000ed_page() -> Vec<u8> {
        hex(
            "01 00 3B 00 0F 00 00 01 00 01 01 17 42 12 0F A0 0F A0 02 9A 00 00 23 03 \
             00 00 00 00 00 00 00 00 00 00 23 04 0F A0 0F A0 01 4D 00 00 87 54 00 00 \
             00 00 00 00 00 00 00 00 33 78 00 00 00 00 00 00 00 00 00 53 00 53 01 01 \
             00 00 01 C2 00 00 10 27 10 0C 03 00 53 00 1B",
        )
    }

    fn ls50ed_page() -> Vec<u8> {
        hex(
            "03 00 3A 00 0F 00 00 00 40 01 01 00 01 31 0F A0 0F A0 00 5A 00 00 00 00 \
             00 00 00 00 00 00 00 00 00 00 0F 6A 0F A0 0F A0 00 5A 00 00 BA 38 00 00 \
             00 00 00 00 00 00 00 00 17 47 00 00 17 47 00 00 00 00 00 61 00 61 06 06 \
             00 00 01 43 00 00 0E 0F 6A 00 01",
        )
    }

    fn hex(s: &str) -> Vec<u8> {
        s.split_whitespace()
            .map(|b| u8::from_str_radix(b, 16).unwrap())
            .collect()
    }

    #[test]
    fn parses_the_ls9000ed_page() {
        let caps = Capabilities::parse(&ls9000ed_page()).unwrap();
        assert_eq!(caps.max_bits, 16);
        assert_eq!(caps.boundary_x, 8964);
        assert_eq!(caps.boundary_y, 13176);
        assert_eq!(caps.x_resolution.optical, 4000);
        assert_eq!(caps.focus, (0, 450));
    }

    #[test]
    fn parses_the_ls50ed_page() {
        let caps = Capabilities::parse(&ls50ed_page()).unwrap();
        assert_eq!(caps.max_bits, 14);
        assert_eq!(caps.boundary_x, 3946);
        assert_eq!(caps.boundary_y, 5959);
        assert_eq!(caps.frame_pitch, 5959);
        assert_eq!(caps.x_resolution.optical, 4000);
    }

    /// `max_x`/`max_y` subtract one from these, so a zero would wrap into a window the size of
    /// the address space rather than failing anywhere near the bad page
    #[test]
    fn a_zero_boundary_is_not_a_usable_page() {
        for offset in [32, 54] {
            let mut page = ls50ed_page();
            page[offset..offset + 4].fill(0);
            assert!(Capabilities::parse(&page).is_err(), "zero at {offset}");
        }
    }

    #[test]
    fn a_short_page_is_rejected() {
        assert!(Capabilities::parse(&ls50ed_page()[..78]).is_err());
    }
}
