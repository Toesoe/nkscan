//! REMOVE LATER: does a minimal scan owe the host anything?
//!
//! Takes the descriptors the unit already holds, shrinks them to a small patch
//! at the lowest resolution, and starts a scan. Single line, no multisampling,
//! 16 bit -- none of the four triggers 2-11-5 lists, so in theory it should
//! raise no cooperative request at all.
//!
//! ```text
//! cargo run --example scan                  # red only
//! cargo run --example scan -- rgb           # three channels, dumped to scan.raw
//! cargo run --example scan -- rgb lockwb    # keep the channels in proportion
//! cargo run --example scan -- rgb noae      # skip metering
//! cargo run --example scan -- rgb nofocus   # skip autofocus
//! cargo run --example scan -- diagnose      # read a pending fault and stop
//! cargo run --example scan -- rgb afy=600   # autofocus at a raw sub-scan address
//! cargo run --example scan -- rgb aftop     # focus the window origin, not its middle
//! cargo run --example scan -- rgb aty=11976 # put the window at a given Y
//! cargo run --example scan -- rgb aperture  # the holder opening, not the whole sensor
//! cargo run --example scan -- rgb multiline # the three-line CCD mode
//! cargo run --example scan -- rgb thumb     # run a thumbnail pass first
//! cargo run --example scan -- noread        # leave the image unread
//! ```
//!
//! This moves the stage: the window origin comes from the frame rectangle.

use std::{
    fs::File,
    io::Write,
    time::{Duration, Instant},
};

use nkscan::{
    device::{self, Selector},
    protocol::{
        caps::{
            Capabilities,
            address::Axis,
            set_window::{ColorInterleaving, ScanKind, ScanMode},
        },
        window::{Composition, Window},
    },
    scan::{Exposure, Focus, expose, thumbnail},
    session::Session,
};

