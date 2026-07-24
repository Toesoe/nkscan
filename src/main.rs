use nkscan::{
    scanners::ls9000ed::{Channel, Ls9000ed, holder::Holder, status::Status},
    scsi::linux::SgDevice,
};
use tracing::*;

fn main() -> anyhow::Result<()> {
    // Set up tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Open scanner device
    let transport = SgDevice::open("/dev/sg4")?;
    let mut scanner = Ls9000ed::new(transport)?;

    // Block until we have a film holder and the scanner is ready
    info!("Waiting for scanner to be ready");
    loop {
        let this_status = scanner.status()?;
        let holder = scanner.holder()?;
        if this_status == Status::Ready && (holder != Holder::None) {
            info!("Scanner ready with film holder: {:#?}", holder);
            break;
        }
    }

    // GET_WINDOW for all channels
    info!("All Channels: {:#?}", scanner.get_window(None)?);

    Ok(())
}
