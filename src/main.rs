use nkscan::{
    scanners::ls9000ed::{Ls9k, status::Status},
    scsi::{
        linux::SgDevice,
        mode_pages::{BasicUnit, MeasurementUnits},
    },
};
use tracing::*;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let transport = SgDevice::open("/dev/sg4").unwrap();
    let mut scanner = Ls9k::new(transport);

    scanner.reserve().unwrap();

    let mut last_status = scanner.status().unwrap();
    info!("Scanner state at program start: {:#?}", last_status);

    // Set to 4000 units per inch as the "point" measurement unit
    scanner
        .set_measurement_units(MeasurementUnits {
            basic_unit: BasicUnit::Inches,
            divisor: 4000,
        })
        .unwrap();

    loop {
        let this_status = scanner.status().unwrap();
        if this_status != last_status {
            info!("Scanner status has changed: {:#?}", this_status);
            if this_status == Status::Ready {
                info!(
                    "Scanner ready with film holder state: {:#?}",
                    scanner.holder().unwrap()
                );

                let focus_before = scanner.get_focus().unwrap();
                info!("Focus position before set: {focus_before}");

                scanner.set_focus(0).unwrap();

                let focus_after = scanner.get_focus().unwrap();
                info!("Focus position after set: {focus_after}");
            }
        }
        last_status = this_status;
    }
}
