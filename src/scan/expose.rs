//! Running an exposure pass
//!
//! [`Exposure`] says which mechanism to use. This does it and hands back the
//! same windows with their exposures filled in.

use super::Exposure;
use crate::{
    error::Error,
    protocol::{
        caps::set_window::ScanMode,
        window::{IR, Window},
    },
    session::Session,
};
use std::time::Duration;
use tracing::*;

/// Long enough for a low-resolution pass over a whole frame
const PASS_TIMEOUT: Duration = Duration::from_secs(300);

/// Meter `windows` and answer them with new exposures
///
/// Leaves everything else about them alone, so the caller can set them and
/// scan. Nothing is left running: the pass is read out or aborted before this
/// returns, since a scan whose data was never read locks out the next command.
pub fn expose(
    session: &mut Session,
    windows: &[Window],
    exposure: Exposure,
) -> Result<Vec<Window>, Error> {
    match exposure {
        Exposure::Unit(kind) => {
            // The unit meters during a pass of its own. It writes the result
            // into the descriptors, so GET WINDOW is what reports it
            let mut pass = windows.to_vec();
            for w in &mut pass {
                w.scanning_kind = kind;
            }
            run(session, &pass)?;
            session.abort()?;

            let held = session.windows()?;
            Ok(windows
                .iter()
                .map(|w| {
                    let mut w = w.clone();
                    if let Some(metered) = held.iter().find(|h| h.id == w.id) {
                        w.exposure = metered.exposure;
                    }
                    w
                })
                .collect())
        }

        Exposure::Host(metering) => {
            // Exposures persist in the unit across sessions, so metering from
            // whatever is in the descriptors compounds run over run. 8Ch is the
            // unit's own neutral, measured at start-up, so we start there every
            // time. Locking needs it to mean anything at all, and unlocked it
            // still saves the extra pass a stale exposure would cost by clipping
            let seeded = seed_white_balance(session, windows)?;
            let pass = prescan_windows(session, &seeded);
            let layout = run(session, &pass)?;

            let mut raw = vec![0u8; layout.total_bytes() as usize];
            let got = session.read_image(&layout, &mut raw)?;
            raw.truncate(got);
            debug!(bytes = got, "metering pass");

            let exposures = metering.apply(session.capabilities(), &layout, &raw, &pass)?;
            Ok(seeded
                .iter()
                .zip(exposures)
                .map(|(w, exposure)| Window {
                    exposure,
                    ..w.clone()
                })
                .collect())
        }
    }
}

/// The same windows with the unit's start-up white balance in them
///
/// Only the visible channels: 2-11-3's qualifier has no infrared, so an IR
/// window keeps the exposure it came with.
fn seed_white_balance(session: &mut Session, windows: &[Window]) -> Result<Vec<Window>, Error> {
    let wb = session.white_balance()?;
    Ok(windows
        .iter()
        .map(|w| {
            let mut w = w.clone();
            if let Some(&exposure) = usize::from(w.id).checked_sub(1).and_then(|n| wb.get(n)) {
                w.exposure = exposure;
            }
            w
        })
        .collect())
}

/// The same windows, shrunk to something quick to take and read
///
/// Lowest resolution the unit offers, high speed if it has it, and no
/// multisampling. Anything past that only costs time, and multisampling or
/// multi-line reading would make the pass ask us for post-processing first.
fn prescan_windows(session: &Session, windows: &[Window]) -> Vec<Window> {
    let caps = session.capabilities();
    let dpi = caps.address.x_axis.dpi_range.start;
    let fast = caps.set_window.mode.contains(ScanMode::HIGH_SPEED);

    windows
        .iter()
        .map(|w| {
            let mut w = w.clone();
            w.resolution = (dpi, dpi);
            w.multiple_reading = 0;
            if fast {
                w.scanning_mode = ScanMode::HIGH_SPEED;
            }
            w
        })
        .collect()
}

/// Set every window, scan them, and wait for the pass to finish
fn run(session: &mut Session, windows: &[Window]) -> Result<crate::protocol::image::Layout, Error> {
    for w in windows {
        session.set_window(w)?;
    }
    let ids: Vec<u8> = windows.iter().map(|w| w.id).collect();
    debug!(?ids, "exposure pass");

    let layout = session.scan(windows)?;
    session.test_unit_ready(PASS_TIMEOUT)?;
    Ok(layout)
}

/// Whether a set has anything worth metering
///
/// Infrared measures obstructions, so a set of nothing else has no exposure to
/// decide from a film's tones.
pub fn meterable(windows: &[Window]) -> bool {
    windows.iter().any(|w| w.id != IR)
}
