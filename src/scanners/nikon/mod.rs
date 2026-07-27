//! What the Nikon Coolscans have in common
//!
//! Nikon-specific but not model-specific: the vendor CDB framing, the capability page, the
//! frame table. Anything whose *encoding* differs between models stays in that model's module
//! even when it shares a name here, because the wire formats disagree in ways that would be
//! silent if merged.

pub mod capabilities;
pub mod cdbs;

/// A millimeter figure in device dots
///
/// `dots_per_inch` is the measurement unit the driver set at open, which is the same number the
/// mode page divides the inch by. It is not the scanner's optical resolution and not the same on
/// every model, so it is a parameter rather than a constant. Negative input floors at zero: a
/// window cannot start before the film does.
pub fn native_dots(millimeters: f32, dots_per_inch: u32) -> u32 {
    const MM_PER_INCH: f32 = 25.4;
    (millimeters * dots_per_inch as f32 / MM_PER_INCH)
        .round()
        .max(0.0) as u32
}

/// The per-channel analog gain out of a window descriptor's vendor tail
///
/// The last four bytes of the ten, big-endian. `None` if the tail is short, which is not the
/// same as a gain of zero: zero is a value a caller would go on to arm a black pass with.
pub fn exposure_from_vendor(vendor: &[u8]) -> Option<u32> {
    vendor
        .get(6..10)
        .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}
