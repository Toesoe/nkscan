//! High-level scanning operations
//!
//! This is where we work out what a scan should do, from what the unit says it
//! can do, and then order the session calls to do it. Checks that a single
//! argument is legal stay down in `session`; picking between two mechanisms
//! happens here, once, before anything moves.

pub mod expose;
pub mod focus;
pub mod framing;
pub mod meter;
pub mod thumbnail;

pub use expose::expose;
pub use focus::Focus;
pub use framing::Framing;
pub use meter::Metering;
pub use thumbnail::Thumbnail;

use crate::{
    error::Error,
    protocol::caps::{
        Capabilities,
        set_window::{AnalogControl, ScanKind},
    },
};

/// How the exposures get decided
///
/// `D1h` byte 4 says whether the unit will meter for itself. If neither AE bit
/// is set, we do it. There is no host-cooperation bit for this in `E1h` the way
/// there is for autofocus, so the missing scan kind is the only signal.
#[derive(Debug, Clone, Copy)]
pub enum Exposure {
    /// The unit meters itself. This is a scanning kind, so it goes in the
    /// window descriptor
    Unit(ScanKind),
    /// We take an ordinary pass and work the exposures out from it
    Host(Metering),
}

impl Exposure {
    /// Pick whichever mechanism this unit has
    ///
    /// Hardware AE is cheaper: one pass instead of a prescan plus our own
    /// arithmetic. `AE` equalizes the channels and `AE_WB` keeps them in
    /// proportion, so only the latter can serve `lock_white_balance`.
    pub fn choose(caps: &Capabilities, lock_white_balance: bool) -> Result<Self, Error> {
        let kinds = caps.set_window.kind;

        if lock_white_balance && kinds.contains(ScanKind::AE_WB) {
            return Ok(Self::Unit(ScanKind::AE_WB));
        }
        if !lock_white_balance && kinds.contains(ScanKind::AE) {
            return Ok(Self::Unit(ScanKind::AE));
        }

        // We meter by moving the exposure in the descriptor, so the unit has to
        // offer that as an analog control. D1h byte 14
        let aic = caps.set_window.aic;
        if !aic.intersects(AnalogControl::EXPOSURE_VALUE | AnalogControl::EXPOSURE_TIME) {
            return Err(Error::Unsupported {
                op: "exposure",
                reason: format!(
                    "this unit runs no AE pass and offers no exposure control, only {aic:?}"
                ),
            });
        }

        Ok(Self::Host(Metering {
            lock_white_balance,
            ..Metering::default()
        }))
    }
}
