//! Running an exposure pass
//!
//! [`Exposure`] says which mechanism to use. This does it and hands back the
//! same windows with their exposures filled in.

use super::Metering;
use crate::{
    error::Error,
    protocol::{
        caps::{
            Capabilities,
            set_window::{AnalogControl, ScanKind, ScanMode},
        },
        window::{Flags, Window},
    },
    session::Session,
};
use std::time::Duration;
use tracing::*;

/// Long enough for a low-resolution pass over a whole frame
const PASS_TIMEOUT: Duration = Duration::from_secs(300);

/// How the exposures get decided
///
/// `SetWindowFunction` byte 4 says whether the unit will meter for itself. If
/// neither AE bit is set, we do it. There is no host-cooperation bit for this in
/// `Features` the way there is for autofocus, so the missing scan kind is the
/// only signal.
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
    pub fn choose(caps: &Capabilities, lock_white_balance: bool) -> Result<Self, Error> {
        let kinds = caps.set_window.kind;

        if lock_white_balance && kinds.contains(ScanKind::AE_WB) {
            return Ok(Self::Unit(ScanKind::AE_WB));
        }
        if !lock_white_balance && kinds.contains(ScanKind::AE) {
            return Ok(Self::Unit(ScanKind::AE));
        }

        // We meter by moving the exposure in the descriptor, so the unit has to
        // offer that as an analog control. `SetWindowFunction` byte 14
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
            // whatever is in the descriptors compounds run over run.
            // `DataType::WhiteBalanceExposure` is the unit's own neutral,
            // measured at start-up, so we start there every
            // time. Locking needs it to mean anything at all, and unlocked it
            // still saves the extra pass a stale exposure would cost by clipping
            let seeded = seed_white_balance(session, windows)?;

            // One proportional step lands a few percent under, so keep going
            // until a pass comes back on target rather than counting passes
            let mut pass = prescan_windows(session, &seeded);
            let mut layout;
            let mut raw;
            let mut n = 0;
            loop {
                layout = run(session, &pass)?;
                raw = vec![0u8; layout.total_bytes() as usize];
                let got = session.read_image(&layout, &mut raw)?;
                raw.truncate(got);
                n += 1;

                let settled = metering.settled(&layout, &raw)?;
                debug!(pass = n, bytes = got, settled, "metering pass");
                if settled {
                    break;
                }

                let next = metering.apply(session.capabilities(), &layout, &raw, &pass)?;
                for (w, exposure) in pass.iter_mut().zip(next) {
                    w.exposure = exposure;
                }
                if n >= metering.max_passes.max(1) {
                    debug!(passes = n, "metering did not settle");
                    break;
                }
            }

            // The unit measured this pass too: `DataType::Setup` reports the film base and
            // the levels its own prescan found. Ours is a percentile and its is
            // a min and a max, so they will not agree exactly. Worth logging
            // side by side until we know which to trust for what
            let measured = metering.measure(&layout, &raw)?;
            for (n, (window, level)) in pass.iter().zip(&measured).enumerate() {
                let unit = session.setup(window.id).ok();
                let image = unit.as_ref().and_then(|s| s.images.first());
                debug!(
                    channel = n,
                    id = window.id,
                    ours = level,
                    base_level = unit.as_ref().map(|s| s.base_level),
                    unit_min = image.map(|i| i.min),
                    unit_max = image.map(|i| i.max),
                    "metering levels"
                );
            }

            // Whatever the loop left in `pass` is what the last pass decided
            Ok(seeded
                .iter()
                .zip(&pass)
                .map(|(w, metered)| Window {
                    exposure: metered.exposure,
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
            // 2-11-3 answers `DataType::WhiteBalanceExposure` in R, G, B, and
            // the default qualifier is green
            if let Some(&exposure) = w.channel().visible_index().and_then(|n| wb.get(n)) {
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
            // A preview halves Y where a scan is square, and runs without the
            // averaging bit. The captures pair 666x333 with byte 41 = 01h and
            // high speed every time
            w.resolution = (dpi, dpi / 2);
            w.multiple_reading = 0;
            w.flags.remove(Flags::AVERAGING);
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

    let started = session.scan(windows)?;
    session.test_unit_ready(PASS_TIMEOUT)?;
    Ok(started.layout)
}

/// Whether a set has anything worth metering
///
/// Infrared measures obstructions, so a set of nothing else has no exposure to
/// decide from a film's tones.
pub fn meterable(windows: &[Window]) -> bool {
    windows.iter().any(|w| w.channel().is_color())
}
