//! Focusing, which is EXECUTE operations 91h, A0h, A1h and C1h. Section 2-15-4

use super::Session;
use crate::{
    error::Error,
    protocol::{
        caps::other::HostCooperation,
        data::{Op, Operation},
    },
};
use std::time::Duration;

/// A focus move drives the lens alone and settles in seconds
const FOCUS_TIMEOUT: Duration = Duration::from_secs(60);

/// Autofocus takes an address on the medium, and the sub-scanning half of that
/// is the feed, so reaching it can move the stage
const AUTOFOCUS_TIMEOUT: Duration = Duration::from_secs(180);

/// [`Session::execute`] checks this too, but checking before the arguments means a unit that cannot do the thing says so, rather than faulting a coordinate
fn offers(session: &Session, operation: Op) -> Result<(), Error> {
    if session.capabilities().features.execute.supports(operation) {
        return Ok(());
    }
    Err(Error::Unsupported {
        op: "execute operation",
        reason: format!("this unit does not offer {operation:?}"),
    })
}

impl Session {
    /// Focus on a point of the medium
    ///
    /// The address is the one a window origin uses, whatever 2-15 means by
    /// calling it an address on the medium: the captures focus a window at top
    /// 10512 length 6696 at 13860, its center.
    ///
    /// Some addresses inside the range are still answered instantly with out of
    /// focus, having moved nothing. What bounds that is not yet known, so this
    /// only checks the range the axis reports.
    ///
    /// `color` picks the channel, which needs the unit to offer A1h; `None`
    /// uses A0h and lets it choose.
    pub fn autofocus(&mut self, x: u32, y: u32, color: Option<u8>) -> Result<(), Error> {
        let operation = match color.is_some() {
            true => Op::ColorAutoFocus,
            false => Op::AutoFocus,
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
        offers(self, Op::FocusMove)?;
        let range = self.capabilities().address.focus_range;
        if position < range.start || position > range.last {
            return Err(Error::Unsupported {
                op: "focus position",
                reason: format!("{position} is outside {} to {}", range.start, range.last),
            });
        }

        self.execute(
            Op::FocusMove,
            Operation {
                first: u32::from(position),
                ..Operation::default()
            },
            FOCUS_TIMEOUT,
        )
    }

    /// Let the unit focus itself when it decides it needs to
    pub fn set_auto_focus(&mut self, on: bool) -> Result<(), Error> {
        offers(self, Op::AutoAf)?;
        self.execute(
            Op::AutoAf,
            Operation {
                first: u32::from(on),
                ..Operation::default()
            },
            FOCUS_TIMEOUT,
        )
    }
}
