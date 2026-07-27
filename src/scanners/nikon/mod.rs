//! What the Nikon Coolscans have in common
//!
//! Nikon-specific but not model-specific: the vendor CDB framing, the capability page, the
//! frame table. Anything whose *encoding* differs between models stays in that model's module
//! even when it shares a name here, because the wire formats disagree in ways that would be
//! silent if merged.

pub mod capabilities;
pub mod cdbs;

/// The per-channel analog gain out of a window descriptor's vendor tail
///
/// The last four bytes of the ten, big-endian. `None` if the tail is short, which is not the
/// same as a gain of zero: zero is a value a caller would go on to arm a black pass with.
pub fn exposure_from_vendor(vendor: &[u8]) -> Option<u32> {
    vendor
        .get(6..10)
        .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}
