use nkscan::{
    scanners::ls9000ed::{Channel, Ls9000ed, holder::Holder, status::Status, window::WindowParams},
    scsi::{
        cdbs::{CompressionType, ImageCompositionCode, PaddingType, WindowDescriptor},
        linux::SgDevice,
    },
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

    // let standard = WindowDescriptor {
    //     id: 0, // overwritten per-channel by set_window()
    //     auto: false,
    //     x_resolution: 4000,
    //     y_resolution: 4000,
    //     x_upper_left: 0,
    //     y_upper_left: 0,
    //     width: 8964,
    //     length: 13176,
    //     brightness: 0,
    //     threshold: 0,
    //     contrast: 0,
    //     composition: ImageCompositionCode::Rgb,
    //     bits_per_pixel: 16,
    //     halftone_pattern: 0,
    //     rif: false,
    //     padding: PaddingType::NoPadding,
    //     bit_ordering: 0,
    //     compression: CompressionType::NoCompression,
    //     compression_arg: 0,
    //     vendor: vec![],
    // };
    // let descriptor = WindowParams::OVERVIEW_SCAN.apply_to(standard);

    // let channels = [Channel::Red, Channel::Green, Channel::Blue];
    // for channel in channels {
    //     info!(?channel, "Setting window");
    //     scanner.set_window(channel, descriptor.clone())?;
    // }

    // info!("Triggering scan");
    // scanner.scan(&channels)?;

    Ok(())
}
