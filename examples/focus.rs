//! REMOVE LATER: drive the focus mechanism
//!
//! ```text
//! cargo run --example focus                # focus moves only, lens alone
//! cargo run --example focus -- auto        # also autofocus, which can move the stage
//! ```

use std::time::{Duration, Instant};

use nkscan::{
    device::{self, Selector},
    session::Session,
};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let auto = std::env::args().nth(1).as_deref() == Some("auto");

    let devices = device::list();
    let device = Selector::Only.resolve(&devices)?;
    let mut session = Session::open(device.open()?)?;
    session.test_unit_ready(Duration::from_secs(60))?;

    let caps = session.capabilities();
    let range = caps.address.focus_range;
    let ccd = u32::from(caps.address.ccd_pixels);
    println!("focus range {} to {}", range.start, range.last);

    // Values Nikon Scan has been seen to send, plus the ends of the range
    for position in [226u16, 174, range.start, range.last, 226] {
        let started = Instant::now();
        let outcome = session.focus_to(position);
        println!(
            "  focus_to({position}) {:?} in {:?}",
            outcome,
            started.elapsed()
        );
        if outcome.is_err() {
            break;
        }
    }

    if auto {
        let y = session
            .windows()?
            .first()
            .map_or(0, |w| w.origin.1 + w.size.1 / 2);
        let x = ccd / 2;
        println!("autofocus at {x}, {y}");
        let started = Instant::now();
        println!(
            "  {:?} in {:?}",
            session.autofocus(x, y, None),
            started.elapsed()
        );
    }

    Ok(())
}