/// Where the raw stream goes, as a fixture to write a decoder against
const DUMP: &str = "scan.raw";

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |flag: &str| args.iter().any(|a| a == flag);
    let arg = |name: &str| {
        args.iter()
            .find_map(|a| a.strip_prefix(&format!("{name}="))?.parse::<u32>().ok())
    };
    let ids: &[u8] = if has("rgb") { &[1, 2, 3] } else { &[1] };

    let devices = device::list();
    let device = Selector::Only.resolve(&devices)?;
    let mut session = Session::open(device.open()?)?;

    // 2-8: whatever a failed operation left behind, read once and gone
    if has("diagnose") {
        match session.diagnose()? {
            Some(sense) => println!("pending fault: {sense:?}"),
            None => println!("the unit reports no pending fault"),
        }
        return Ok(());
    }

    // The lowest resolution this unit offers, over a small patch of the frame
    let caps = session.capabilities();
    let dpi = caps.address.x_axis.dpi_range.start;
    let aperture = (
        caps.frames
            .as_ref()
            .and_then(|f| f.images.first())
            .map_or(0, |f| f.left),
        caps.address.x_axis.boundary,
    );
    let (mut origin, size) = place(caps, PATCH);
    // aty=N puts the window where we want it, so the scan can be moved onto
    // film autofocus will actually accept
    if let Some(y) = arg("aty") {
        origin.1 = y;
    }
    println!(
        "placing {size:?} at {origin:?}, center y={}",
        origin.1 + size.1 / 2
    );

    let held = session.windows()?;
    let mut windows = Vec::new();
    for id in ids {
        let mut w = held
            .iter()
            .find(|w| w.id == *id)
            .unwrap_or_else(|| panic!("no window {id}"))
            .clone();
        // Three things Nikon Scan's preview descriptor does that ours does not,
        // each on its own flag so they can be bisected
        // A scan is square; the metering pass halves Y for itself
        w.resolution = (dpi, dpi);
        w.origin = origin;
        w.size = size;
        if has("aperture") {
            w.origin.0 = aperture.0;
            w.size.0 = aperture.1;
        }
        // The unit keeps whatever the last run left in these, so say what we
        // want rather than inheriting a previous experiment
        w.scanning_kind = ScanKind::IMAGE;
        w.scanning_mode = ScanMode::HIGH_SPEED;
        w.multiple_reading = 0;
        w.color_interleaving = match has("multiline") {
            true => ColorInterleaving::MULTILINE_SIMULTANEOUS,
            false => ColorInterleaving::LINE_WITHOUT_DISTANCE,
        };
        // 2-10-6 has one code for a one-plane output and one for three
        w.composition = if ids.len() > 1 {
            Composition::MultilevelRGB
        } else {
            Composition::MultilevelBW
        };
        windows.push(w);
    }
    println!("{:#?}", windows[0]);
    let show = |what: &str, windows: &[Window]| {
        let e: Vec<_> = windows.iter().map(|w| (w.id, w.exposure)).collect();
        println!("{what}: {e:?}");
    };
    show("exposures as held", &windows);

    // Every capture opens with one of these, and this unit has never measured
    // the strip: C8h reports a frame length of 0. See whether it settles the
    // mechanism down
    if has("thumb") {
        let began = Instant::now();
        println!(
            "thumbnail available={} frames known={} host builds={}",
            thumbnail::available(&session),
            thumbnail::frames_known(&session),
            thumbnail::host_builds(&session)
        );
        match thumbnail::scan(&mut session) {
            Ok(t) => println!(
                "thumbnail {} bytes of an expected {} in {:?}, owes {:?}",
                t.data.len(),
                t.layout.total_bytes(),
                began.elapsed(),
                t.cooperation
            ),
            Err(e) => println!("thumbnail refused: {e}"),
        }
        session.refresh()?;
        println!("frames now: {:?}", session.capabilities().frames);
    }

    // Before metering, which is the order in the captures: autofocus, then the
    // preview passes that decide the exposures. So AE measures a focused frame
    if !has("nofocus") {
        let started = Instant::now();
        // afy=N drives the sub-scanning address straight, to find out which
        // coordinate space AF actually wants. 2-15 calls it an address on the
        // medium, while C1h says SET WINDOW addresses are mechanism positions
        let focused = match arg("afy") {
            Some(y) => {
                println!("autofocus at raw y={y}");
                match session.autofocus(5000, y, None) {
                    Ok(()) => "Yes".to_string(),
                    Err(e) => format!("{e}"),
                }
            }
            // Focusing at the window origin leaves the stage where the scan
            // begins, so the SET WINDOW after it has nothing to move
            None => {
                let at = if has("aftop") { (0.5, 0.0) } else { (0.5, 0.5) };
                let focus = Focus::Auto { at, color: None };
                format!("{:?}", focus.apply(&mut session, &windows)?)
            }
        };
        println!("focus: {focused} in {:?}", started.elapsed());
    }

    if !has("noae") {
        let exposure = Exposure::choose(session.capabilities(), has("lockwb"))?;
        println!("metering: {exposure:?}");
        let started = Instant::now();
        windows = expose(&mut session, &windows, exposure)?;
        println!("metered in {:?}", started.elapsed());
        show("exposures metered", &windows);
    }

    let started = Instant::now();
    for w in &windows {
        session.set_window(w)?;
    }
    println!("set {} window(s) in {:?}", windows.len(), started.elapsed());

    let started = Instant::now();
    let layout = match session.scan(&windows) {
        Ok(begun) => {
            println!(
                "scan started in {:?}, owes {:?}",
                started.elapsed(),
                begun.cooperation
            );
            begun.layout
        }
        Err(e) => {
            println!("scan refused after {:?}: {e}", started.elapsed());
            return Ok(());
        }
    };

    let started = Instant::now();
    session.test_unit_ready(Duration::from_secs(180))?;
    println!("scan finished in {:?}", started.elapsed());

    // Leaving the image unread is what wedges the next session
    if has("noread") {
        println!("leaving the image unread");
        return Ok(());
    }

    println!(
        "expecting {} x {} at {} dpi, pitch {}, {:?} over {} bytes",
        layout.pixels,
        layout.lines,
        layout.dpi,
        layout.pitch,
        layout.interleaving,
        layout.total_bytes()
    );

    let started = Instant::now();
    let mut out = File::create(DUMP)?;
    let mut got = 0u64;
    let mut chunks = session.image_chunks(&layout)?;
    while let Some(chunk) = chunks.next() {
        let chunk = chunk?;
        out.write_all(chunk)?;
        got += chunk.len() as u64;
    }
    println!(
        "read {got} of {} bytes in {:?}, written to {DUMP}",
        layout.total_bytes(),
        started.elapsed()
    );

    Ok(())
}

/// How big a patch to ask for, before any axis says otherwise
const PATCH: u32 = 1200;

/// Put a window at the far end of what this holder offers, using only what the
/// unit advertises so any scanner and any holder land somewhere legal
///
/// Full sensor width, since the CCD reaches past the holder aperture and the
/// film border it catches is worth seeing. `patch` only bounds the length.
///
/// Answers `(origin, size)`
fn place(caps: &Capabilities, patch: u32) -> ((u32, u32), (u32, u32)) {
    let (x, y) = (&caps.address.x_axis, &caps.address.y_axis);

    // 2-2-2-3: an axis with no address range has to be read whole
    let width = if x.croppable() {
        u32::from(caps.address.ccd_pixels)
    } else {
        x.boundary
    };
    let height = if y.croppable() {
        patch.min(y.boundary)
    } else {
        y.boundary
    };

    // Where a frame starts is published; where it ends is only published for a
    // holder that has measured it. Failing that the boundary is the longest
    // window this holder allows, which is as far as the scannable region goes
    let last = caps.frames.as_ref().and_then(|f| f.images.last());
    let (left, top) = match last {
        // Full width means starting at the sensor, not at the frame
        Some(frame) => (
            0,
            frame.top + frame.length.unwrap_or(y.boundary).saturating_sub(height),
        ),
        None => (
            x.address_range.start,
            y.address_range.start + y.boundary.saturating_sub(height),
        ),
    };

    let clamp = |v: u32, axis: &Axis| v.clamp(axis.address_range.start, axis.address_range.last);
    ((clamp(left, x), clamp(top, y)), (width, height))
}
