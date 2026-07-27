//! What the scanner says it can do, from vital product data page 0xC1
//!
//! The page layout and its parsing are shared, in
//! [`nikon::capabilities`](crate::scanners::nikon::capabilities). Only the allocation length is
//! this driver's.

use crate::scanners::nikon::capabilities::{self as nikon, Capabilities};
use crate::scsi::{self, Transport};

/// As much as the field carries. This unit answers with 83 bytes.
const ALLOCATION_LENGTH: u8 = 0xFF;

/// Ask the device, before there is a scanner to ask
pub(super) fn read<T: Transport + ?Sized>(transport: &mut T) -> Result<Capabilities, scsi::Error> {
    nikon::read(transport, ALLOCATION_LENGTH)
}

#[cfg(test)]
pub(super) mod fixture {
    /// The whole response, header included, as a mock transport has to answer it
    pub fn raw_page() -> Vec<u8> {
        let body = super::tests::captured();
        let mut raw = vec![0x06, crate::scanners::nikon::capabilities::PAGE];
        raw.extend_from_slice(&(body.len() as u16).to_be_bytes());
        raw.extend_from_slice(&body);
        raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The page as an LS-9000 ED with a strip holder loaded reports it
    pub(super) fn captured() -> Vec<u8> {
        let hex = "01 00 3B 00 0F 00 00 01 00 01 01 17 42 12 0F A0 0F A0 02 9A 00 00 23 03 \
                   00 00 00 00 00 00 00 00 00 00 23 04 0F A0 0F A0 01 4D 00 00 87 54 00 00 \
                   00 00 00 00 00 00 00 00 33 78 00 00 00 00 00 00 00 00 00 53 00 53 01 01 \
                   00 00 01 C2 00 00 10 27 10 0C 03 00 53 00 1B";
        hex.split_whitespace()
            .map(|b| u8::from_str_radix(b, 16).unwrap())
            .collect()
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
}
