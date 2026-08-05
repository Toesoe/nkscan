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
    protocol::session::Session,
};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let devices = device::list();
    let device = Selector::Only.resolve(&devices)?;
    let mut session = Session::open(device.open()?)?;
    session.test_unit_ready(Duration::from_secs(60))?;

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
    match session.scan(&[window.id]) {
        Ok(()) => {
            println!("scan started in {:?}", started.elapsed());
            let started = Instant::now();
            session.test_unit_ready(Duration::from_secs(180))?;
            println!("scan finished in {:?}", started.elapsed());

            // 2-10's formula: pitch is the optical resolution over the asked-for
            // one, and the pixel count is the window over the pitch
            let pitch =
                u32::from(session.capabilities().address.x_axis.optical_dpi) / u32::from(dpi);
            let (pixels, lines) = (window.size.0 / pitch, window.size.1 / pitch);
            let bytes_per_pixel = window.bpp / 8;
            let line = pixels as usize * bytes_per_pixel as usize;
            println!("expecting {pixels} x {lines}, {line} bytes a line");

            let mut buf = vec![0u8; line * lines as usize];
            let started = Instant::now();
            let got = session.read_image(&mut buf, line, bytes_per_pixel)?;
            println!(
                "read {got} of {} bytes in {:?}",
                buf.len(),
                started.elapsed()
            );

            let words: Vec<u16> = buf[..got.min(16)]
                .chunks_exact(2)
                .map(|c| u16::from_be_bytes([c[0], c[1]]))
                .collect();
            println!("first pixels: {words:?}");
        }
        Err(e) => println!("scan refused after {:?}: {e}", started.elapsed()),
    }

    Ok(())
}
