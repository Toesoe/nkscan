//! Drive a real LS-9000ED through calibration and a scan.

use image::ImageFormat;
use nkscan::{
    decode::StreamDecoder,
    scanners::{
        FilmHolder, Scanner,
        ls9000ed::{
            CcdMode, Channel, Ls9000ed, Multisample, ScanArea,
            boundaries::FrameBoundaries,
            decode::OverviewDecoder,
            holder::Holder,
            status::Status,
            window::{BaseQuality, WindowKind, WindowParams},
        },
    },
    scsi::{Transport, linux::SgDevice},
};
use std::{fs::File, io::BufWriter, thread::sleep, time::Duration};
use tracing::*;

/// How many frames the strip holds, which is all the detector needs
const FRAME_COUNT: usize = 3;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Boxed so the backend could be chosen at runtime
    let transport: Box<dyn Transport> = Box::new(SgDevice::open("/dev/sg4")?);
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

    let exposures = scanner.channel_exposures()?;
    info!(?exposures, "Exposures staged in the scanner");

    scanner.calibrate(exposures)?;
    info!("Calibrated");

    // The 83-DPI thumbnail: the whole strip in one pass, single-line CCD, RGB
    let channels = Channel::RGB;
    for channel in channels {
        let params = WindowParams {
            ccd: CcdMode::SingleLine,
            multisample: Multisample::X1,
            quality: BaseQuality::Scan,
            window_kind: WindowKind::Overview,
            exposure: exposures.get(channel),
        };
        info!(?channel, "Setting overview window");
        scanner.set_window(channel, params.descriptor(83, ScanArea::overview()))?;
    }

    info!("Triggering thumbnail scan");
    scanner.scan(&channels)?;

    // The scanner reports NotReady for the whole pass
    while scanner.status()? != Status::Ready {
        sleep(Duration::from_millis(200));
    }
    // Nikon Scan reads the windows back here before pulling the image
    info!(exposures = ?scanner.channel_exposures()?, "Scan finished");

    let mut decoder = OverviewDecoder::new();

    let line = ScanArea::overview_dims().0 * 3 * 2;
    let chunk = line * (32 * 1024 / line);

    let expected = decoder.expected_bytes();
    info!(expected, chunk, "Reading image");

    let mut last_percent = 0;
    scanner.read_into_with(&mut decoder, chunk, |received, expected| {
        let percent = received * 100 / expected;
        if percent >= last_percent + 10 {
            last_percent = percent;
            info!(percent, "Reading");
        }
    })?;

    let image = decoder.finish()?;
    let mut out = BufWriter::new(File::create("thumbnail.tiff")?);
    image.write_to(&mut out, ImageFormat::Tiff)?;
    info!(dimensions = ?image.dimensions(), "Wrote thumbnail.tiff");

    // Where the frames actually landed, replacing the nominal table calibration wrote. Nikon
    // Scan does the same once its own overview has located them.
    let Some(found) = FrameBoundaries::detect(&image, FRAME_COUNT) else {
        anyhow::bail!("no frames found on the strip");
    };
    for (i, rect) in found.0.iter().enumerate() {
        info!(
            frame = i,
            y_top = rect.y_top,
            y_bottom = rect.y_bottom,
            length = rect.y_bottom - rect.y_top,
            "Found frame"
        );
    }
    scanner.set_frame_boundaries(&found)?;
    info!("Frame table written");

    Ok(())
}
