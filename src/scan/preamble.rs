//! The one-time setup a session owes the unit before it scans
//!
//! Nikon Scan runs this once after opening and before the first pass, and
//! nothing in it is a scan. No section of either spec says it is needed,
//! and the unit accepts commands without it.
//! What it appears to buy is a mechanism in a known state.

use crate::{
    error::Error,
    protocol::{
        caps::{Capabilities, other::DataTypes},
        data::{Boundary, DataType, Op},
    },
    session::Session,
};
use tracing::*;

/// Run the canonical Nikon Scan preamble.
///
/// `frames` is where the caller believes the frames are, from [`framing::table`](super::framing::table)
pub fn run(session: &mut Session, frames: &Boundary) -> Result<(), Error> {
    // The CCD's own response curves. The captures read these per channel
    // before anything else
    if offers(session.capabilities(), DataTypes::CCD_DATA_READ) {
        for color in 0..=3 {
            if let Err(e) = session.read_data(DataType::CcdData, color) {
                debug!(color, %e, "no CCD data for this channel");
                break;
            }
        }
    }

    let held = session.windows()?;
    for w in &held {
        if let Err(e) = session.set_window(w) {
            debug!(id = w.id, %e, "this window would not go back");
        }
    }

    // Where the focus motor is, and whether the unit focuses on its own schedule
    let position = session.get_parameter(Op::FocusMove).ok();
    if let Ok(auto) = session.get_parameter(Op::AutoAf) {
        debug!(on = auto.first, "automatic autofocus");
    }

    // Drive the focus to where it already is
    if let Some(params) = position {
        let at = params.first.min(u32::from(u16::MAX)) as u16;
        match session.focus_to(at) {
            Ok(()) => debug!(at, "staged the focus"),
            Err(e) => debug!(at, %e, "could not stage the focus"),
        }
    }

    // Say where the frames are
    if offers(session.capabilities(), DataTypes::BOUNDARY_READ)
        && offers(session.capabilities(), DataTypes::BOUNDARY_WRITE)
    {
        match session.boundaries() {
            Ok(held) => debug!(frames = held.frames.len(), "the table the unit held"),
            Err(e) => debug!(%e, "no boundary to read"),
        }
        if !frames.frames.is_empty() {
            debug!(frames = frames.frames.len(), "sending the frame table");
            session.set_boundaries(frames)?;
        }
    }

    debug!(windows = held.len(), "preamble done");
    Ok(())
}

fn offers(caps: &Capabilities, bit: DataTypes) -> bool {
    caps.features.data_types.contains(bit)
}
