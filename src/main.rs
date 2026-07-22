use nkscan::{
    scanners::ls9000ed::{Ls9k, status::Status},
    scsi::linux::SgDevice,
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

    let mut last_status = scanner.status().unwrap();
    info!("Scanner state at program start: {:#?}", last_status);

    loop {
        let this_status = scanner.status().unwrap();
        if this_status != last_status {
            info!("Scanner status has changed: {:#?}", this_status);
            if this_status == Status::Ready {
                info!(
                    "Scanner ready with film holder state: {:#?}",
                    scanner.holder().unwrap()
                )
            }
        }
        last_status = this_status;
    }
}
