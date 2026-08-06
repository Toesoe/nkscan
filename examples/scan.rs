//! REMOVE LATER: does a minimal scan owe the host anything?
//!
//! Takes the descriptor the unit already holds for one channel, shrinks it to a
//! small patch at the lowest resolution, and starts a scan. Single line, no
//! multisampling, 16 bit -- none of the four triggers 2-11-5 lists, so in theory
//! it should raise no cooperative request at all.
//!
//! This moves the stage: the window origin comes from the frame rectangle.

use std::time::{Duration, Instant};

use nkscan::{
    device::{self, Selector},
    session::Session,
};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let devices = device::list();
    let device = Selector::Only.resolve(&devices)?;
    let mut session = Session::open(device.open()?)?;

    let mut window = session
        .windows()?
        .into_iter()
        .find(|w| w.id == 1)
        .expect("the red window");

    // The lowest resolution this unit offers, over a small patch of the frame
    let dpi = session.capabilities().address.x_axis.dpi_range.start;
    window.resolution = (dpi, dpi);
    window.origin = (518, 2236);
    window.size = (1200, 1200);
    println!("{window:#?}");

    let started = Instant::now();
    session.set_window(&window)?;
    println!("set window in {:?}", started.elapsed());

    let started = Instant::now();
    let layout = match session.scan(std::slice::from_ref(&window)) {
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
    if std::env::args().nth(1).as_deref() == Some("noread") {
        println!("leaving the image unread");
        return Ok(());
    }

    println!(
        "expecting {} x {} at pitch {}, {} bytes",
        layout.pixels,
        layout.lines,
        layout.pitch,
        layout.total_bytes()
    );

    let started = Instant::now();
    let mut got = 0u64;
    let mut first = Vec::new();
    let mut chunks = session.image_chunks(&layout)?;
    while let Some(chunk) = chunks.next() {
        let chunk = chunk?;
        if first.is_empty() {
            first = chunk[..chunk.len().min(16)].to_vec();
        }
        got += chunk.len() as u64;
    }
    println!(
        "read {got} of {} bytes in {:?}",
        layout.total_bytes(),
        started.elapsed()
    );

    let words: Vec<u16> = first
        .chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    println!("first pixels: {words:?}");

    Ok(())
}
