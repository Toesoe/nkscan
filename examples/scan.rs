//! REMOVE LATER: does a minimal scan owe the host anything?
//!
//! Takes the descriptors the unit already holds, sets a window at the chosen
//! frame's front edge, and starts a scan. Single line, no multisampling,
//! 16 bit, none of the four triggers 2-11-5 lists, so in theory it should
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
//! cargo run --example scan -- rgb frame=1   # the second of the unit's frames
//! cargo run --example scan -- rgb len=6696  # 6x4.5 frames rather than 6x6
//! cargo run --example scan -- rgb notable   # put the whole-sensor table back
//! cargo run --example scan -- rgb aperture  # the holder opening, not the whole sensor
//! cargo run --example scan -- rgb multiline # the three-line CCD mode
//! cargo run --example scan -- thumb only     # the thumbnail pass alone, to thumb.raw
//! cargo run --example scan -- noread        # leave the image unread
//! ```
//!
//! This moves the stage: the window origin comes from the frame's front edge.
//! Those edges are nominal until a thumbnail has measured them (2-11-6), so the
//! preamble tiles `len` along the opening the holder publishes.

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
        data::{Boundary, Rect},
        window::{Composition, Window},
    },
    scan::{
        expose::{self, Exposure},
        focus::Focus,
        framing::{self, Framing},
        preamble, thumbnail,
    },
    session::Session,
};

/// Where the raw stream goes, as a fixture to write a decoder against
const DUMP: &str = "scan.raw";

/// Where the thumbnail pass goes
const THUMB: &str = "thumb.raw";

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

    // The setup the captures run once before the first pass. len=N is the film
    // format, which nothing advertises
    let format = arg("len").unwrap_or(8964);
    // A table outlives a session, so showing what one is worth means writing the
    // whole-sensor rectangle back rather than declining to write anything
    let table = match has("notable") {
        true => whole_sensor(session.capabilities()),
        false => framing::table(session.capabilities(), format)?,
    };
    if !has("nopreamble") {
        let started = Instant::now();
        preamble::run(&mut session, &table)?;
        println!("preamble in {:?}", started.elapsed());
    }

    // The captures open with the whole-strip pass before any frame placement:
    // it is what finds where the frames are. Run it first so the stage does not
    // go out to a frame and then all the way back to the strip start
    if has("thumb") {
        let began = Instant::now();
        println!(
            "thumbnail available={} framing {:?} ready={} host builds={}",
            thumbnail::available(session.capabilities()),
            Framing::choose(session.capabilities()),
            Framing::choose(session.capabilities()).ready(),
            thumbnail::host_builds(session.capabilities())
        );
        match thumbnail::scan(&mut session) {
            Ok(t) => {
                println!(
                    "thumbnail {} x {} in {:?}, {} of an expected {} bytes, owes {:?}",
                    t.layout.pixels,
                    t.layout.lines,
                    began.elapsed(),
                    t.data.len(),
                    t.layout.total_bytes(),
                    t.cooperation
                );
                File::create(THUMB)?.write_all(&t.data)?;
                println!(
                    "written to {THUMB}: cargo run --release --example decode -- {THUMB} {} {} 1",
                    t.layout.pixels, t.layout.lines
                );
            }
            Err(e) => println!("thumbnail refused: {e}"),
        }
        session.refresh()?;
        println!("frames now: {:?}", session.capabilities().frames);
        if has("only") {
            return Ok(());
        }
    }

    // 2-11-6: where the unit thinks each frame is, which the preamble has just
    // told it
    let boundary = session.boundaries()?;
    println!("boundaries: {boundary:?}");

    // The lowest resolution this unit offers, over the frame the window lands on
    let caps = session.capabilities();
    let dpi = caps.address.x_axis.dpi_range.start;
    let aperture = (
        caps.frames
            .as_ref()
            .and_then(|f| f.images.first())
            .map_or(0, |f| f.left),
        caps.address.x_axis.boundary,
    );
    let (mut origin, size) = place(caps, &boundary, arg("frame").or(Some(1)));
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

    // Before metering, which is the order in the captures: autofocus, then the
    // preview passes that decide the exposures. So AE measures a focused frame
    if !has("nofocus") {
        let started = Instant::now();
        // afy=N drives the sub-scanning address straight, to find out which
        // coordinate space AF actually wants. 2-15 calls it an address on the
        // medium, while Address says SET WINDOW addresses are mechanism positions
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
        windows = expose::expose(&mut session, &windows, exposure)?;
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

/// The one rectangle the unit answers `DataType::Boundary` with before any host has told it
/// where the frames are, so `notable` can put the mechanism back in that state
fn whole_sensor(caps: &Capabilities) -> Boundary {
    Boundary {
        frames: vec![Rect {
            top: 0,
            left: 0,
            bottom: caps.address.y_axis.address_range.last,
            right: caps.address.x_axis.address_range.last,
        }],
    }
}

/// Where Nikon Scan put the window in the reference capture, and the frame's
/// front edge a scan is supposed to sit at: frame 2 of the 6x9 strip
/// (`docs/CAPTURES.md`), origin `(518, 12720)`, size `(8964, 8964)`.
///
/// Until this unit has measured a strip it answers `DataType::Boundary` with one rectangle
/// covering the whole sensor, so "read the boundary" is not enough to know
/// where the frames are; the measured geometry from `--frame` wins when the
/// unit has it, and this is what a scan falls back to.
const NIKON_ORIGIN: (u32, u32) = (518, 12720);
const NIKON_SIZE: (u32, u32) = (8964, 8964);

/// Put a window at the front edge of the chosen measured frame, or at the
/// window Nikon Scan itself used when nothing is measured.
///
/// The unit's rectangles are in window-origin coordinates, so the
/// frame's front edge is its own top-left corner and its size is the whole
/// rectangle, so the stage goes to the frame and stays there. No frame is
/// measured until the host has done the boundary write-back 2-11-6 asks for,
/// so without `--frame` the Nikon Scan geometry stands in.
fn place(caps: &Capabilities, boundary: &Boundary, pick: Option<u32>) -> ((u32, u32), (u32, u32)) {
    let (x, y) = (&caps.address.x_axis, &caps.address.y_axis);

    // A frame the unit has measured: its front edge is the origin, its extent
    // is the window, since the capture's window is the whole frame
    if let Some(frame) = pick.and_then(|n| boundary.frames.get(n as usize)) {
        let origin = (frame.left, frame.top);
        let size = (frame.right - frame.left, frame.bottom - frame.top);
        let clamp =
            |v: u32, axis: &Axis| v.clamp(axis.address_range.start, axis.address_range.last);
        return ((clamp(origin.0, x), clamp(origin.1, y)), size);
    }

    // Nothing measured, and the unit's one rectangle is the whole sensor:
    // reproduce the window Nikon Scan itself used for this holder
    (
        (NIKON_ORIGIN.0, NIKON_ORIGIN.1),
        (NIKON_SIZE.0, NIKON_SIZE.1),
    )
}
