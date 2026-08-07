//! REMOVE LATER: scan one frame and write it out
//!
//! Takes the descriptors the unit already holds, puts a window on the chosen
//! frame, meters it, scans it and writes the result as 16-bit Netpbm. Colour
//! planes go together and anything else, infrared being the one that turns up,
//! gets a file of its own.
//!
//! ```text
//! cargo run --example scan                 # meter and scan frame 1
//! cargo run --example scan -- mono         # one channel rather than colour
//! cargo run --example scan -- ir           # add the infrared channel
//! cargo run --example scan -- multiline    # the three-line CCD mode
//! cargo run --example scan -- samples=2    # read each line twice and average
//! cargo run --example scan -- dpi=4000     # full resolution
//! cargo run --example scan -- frame=1      # which frame of the tiled table
//! cargo run --example scan -- len=6696     # 6x4.5 frames rather than 6x6
//! cargo run --example scan -- lockwb       # keep the channels in proportion
//! cargo run --example scan -- noae nofocus # skip metering and focus
//! cargo run --example scan -- thumb        # measure the strip and stop
//! cargo run --example scan -- keep frame=2 # scan against what thumb measured
//! cargo run --example scan -- diagnose     # read a pending fault and stop
//! cargo run --example scan -- eject        # give the film back and stop
//! ```
//!
//! This moves the stage: the window origin comes from the frame's front edge.
//! Those edges are nominal until a thumbnail has measured them (2-11-6), so the
//! preamble tiles `len` along the opening the adapter publishes. `thumb` takes
//! that pass and replaces the tiled table with a measured one; `keep` is what
//! scans against it afterwards rather than tiling over it again.

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
        data::Boundary,
        window::{Channel, Composition, Flags, Window},
    },
    scan::{
        expose::{self, Exposure},
        focus::Focus,
        framing::{self, Framing},
        pass::{self, Pass},
        preamble, thumbnail,
    },
    session::Session,
};

/// What the scan is written as
const DUMP: &str = "scan";

/// A full-resolution pass is minutes of stage travel
const SCAN_TIMEOUT: Duration = Duration::from_secs(600);

