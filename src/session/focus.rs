//! Focusing, which is EXECUTE operations 91h, A0h, A1h and C1h. Section 2-15-4

use super::Session;
use crate::{
    error::Error,
    protocol::{caps::other::HostCooperation, data::Operation},
};
use std::time::Duration;

/// Auto Focus, 2-15-4 A0h
const AUTO_FOCUS: u8 = 0xA0;
/// Color oriented Auto Focus, A1h. Takes the same addresses plus a channel
const COLOR_AUTO_FOCUS: u8 = 0xA1;
/// Automatic AF execution, 91h. First setting value is 0 off, 1 on
const AUTO_AF: u8 = 0x91;
/// Focus Move, C1h. Moves the scan block in the AF direction
const FOCUS_MOVE: u8 = 0xC1;

/// A focus move drives the lens alone and settles in seconds
const FOCUS_TIMEOUT: Duration = Duration::from_secs(60);

/// Autofocus takes an address on the medium, and the sub-scanning half of that
/// is the feed, so reaching it can move the stage
const AUTOFOCUS_TIMEOUT: Duration = Duration::from_secs(180);

/// [`Session::execute`] checks this too, but checking before the arguments means a unit that cannot do the thing says so, rather than faulting a coordinate
fn offers(session: &Session, operation: u8) -> Result<(), Error> {
    if session.capabilities().features.execute.supports(operation) {
        return Ok(());
    }
    Err(Error::Unsupported {
        op: "execute operation",
        reason: format!("this unit does not offer {operation:02X}h"),
    })
}

impl Session {
    /// Focus on a point of the medium
    ///
    /// The address is in the same pixels a window origin uses, and is bounded by
    /// what `C1h` reports for each axis. `color` picks the channel to focus on,
    /// which needs the unit to offer A1h; `None` uses A0h and lets it choose.
    pub fn autofocus(&mut self, x: u32, y: u32, color: Option<u8>) -> Result<(), Error> {
        let operation = if color.is_some() {
            COLOR_AUTO_FOCUS
        } else {
            AUTO_FOCUS
        };
        offers(self, operation)?;

        // A unit that sets this expects the initiator to do the focusing, which is a different job to asking the unit to do it
        // I think all of our scanners have hardware AF
        let coop = self.capabilities().features.cooperation;
        if coop.contains(HostCooperation::AUTOFOCUS) {
            return Err(Error::Unsupported {
                op: "autofocus",
                reason: "this unit leaves focusing to the driver".into(),
            });
        }

        let caps = self.capabilities();
        for (axis, name, value) in [
            (&caps.address.x_axis, 'X', x),
            (&caps.address.y_axis, 'Y', y),
        ] {
            if value < axis.address_range.start || value > axis.address_range.last {
                return Err(Error::Unsupported {
                    op: "autofocus address",
                    reason: format!(
                        "{name} {value} is outside {} to {}",
                        axis.address_range.start, axis.address_range.last
                    ),
                });
            }
        }

        self.execute(
            operation,
            Operation {
                color: color.unwrap_or(0),
                first: x,
                second: y,
            },
            AUTOFOCUS_TIMEOUT,
        )
    }

    /// Move the scan block to an absolute focus position
    pub fn focus_to(&mut self, position: u16) -> Result<(), Error> {
        offers(self, FOCUS_MOVE)?;
        let range = self.capabilities().address.focus_range;
        if position < range.start || position > range.last {
            return Err(Error::Unsupported {
                op: "focus position",
                reason: format!("{position} is outside {} to {}", range.start, range.last),
            });
        }

        self.execute(
            FOCUS_MOVE,
            Operation {
                first: u32::from(position),
                ..Operation::default()
            },
            FOCUS_TIMEOUT,
        )
    }

    /// Let the unit focus itself when it decides it needs to
    pub fn set_auto_focus(&mut self, on: bool) -> Result<(), Error> {
        offers(self, AUTO_AF)?;
        self.execute(
            AUTO_AF,
            Operation {
                first: u32::from(on),
                ..Operation::default()
            },
            FOCUS_TIMEOUT,
        )
    }
}
