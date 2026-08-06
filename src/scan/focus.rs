//! Deciding where to focus
//!
//! `session` has the operations. This picks the point they get pointed at.

use crate::{
    error::Error,
    protocol::{
        caps::address::Axis,
        sense::{Failure, Fault},
        window::Window,
    },
    session::Session,
};
use tracing::*;

/// How focusing went
///
/// Not reaching focus is a recovered error, sense key 01h, so the command
/// finished and the lens is wherever it ended up. Worth knowing about, not
/// worth refusing to scan over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focused {
    /// The unit reached focus, or was driven to a position
    Yes,
    /// Autofocus ran and did not converge. Usually means it was pointed at
    /// something with no detail to focus on
    NotReached,
    /// Nothing was asked of it
    Skipped,
}

/// What to do about focus before a scan
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    /// Let the unit focus on a point of the window, given as a fraction of its
    /// size. `(0.5, 0.5)` is the middle
    ///
    /// `color` needs the unit to offer `A1h`. `None` uses `A0h` and lets it
    /// choose the channel.
    Auto { at: (f32, f32), color: Option<u8> },
    /// Drive the lens to an absolute position, bounded by `C1h` bytes 76-79
    At(u16),
    /// Leave the focus wherever it is
    Hold,
}

impl Default for Focus {
    /// The middle of the window, which is what Nikon Scan focuses on
    fn default() -> Self {
        Self::Auto {
            at: (0.5, 0.5),
            color: None,
        }
    }
}

impl Focus {
    /// Focus for a scan of `windows`
    ///
    /// The address is worked out from the first window. A set has to agree on
    /// geometry, so any of them would do.
    pub fn apply(self, session: &mut Session, windows: &[Window]) -> Result<Focused, Error> {
        let Some(window) = windows.first() else {
            return Ok(Focused::Skipped);
        };

        match self {
            Self::Hold => Ok(Focused::Skipped),
            Self::At(position) => session.focus_to(position).map(|()| Focused::Yes),
            Self::Auto { at, color } => {
                let caps = session.capabilities();
                let point = |axis: &Axis, origin: u32, size: u32, fraction: f32| {
                    let offset = (size as f32 * fraction.clamp(0.0, 1.0)) as u32;
                    // The window may reach past what an address can name, and
                    // the sensor is wider than the holder opening
                    (origin + offset).clamp(axis.address_range.start, axis.address_range.last)
                };
                let x = point(&caps.address.x_axis, window.origin.0, window.size.0, at.0);
                let y = point(&caps.address.y_axis, window.origin.1, window.size.1, at.1);

                debug!(x, y, "focusing");
                match session.autofocus(x, y, color) {
                    Ok(()) => Ok(Focused::Yes),
                    Err(Error::Device(fault))
                        if matches!(*fault, Fault::Reported(Failure::OutOfFocus, _)) =>
                    {
                        warn!(x, y, "autofocus did not reach focus");
                        Ok(Focused::NotReached)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_is_the_middle_of_the_window() {
        assert_eq!(
            Focus::default(),
            Focus::Auto {
                at: (0.5, 0.5),
                color: None
            }
        );
    }
}
