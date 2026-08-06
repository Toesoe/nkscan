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
    protocol::window::Composition,
    session::Session,
};

/// Where the raw stream goes, as a fixture to write a decoder against
const DUMP: &str = "scan.raw";

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let mode = std::env::args().nth(1).unwrap_or_default();
    let ids: &[u8] = if mode == "rgb" { &[1, 2, 3] } else { &[1] };

    let devices = device::list();
    let device = Selector::Only.resolve(&devices)?;
    let mut session = Session::open(device.open()?)?;

    // The lowest resolution this unit offers, over a small patch of the frame
    let dpi = session.capabilities().address.x_axis.dpi_range.start;
    let held = session.windows()?;
    let mut windows = Vec::new();
    for id in ids {
        let mut w = held
            .iter()
            .find(|w| w.id == *id)
            .unwrap_or_else(|| panic!("no window {id}"))
            .clone();
        w.resolution = (dpi, dpi);
        w.origin = (518, 2236);
        w.size = (1200, 1200);
        // 2-10-6 has one code for a one-plane output and one for three
        w.composition = if ids.len() > 1 {
            Composition::MultilevelRGB
        } else {
            Composition::MultilevelBW
        };
        windows.push(w);
    }
    println!("{:#?}", windows[0]);
    println!(
        "exposures: {:?}",
        windows
            .iter()
            .map(|w| (w.id, w.exposure))
            .collect::<Vec<_>>()
    );

    let started = Instant::now();
    for w in &windows {
        session.set_window(w)?;
    }
    println!("set {} window(s) in {:?}", windows.len(), started.elapsed());

    let started = Instant::now();
    let layout = match session.scan(&windows) {
        Ok(layout) => {
            println!("scan started in {:?}", started.elapsed());
            layout
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
    if mode == "noread" {
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
