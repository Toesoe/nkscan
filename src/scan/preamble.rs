//! The one-time setup a session owes the unit before it scans
//!
//! Nikon Scan runs this once after opening and before the first pass, and
//! nothing in it is a scan. The captures are the only description of it: no
//! section of either spec says it is needed, and the unit accepts commands
//! without it. What it appears to buy is a mechanism in a known state.

use crate::{
    error::Error,
    protocol::{
        caps::other::DataTypes,
        data::{DataType, Op},
    },
    session::Session,
};
use tracing::*;

/// Put the unit into the state Nikon Scan leaves it in before its first scan
///
/// Every step is gated on what the unit advertises, and each is skipped rather
/// than fatal where it is not offered: this whole sequence is undocumented, so
/// refusing to work without it would be worse than going ahead.
pub fn run(session: &mut Session) -> Result<(), Error> {
    // The CCD's own response curves. The captures read these per channel
    // before anything else
    if offers(session, DataTypes::CCD_DATA_READ) {
        for color in 0..=3 {
            if let Err(e) = session.read_data(DataType::CcdData, color) {
                debug!(color, %e, "no CCD data for this channel");
                break;
            }
        }
    }

    // Every window the unit holds, written back as it stands. A SET WINDOW is
    // what moves the mechanism on this family, so this is the stage being put
    // somewhere known rather than the descriptors being changed
    //
    // A descriptor the unit hands back is not necessarily one it will take:
    // the power-on defaults reach past the holder aperture, and writing one
    // back is answered with common error 1. Skip those rather than stop
    let held = session.windows()?;
    for w in &held {
        if let Err(e) = session.set_window(w) {
            debug!(id = w.id, %e, "this window would not go back");
        }
    }

    // Where the lens is, and whether the unit focuses on its own schedule
    let position = session.get_parameter(Op::FocusMove).ok();
    if let Ok(auto) = session.get_parameter(Op::AutoAf) {
        debug!(on = auto.first, "automatic autofocus");
    }

    // Drive the lens to where it already is. The captures do exactly this, and
    // it is the only thing here that could be initializing the focus mechanism
    if let Some(params) = position {
        let at = params.first.min(u32::from(u16::MAX)) as u16;
        match session.focus_to(at) {
            Ok(()) => debug!(at, "staged the focus"),
            Err(e) => debug!(at, %e, "could not stage the focus"),
        }
    }

    // Read the frame boundaries and write them straight back
    if offers(session, DataTypes::BOUNDARY_READ) && offers(session, DataTypes::BOUNDARY_WRITE) {
        match session.boundaries() {
            Ok(boundary) => {
                debug!(frames = boundary.frames.len(), "returning the boundary");
                session.set_boundaries(&boundary)?;
            }
            Err(e) => debug!(%e, "no boundary to return"),
        }
    }

    debug!(windows = held.len(), "preamble done");
    Ok(())
}

fn offers(session: &Session, bit: DataTypes) -> bool {
    session.capabilities().features.data_types.contains(bit)
}
