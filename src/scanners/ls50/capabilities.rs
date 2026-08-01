//! This model's read of the shared capability page
//!
//! The page and its parsing are in
//! [`nikon::limits_usb`](crate::scanners::nikon::limits_usb). The LS-50 and the LS-5000 answer
//! it identically apart from the converter width, which the device reports at runtime; the
//! width appears here only so the test fixture reproduces this model rather than the other one.

pub(super) use crate::scanners::nikon::limits_usb::{read, read_sensed_frames};

#[cfg(test)]
pub(super) mod fixture {
    use crate::scanners::nikon::limits::DeviceLimits;
    use crate::scanners::nikon::limits_usb::fixture as shared;

    /// This model's converter is narrower than the LS-5000's
    const MAX_BITS: u8 = 14;

    pub fn capabilities() -> DeviceLimits {
        shared::capabilities(MAX_BITS)
    }

    pub fn raw_page() -> Vec<u8> {
        shared::raw_page(MAX_BITS)
    }
}

#[cfg(test)]
mod tests {
    /// The one thing this module binds, and the reason the two models cannot share a fixture
    #[test]
    fn the_fixture_reports_this_models_converter_width() {
        assert_eq!(super::fixture::capabilities().max_bits, 14);
    }
}
