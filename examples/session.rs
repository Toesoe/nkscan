//! REMOVE LATER: testing transport stuff

use std::time::Duration;

use nkscan::{
    device::{self, Selector},
    protocol::{data::DataType, session::Session},
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
    println!(
        "holder {:?} bounds {} x {}",
        session.capabilities().address.holder_id,
        session.capabilities().address.x_axis.boundary,
        session.capabilities().address.y_axis.boundary
    );

    for w in session.windows()? {
        println!("{} -> {:?}", w.id, w.validate(session.capabilities()));
    }

    // None of these need a scan to have run first
    for kind in [
        DataType::LeakVolume,
        DataType::WhiteBalanceExposure,
        DataType::AnalogGain,
        DataType::Boundary,
    ] {
        match session.read_data(kind, 0) {
            Ok((h, values)) => println!("{kind:?} bits={} {values:?}", h.bits),
            Err(e) => println!("{kind:?} -> {e}"),
        }
    }

    Ok(())
}
