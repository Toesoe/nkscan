//! REMOVE LATER: is C1h's boundary a limit or just the holder's opening?
//!
//! Sends back a descriptor the unit already holds. Its own defaults are wider
//! than the boundary it reports, so accepting one proves the boundary is advisory

use std::time::Duration;

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
    session.test_unit_ready(Duration::from_secs(60))?;

    let caps = session.capabilities();
    println!(
        "aperture {} x {}, sensor {}",
        caps.address.x_axis.boundary, caps.address.y_axis.boundary, caps.address.ccd_pixels
    );

    let window = session
        .windows()?
        .into_iter()
        .find(|w| w.id == 1)
        .expect("the red window");
    println!(
        "echoing back id {} {:?} {:?}",
        window.id, window.origin, window.size
    );

    match session.set_window(&window) {
        Ok(()) => println!("ACCEPTED -- the boundary is advisory"),
        Err(e) => println!("REFUSED  -- {e}"),
    }

    for w in session.windows()? {
        println!("  {} {:?} {:?}", w.id, w.origin, w.size);
    }

    Ok(())
}