/// Where the thumbnail pass goes
const THUMB: &str = "thumb";

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
    // SCAN order, which is also the order the stream interleaves them. The
    // captures lead with infrared
    let mut ids: Vec<u8> = if has("mono") { vec![1] } else { vec![1, 2, 3] };
    if has("ir") {
        ids.insert(0, Channel::Infrared.id());
    }
    let ids = &ids[..];

    let devices = device::list();
    let device = Selector::Only.resolve(&devices)?;
    let mut session = Session::open(device.open()?)?;

    // 2-15-3: give the film back and stop
    if has("eject") {
        let started = Instant::now();
        session.eject()?;
        println!("ejected in {:?}", started.elapsed());
        return Ok(());
    }

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
    // A measured table is the one the stage should step against, and tiling
    // over it would throw the measurement away
    let table = match has("keep") {
        true => Boundary::default(),
        false => framing::table(session.capabilities(), format)?,
    };
    let started = Instant::now();
    preamble::run(&mut session, &table)?;
    println!("preamble in {:?}", started.elapsed());

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
        let curves = session.ccd_curves(0);
        let mut samples = Vec::new();
        match thumbnail::scan(&mut session, curves.as_ref(), &mut samples) {
            Ok(t) => {
                println!(
                    "thumbnail {} x {} in {:?}, complete={}, owes {:?}",
                    t.layout.pixels,
                    t.layout.lines,
                    began.elapsed(),
                    t.complete,
                    t.cooperation
                );
                // Decoded on the way in, so this is an image rather than a
                // stream needing the decode example
                netpbm(THUMB, &samples, &t)?;

                // 2-11-6: the host is what works the frames out of this pass
                let measured =
                    thumbnail::frames(session.capabilities(), &t, &samples, format, None)?;
                println!("measured: {measured:?}");
                session.set_boundaries(&measured)?;

                // Bar every edge the table claims, so it can be looked at
                // against the picture it was measured from
                let pitch = t.layout.line_pitch.max(1);
                let origin = session.capabilities().address.y_axis.address_range.start;
                let rows: Vec<usize> = measured
                    .frames
                    .iter()
                    .flat_map(|f| [f.top, f.bottom])
                    .map(|y| (y.saturating_sub(origin) / pitch) as usize)
                    .collect();
                mark(&mut samples, &t, &rows);
                netpbm(&format!("{THUMB}.frames"), &samples, &t)?;
            }
            Err(e) => println!("thumbnail refused: {e}"),
        }
        session.refresh()?;
        println!("frames now: {:?}", session.capabilities().frames);
        return Ok(());
    }

    // 2-11-6: where the unit thinks each frame is, which the preamble has just
    // told it
    let boundary = session.boundaries()?;
    println!("boundaries: {boundary:?}");

    // The lowest resolution this unit offers, over the frame the window lands on
    let caps = session.capabilities();
    // dpi=N scans at something other than the cheapest resolution. Off the
    // ladder the unit rounds and says so with 01h-37h rather than refusing
    let dpi = arg("dpi").unwrap_or(u32::from(caps.address.x_axis.dpi_range.start)) as u16;
    let (origin, size) = place(caps, &boundary, arg("frame").or(Some(1)));
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
        // The unit keeps whatever the last run left in these, so say what we
        // want rather than inheriting a previous experiment
        w.scanning_kind = ScanKind::IMAGE;
        w.scanning_mode = ScanMode::HIGH_SPEED;
        // A set has to agree on these, and the unit holds a different set per
        // window, so every one of them gets said rather than inherited
        w.flags = Flags::POSITIVE;
        // samples=N reads each line N times for the host to average. Byte 40
        // carries one less than the count, and byte 43 has to say so too
        w.multiple_reading = arg("samples").unwrap_or(1).max(1) as u8 - 1;
        if w.multiple_reading != 0 {
            w.scanning_mode |= ScanMode::MULTI_READING;
        }
        w.color_interleaving = match has("multiline") {
            true => ColorInterleaving::MULTILINE_SIMULTANEOUS,
            false => ColorInterleaving::LINE_WITHOUT_DISTANCE,
        };
        // 2-10-6 has one code for a one-plane output and one for three
        // 2-10-6 counts the visible planes, so infrared does not sway it
        w.composition = match ids.iter().filter(|&&i| Channel::from(i).is_color()).count() {
            1 => Composition::MultilevelBW,
            _ => Composition::MultilevelRGB,
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
    // Before metering, which is the order in the captures: autofocus, then the
    // passes that decide the exposures. So AE measures a focused frame
    if !has("nofocus") {
        let started = Instant::now();
        let focused = Focus::default().apply(&mut session, &windows)?;
        println!("focus: {focused:?} in {:?}", started.elapsed());
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
    let curves = session.ccd_curves(0);
    let mut samples = Vec::new();
    let taken = pass::take(
        &mut session,
        &windows,
        SCAN_TIMEOUT,
        curves.as_ref(),
        &mut samples,
    )?;
    println!(
        "{} x {} at {} dpi, {:?}, complete={}, owes {:?}, in {:?}",
        taken.layout.pixels,
        taken.layout.lines,
        taken.layout.dpi,
        taken.layout.interleaving,
        taken.complete,
        taken.cooperation,
        started.elapsed()
    );

    netpbm(DUMP, &samples, &taken)?;
    Ok(())
}

/// Paint the named rows solid, so a frame table can be looked at against the
/// thumbnail it was measured from
fn mark(samples: &mut [u16], pass: &Pass, rows: &[usize]) {
    let stride = pass.cols * pass.layout.channels.len();
    for y in rows {
        if let Some(row) = samples.get_mut(y * stride..(y + 1) * stride) {
            row.fill(u16::MAX);
        }
    }
}

/// Write decoded samples where they can be looked at, 16-bit Netpbm
///
/// A row at a time: the whole point of decoding as the stream arrives is not to
/// hold a second copy of the image
fn netpbm(stem: &str, samples: &[u16], pass: &Pass) -> anyhow::Result<()> {
    let ids = &pass.layout.channels;
    let color: Vec<usize> = (0..ids.len())
        .filter(|&c| Channel::from(ids[c]).is_color())
        .collect();
    plane(stem, samples, pass, &color)?;

    // Infrared is not a color and has no place in an RGB file
    for (c, id) in ids.iter().enumerate() {
        if !Channel::from(*id).is_color() {
            let name = format!(
                "{stem}.{}",
                format!("{:?}", Channel::from(*id)).to_lowercase()
            );
            plane(&name, samples, pass, &[c])?;
        }
    }
    Ok(())
}

/// One file holding the named channels, written a row at a time so the image is
/// never copied whole
fn plane(stem: &str, samples: &[u16], pass: &Pass, channels: &[usize]) -> anyhow::Result<()> {
    let (magic, ext) = match channels.len() {
        1 => ("P5", "pgm"),
        3 => ("P6", "ppm"),
        n => anyhow::bail!("{n} channels has no Netpbm form"),
    };
    let (rows, cols, stride) = (pass.rows, pass.cols, pass.layout.channels.len());
    let dest = format!("{stem}.{ext}");
    let mut file = std::io::BufWriter::new(File::create(&dest)?);
    write!(file, "{magic}\n{cols} {rows}\n65535\n")?;

    let mut row = Vec::with_capacity(cols * channels.len() * 2);
    for y in 0..rows {
        row.clear();
        for x in 0..cols {
            for &c in channels {
                row.extend_from_slice(&samples[(y * cols + x) * stride + c].to_be_bytes());
            }
        }
        file.write_all(&row)?;
    }
    file.flush()?;
    println!("wrote {dest}");
    Ok(())
}

/// Where Nikon Scan put the window in the reference capture, and the frame's
/// front edge a scan is supposed to sit at: frame 2 of the 6x9 strip
/// (`docs/CAPTURES.md`), origin `(518, 12720)`, size `(8964, 8964)`.
///
/// Until this unit has measured a strip it answers `DataType::Boundary` with one rectangle
/// covering the whole sensor, so "read the boundary" is not enough to know
/// where the frames are; the measured geometry from `frame=` wins when the
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
/// so without `frame=` the Nikon Scan geometry stands in.
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
