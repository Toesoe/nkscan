//! Running an exposure pass
//!
//! [`Exposure`] says which mechanism to use. This does it and hands back the
//! same windows with their exposures filled in.

use super::{meter::Metering, pass};
use crate::{
    error::Error,
    protocol::{
        caps::{
            Capabilities,
            set_window::{AnalogControl, ScanKind, ScanMode},
        },
        decode::{Decoder, Image},
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
/// Leaves everything else about them alone, so the caller can set them and scan
pub fn expose(
    session: &mut Session,
    windows: &[Window],
    exposure: Exposure,
) -> Result<Vec<Window>, Error> {
    match exposure {
        Exposure::Unit(kind) => {
            // The unit meters during a pass of its own. It writes the result
            // into the descriptors, so GET WINDOW is what reports it
            let mut metering = windows.to_vec();
            for w in &mut metering {
                w.scanning_kind = kind;
            }
            // Its own numbers are the point, not the image, so the pass is
            // stopped rather than read out
            pass::start(session, &metering, PASS_TIMEOUT)?;
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
            let mut windows = prescan_windows(session.capabilities(), &seeded);
            // Every pass has the same shape, so one buffer serves all of them
            let mut samples = Vec::new();
            let mut layout = None;
            let mut n = 0;
            loop {
                let taken = pass::take(session, &windows, PASS_TIMEOUT)?;
                let mut decoder = Decoder::new(&taken.layout)?;
                samples.resize(decoder.samples(), 0);
                decoder.push(&taken.data, &mut samples)?;
                let layout = layout.insert(taken.layout);
                let image = Image::new(layout, &samples)?;
                n += 1;

                let settled = metering.settled(&image);
                debug!(pass = n, settled, "metering pass");
                if settled {
                    break;
                }

                let next = metering.apply(session.capabilities(), &image, &windows)?;
                for (w, exposure) in windows.iter_mut().zip(next) {
                    w.exposure = exposure;
                }
                if n >= metering.max_passes.max(1) {
                    debug!(passes = n, "metering did not settle");
                    break;
                }
            }

            // `DataType::Setup` holds what the unit made of the same pass. Ours
            // is a percentile and its is a min and a max, so the two will not
            // agree exactly
            let layout = layout.expect("the loop runs at least once");
            let measured = metering.measure(&Image::new(&layout, &samples)?);
            for (n, (window, level)) in windows.iter().zip(&measured).enumerate() {
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

            // Whatever the loop left in `windows` is what the last pass decided
            Ok(seeded
                .iter()
                .zip(&windows)
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
/// Any pass wants this, not just a metered one: the channels do not read neutral
/// at equal exposures, so a descriptor left at 0 comes back with a cast.
///
/// Only the visible channels: 2-11-3's qualifier has no infrared, so an IR
/// window keeps the exposure it came with.
pub fn seed_white_balance(session: &mut Session, windows: &[Window]) -> Result<Vec<Window>, Error> {
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
fn prescan_windows(caps: &Capabilities, windows: &[Window]) -> Vec<Window> {
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

/// Whether a set has anything worth metering
///
/// Infrared measures obstructions, so a set of nothing else has no exposure to
/// decide from a film's tones.
pub fn meterable(windows: &[Window]) -> bool {
    windows.iter().any(|w| w.channel().is_color())
}
