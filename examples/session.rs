//! REMOVE LATER: testing transport stuff

use std::time::Duration;

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
    let transport = device.open()?;
    let mut session = Session::open(transport)?;

    session.test_unit_ready(Duration::from_secs(30))?;

    println!("units: {}", session.units()?);

    for w in session.windows()? {
        println!("{} -> {:?}", w.id, w.validate(session.capabilities()));
    }

    Ok(())
}
