//! What the scanner says it can do, from vital product data page 0xC1
//!
//! Most of this was hardcoded from captures. Reading it means the geometry the library will
//! accept comes from the device, which matters most for
//! [`boundary_y`](Capabilities::boundary_y).

use crate::scsi::{self, Transport, TransportExt, cdbs::VpdInquiry};

/// The page code this lives on
const PAGE: u8 = 0xC1;

/// Device-reported limits and geometry
///
/// Field offsets are into the page body, after the four-byte VPD header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    /// Bits per sample the scanner produces
    pub max_bits: u8,
    /// Native sensor resolution, and the range it will divide down to
    pub x_resolution: ResolutionRange,
    /// Same along stage travel. The minimum is the 333 a preview steps at.
    pub y_resolution: ResolutionRange,
    /// Widest window along the sensor bar, in 1/4000-in dots
    pub boundary_x: u32,
    /// Longest frame window, in 1/4000-in dots. This should be the length of a 6x9 frame
    /// Weirdly, if you set a frame longer than this (say to try to scan 6x12), you crash the stage motor into its endstop
    pub boundary_y: u32,
    /// Frames the device reports for the loaded holder. Reads 1 with a strip holder in, which
    /// a strip does not have, so this is not the frame count.
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
    /// Ask the device, before there is a scanner to ask
    pub(super) fn read<T: Transport + ?Sized>(transport: &mut T) -> Result<Self, scsi::Error> {
        Self::parse(&transport.send(&VpdInquiry::new(PAGE, 0xFF))?.data)
    }

    fn parse(data: &[u8]) -> Result<Self, scsi::Error> {
        // The last field we read starts at 78, so anything shorter is not this page
        if data.len() < 79 {
            return Err(scsi::Error::InvalidResponse(
                "capability page shorter than its known fields",
            ));
        }
        let short = |at: usize| u16::from_be_bytes([data[at], data[at + 1]]);
        let long =
            |at: usize| u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);

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
            boundary_x: long(32),
            boundary_y: long(54),
            frames: data[71],
            focus: (short(72), short(74)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The page as an LS-9000 ED with a strip holder loaded reports it
    fn captured() -> Vec<u8> {
        let hex = "01 00 3B 00 0F 00 00 01 00 01 01 17 42 12 0F A0 0F A0 02 9A 00 00 23 03 \
                   00 00 00 00 00 00 00 00 00 00 23 04 0F A0 0F A0 01 4D 00 00 87 54 00 00 \
                   00 00 00 00 00 00 00 00 33 78 00 00 00 00 00 00 00 00 00 53 00 53 01 01 \
                   00 00 01 C2 00 00 10 27 10 0C 03 00 53 00 1B";
        hex.split_whitespace()
            .map(|b| u8::from_str_radix(b, 16).unwrap())
            .collect()
    }

    #[test]
    fn parses_the_captured_page() {
        let caps = Capabilities::parse(&captured()).unwrap();

        assert_eq!(caps.max_bits, 16);
        assert_eq!(caps.boundary_x, 8964);
        assert_eq!(caps.boundary_y, 13176);
        assert_eq!(caps.focus, (0, 450));

        assert_eq!(caps.x_resolution.optical, 4000);
        assert_eq!(caps.x_resolution.max, 4000);
        assert_eq!(caps.x_resolution.min, 666);

        assert_eq!(caps.y_resolution.optical, 4000);
        assert_eq!(caps.y_resolution.max, 4000);
        assert_eq!(caps.y_resolution.min, 333);
    }

    /// Every focus value read off real hardware sits inside the reported range
    #[test]
    fn observed_focus_values_are_in_the_reported_range() {
        let (min, max) = Capabilities::parse(&captured()).unwrap().focus;
        for focus in [189u16, 190, 195, 200, 207, 217, 226] {
            assert!(focus >= min && focus <= max, "{focus} outside {min}..{max}");
        }
    }

    /// Both window resolutions the library sends have to be ones the device offers
    #[test]
    fn the_resolutions_we_send_are_offered() {
        let caps = Capabilities::parse(&captured()).unwrap();
        assert!(caps.x_resolution.min <= 666 && 4000 <= caps.x_resolution.max);
        assert!(caps.y_resolution.min <= 333 && 4000 <= caps.y_resolution.max);
    }

    #[test]
    fn a_short_page_is_an_error() {
        assert!(Capabilities::parse(&[0u8; 40]).is_err());
    }
}
