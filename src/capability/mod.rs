//! What a scanner can do, as data rather than as a trait per option
//!
//! One row per option Nikon Scan offered. Most of them are decided by the loaded adapter rather
//! than by the model — eject, thumbnails, batch and strip offset all vary across the adapters a
//! single body takes — so this is computed from the pair, in [`table`].
//!
//! Everything here is a plain value. Adding a model is a row in [`table`], not a new trait and
//! not a new impl, and the only place a model name sits next to a behavior is that table.
//!
//! Two things are deliberately *not* booleans. Eject is an [`EjectAction`], because the same
//! control returns a holder on one body, rewinds a cartridge on another and swaps slides on a
//! third. Exposure is an [`ExposureControl`], because "no white balance lock" and "no
//! autoexposure" are different claims and one model makes only the first.

use crate::adapter::Adapter;
use crate::model::{Interface, Model};
use crate::scanners::nikon::limits::DeviceLimits;

pub mod resolve;
pub mod table;
pub mod unsupported;

/// What a scanner can do, for one model with one adapter loaded
#[derive(Debug, Clone, PartialEq)]
pub struct Capabilities {
    pub model: Model,
    pub adapter: Adapter,

    // --- decided by the model
    pub interface: Interface,
    pub resolution: Resolution,
    pub depth: Depth,
    /// Repeat counts the model averages in hardware, `[1]` where it drives none
    pub multisample: &'static [u8],
    /// Readout modes a caller may choose between, empty where the model offers no choice
    pub ccd_modes: &'static [CcdMode],
    pub ice: Ice,
    pub focus: FocusSupport,
    pub exposure: ExposureControl,

    // --- decided by the adapter
    pub eject: EjectAction,
    pub overview: Overview,
    pub frames: FrameLocation,
    /// Whether a whole holder or roll can be run without a person in between
    pub batch: bool,
    /// Whether the adapter can shift film to line its frames up with the scan positions
    pub strip_offset: bool,
    pub max_area_mm: (f32, f32),
}

/// What the eject control physically does on this adapter
///
/// Not a boolean, which is the whole point: a caller that only knows "can eject" cannot tell a
/// holder being handed back from a cartridge being rewound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EjectAction {
    /// Nothing to eject: the film is placed and taken out by hand
    Unavailable,
    /// Puts the film holder out of the body
    EjectHolder,
    /// Pushes the film back out of the adapter
    EjectFilm,
    /// Winds the cartridge back in
    RewindFilm,
    /// Returns the slide in the gate and feeds the next one
    FeedNextSlide,
}

/// Whether this adapter supports the low-resolution pass Nikon Scan calls a thumbnail
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overview {
    Unavailable,
    Available,
}

/// How the frame positions are known
///
/// Separate from [`Overview`] on purpose. The two are orthogonal: a mounted-slide adapter has
/// fixed positions and no overview, and a strip feeder has an overview and unfixed positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameLocation {
    /// Mechanically fixed apertures, this many of them
    Mechanical(u8),
    /// Not fixed: they have to be found in an overview, or placed by the caller
    Detected,
    /// The transport senses them and reports a table
    Reported,
    /// One frame at a time
    Single,
}

/// Where the gain for a pass is decided
///
/// The LS-50's firmware meters and hands back what it settled on, which is *why* it has no white
/// balance lock — a different fact from having no autoexposure. A refusal cites this rather than
/// naming a model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExposureControl {
    /// The host meters from a preview pass, and can hold the channel ratios where noted
    Host { lock_white_balance: bool },
    /// The firmware meters. Fixing the gain is the only way to hold the ratios.
    Firmware,
}

/// The sensor readout mode, where a model lets one be chosen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CcdMode {
    /// All three sensor lines, the faster readout
    ThreeLine,
    /// One line, slower, and it reduces the banding the three-line readout can show
    SingleLine,
}

/// Bits per sample
///
/// The wire is always 16 regardless; anything narrower is a host-side conversion, which is why
/// `offered` can be wider than what the sensor actually resolves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Depth {
    pub native: u8,
    pub offered: &'static [u8],
}

/// The sensor's own pitch, and the divisions the firmware offers
///
/// `optical` is not 4000 on every model — the LS-40 is 2900 — so nothing may derive a resolution
/// by dividing a literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub optical: u16,
    pub ladder: Vec<u16>,
}

/// Digital ICE
///
/// Advisory. ICE is a host-side algorithm over the infrared plane rather than a device mode: this
/// library captures infrared and something else runs the algorithm. The row says which variant
/// Nikon Scan would have offered, so a consumer can pick the same one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ice {
    pub infrared: bool,
    pub kodachrome: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusSupport {
    pub auto: bool,
    /// The setpoint range the unit reports, `None` until one is open
    pub range: Option<(u16, u16)>,
}

impl Capabilities {
    /// What this model can do before anything is open, with nothing loaded
    pub fn of(model: Model) -> Self {
        table::compute(model, Adapter::None)
    }

    /// Fold in what an open unit reports about itself
    ///
    /// The table is what the model and adapter imply; this is what the unit in front of you
    /// actually answers, and it wins wherever the two disagree. `ladder` is the driver's own
    /// resolution rungs, filtered here to the ones the device says it will divide to.
    pub(crate) fn refine(mut self, limits: &DeviceLimits, ladder: &[u16]) -> Self {
        self.resolution.optical = limits.x_resolution.optical;
        self.resolution.ladder = ladder
            .iter()
            .copied()
            .filter(|&dpi| limits.x_resolution.allows(dpi))
            .collect();
        self.depth.native = limits.max_bits;
        self.focus.range = Some(limits.focus);
        // Against the reported optical pitch rather than a hardcoded one, which is what makes
        // this right on a body whose sensor is not 4000 DPI
        self.max_area_mm = (
            dots_to_mm(limits.boundary_x, limits.x_resolution.optical),
            dots_to_mm(limits.boundary_y, limits.y_resolution.optical),
        );
        self
    }

    /// What to call the loaded adapter
    ///
    /// The part number where the model pins one down, since that is what is printed on the object
    /// in the user's hand, and a description of what it is otherwise.
    pub fn adapter_name(&self) -> String {
        self.adapter
            .part_number(self.model)
            .map(str::to_owned)
            .unwrap_or_else(|| self.adapter.to_string())
    }

    /// Whether this unit offers `dpi` at all
    pub fn allows_dpi(&self, dpi: u16) -> bool {
        self.resolution.ladder.contains(&dpi)
    }

    /// Whether a caller may ask for this readout mode
    pub fn allows_ccd_mode(&self, mode: CcdMode) -> bool {
        self.ccd_modes.contains(&mode)
    }
}

/// A length in a scanner's own dots, as millimeters
///
/// Takes the pitch rather than assuming it. Every driven model works in 1/4000-inch dots, but the
/// LS-40's sensor is 2900 DPI and assuming otherwise misreports every area by 38%.
pub fn dots_to_mm(dots: u32, optical_dpi: u16) -> f32 {
    dots as f32 * 25.4 / f32::from(optical_dpi.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dots_convert_against_the_pitch_they_were_measured_at() {
        // A 4000 DPI body's full 35 mm width
        assert!((dots_to_mm(3946, 4000) - 25.05).abs() < 0.01);
        // The same dot count on a 2900 DPI body is a longer distance, not the same one
        assert!((dots_to_mm(3946, 2900) - 34.56).abs() < 0.01);
    }

    /// Nothing may divide by a pitch of zero, however malformed the page was
    #[test]
    fn a_zero_pitch_does_not_divide_by_zero() {
        assert!(dots_to_mm(100, 0).is_finite());
    }
}
