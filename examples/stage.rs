//! REMOVE LATER: what does the frame table have to say for the stage to step?
//!
//! A frame-kind SET WINDOW against the placeholder table the unit ships with
//! drives the holder nearly out the front and back; against a table of real
//! frames it steps. Three readings fit that, and they disagree about what the
//! table has to say:
//!
//! - **span**: the table's extent has to cover the window
//! - **containment**: one rectangle has to contain the window
//! - **correspondence**: one rectangle has to be the window
//!
//! Hold the film format still, vary the table and the window, and time the two
//! things that position the stage: autofocus, and the SET WINDOW after it.
//!
//! ```text
//! cargo run --example stage
//! ```
//!
//! Every rectangle here is bounded by the Y boundary `Address` reports. Past
//! that the stage position a frame-kind SET WINDOW drives to comes out behind
//! the home stop and the mechanism grinds until a power cycle, so `framing`
//! refuses it and this probe must not hand-roll its way around that.
//!
//! Watch the holder as well as the clock. Homing is unmistakable.

use std::time::Instant;

use nkscan::{
    device::{self, Selector},
    protocol::{
        caps::set_window::{ColorInterleaving, ScanKind, ScanMode},
        data::{Boundary, Rect},
        window::Window,
    },
    scan::framing,
    session::Session,
};

/// 6x6 on 120 film, the format the strip in the holder is cut to
const FORMAT: u32 = 8964;
/// The frame this probe scans, the middle of the three the format tiles into
const FRAME: usize = 1;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let devices = device::list();
    let device = Selector::Only.resolve(&devices)?;
    let mut session = Session::open(device.open()?)?;

    let caps = session.capabilities();
    let limit = caps.address.y_axis.boundary;
    let tiled = framing::table(caps, FORMAT)?;
    let frame = tiled.frames[FRAME];
    let (left, right) = (frame.left, frame.right);
    let one = |top, length| Boundary {
        frames: vec![Rect {
            top,
            left,
            bottom: top + length,
            right,
        }],
    };

    // Each round is a table and the window to try against it, ordered so the
    // two known answers bracket the two that separate the readings
    let rounds: [(&str, Boundary, Rect); 5] = [
        // Known fast: real frames, window on the middle one
        ("tiled", tiled.clone(), frame),
        // Known slow: the placeholder the unit answers with until a host writes
        // one, which the window outruns
        ("placeholder", one(0, limit), frame),
        // Contains the window, corresponds to no frame, and still inside the
        // boundary. Fast here means containment is enough
        ("loose", one(frame.top, limit), frame),
        // Contains the window and is the window
        ("exact", one(frame.top, FORMAT), frame),
        // Real frames, window laid across two of them: inside the table's
        // extent but inside no single rectangle
        (
            "straddle",
            tiled.clone(),
            Rect {
                top: frame.top + FORMAT / 2,
                bottom: frame.top + FORMAT / 2 + FORMAT,
                left,
                right,
            },
        ),
    ];

    let held = session.windows()?;
    let mut results = Vec::new();

    for (name, table, window) in rounds {
        println!("\n== {name} ==");
        println!("  table {:?}", table.frames);
        println!("  window y {}..{}", window.top, window.bottom);
        session.set_boundaries(&table)?;

        // Put the stage on the window, so every round starts the same way
        // whatever the last one left behind
        stage(&mut session, &held, window)?;

        // Autofocus positions the stage at the window center, so its own
        // elapsed time reports the same thing the SET WINDOW after it does
        let (x, y) = (
            window.left + (window.right - window.left) / 2,
            window.top + (window.bottom - window.top) / 2,
        );
        let started = Instant::now();
        let focused = session.autofocus(x, y, None);
        let af = started.elapsed();
        println!("  autofocus at ({x}, {y}): {focused:?} in {af:?}");

        // The measurement: half a frame of travel if it steps, out the front
        // and back if it homes
        let started = Instant::now();
        stage(&mut session, &held, window)?;
        let set = started.elapsed();
        println!("  SET WINDOW in {set:?}");
        results.push((name, af, set));
    }

    println!("\n{:<12} {:>10} {:>12}", "round", "autofocus", "SET WINDOW");
    for (name, af, set) in results {
        println!("{name:<12} {af:>10.2?} {set:>12.2?}");
    }
    Ok(())
}

/// Put the probe's window on every visible channel, taking the rest of each
/// descriptor from what the unit already holds
fn stage(session: &mut Session, held: &[Window], at: Rect) -> anyhow::Result<()> {
    let dpi = session.capabilities().address.x_axis.dpi_range.start;
    for id in [1u8, 2, 3] {
        let mut w = held
            .iter()
            .find(|w| w.id == id)
            .unwrap_or_else(|| panic!("no window {id}"))
            .clone();
        w.resolution = (dpi, dpi);
        w.origin = (at.left, at.top);
        w.size = (at.right - at.left, at.bottom - at.top);
        w.scanning_kind = ScanKind::IMAGE;
        w.scanning_mode = ScanMode::HIGH_SPEED;
        w.multiple_reading = 0;
        w.color_interleaving = ColorInterleaving::LINE_WITHOUT_DISTANCE;
        session.set_window(&w)?;
    }
    Ok(())
}
