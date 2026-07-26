//! Locate the frames on a strip, then meter and scan one of them.
//!
//! The 83-DPI overview locates frames and is fixed at that resolution, so the frame window is
//! the only imaging path. It covers at most the device's `boundary_y`, 13176 dots or 83.7 mm.

use image::{
    ImageFormat,
    imageops::{self, FilterType},
};
use nkscan::{
    scanners::{
        FilmHolder, Focus, Scanner,
        ls9000ed::{
            CcdMode, ChannelExposures, Dpi, Ls9000ed, Metering, Multisample, POLL_INTERVAL,
            ScanSettings, boundaries::FrameBoundaries, holder::Holder, status::Status,
            window::BaseQuality,
        },
    },
    scsi::{Transport, linux::SgDevice},
};
use std::{fs::File, io::BufWriter, thread::sleep};
use tracing::*;

/// How many frames the strip holds. This is what the detector fits, not how many to scan:
/// telling it 1 makes it return a single frame spanning everything.
const FRAMES_ON_STRIP: usize = 3;
/// Which of them to scan
const FRAME: usize = 0;

/// Metering geometry, the 666x333 pass Nikon Scan meters on
const METER_DPI: Dpi = Dpi::_666;
const METER_QUALITY: BaseQuality = BaseQuality::Preview;

/// Scan geometry. Gain carries between the two unchanged, so metering stays cheap.
///
/// One 6x6 frame, RGB, no IR: 666 preview is about 7 MB, 4000 square about 523 MB. The latter
/// takes about 3 minutes, running at 2.65 MB/s against the preview's 310 KB/s: the scanner
/// streams as the stage steps, so more data per step is a higher rate.
const SCAN_DPI: Dpi = Dpi::_4000;
const SCAN_QUALITY: BaseQuality = BaseQuality::Scan;

/// Where metering starts, well under clipping so the first pass is measurable. Not read from
/// the scanner: gain persists there, so a run would compound the last.
const BASE: ChannelExposures = ChannelExposures {
    red: 71_890,
    green: 50_732,
    blue: 41_419,
    ir: 93_634,
};

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
    info!(capabilities = ?scanner.capabilities(), "Scanner");

    // Block until we have a film holder and the scanner is ready
    info!("Waiting for scanner to be ready");
    loop {
        let this_status = scanner.status()?;
        let holder = scanner.holder()?;
        if this_status == Status::Ready && (holder != Holder::None) {
            info!("Scanner ready with film holder: {:#?}", holder);
            break;
        }
        // Unlike the other waits this one has no bound, since it is waiting on a person
        sleep(POLL_INTERVAL);
    }

    scanner.calibrate(BASE)?;

    let overview = scanner.overview(BASE)?;
    let Some(found) = FrameBoundaries::detect(&overview, FRAMES_ON_STRIP) else {
        anyhow::bail!("no frames found on the strip");
    };
    for (i, rect) in found.0.iter().enumerate() {
        info!(
            frame = i,
            y_top = rect.y_top,
            length = rect.y_bottom - rect.y_top,
            "Found frame"
        );
    }
    scanner.set_frame_boundaries(&found)?;

    let frame = *found
        .0
        .get(FRAME)
        .ok_or_else(|| anyhow::anyhow!("strip has no frame {FRAME}"))?;

    // Autofocus aims against the frame table, and every capture has the window equal to it
    let before = scanner.focus()?;
    let after = scanner.autofocus(frame.center())?;
    info!(FRAME, point = ?frame.center(), before, after, "Autofocused");

    let metering = ScanSettings {
        ccd_mode: CcdMode::ThreeLine,
        ir: false,
        dpi: METER_DPI,
        quality: METER_QUALITY,
        multisample: Multisample::X1,
        window: frame.scan_area(),
    };
    let settings = ScanSettings {
        dpi: SCAN_DPI,
        quality: SCAN_QUALITY,
        ..metering
    };

    let (gain, _preview) = scanner.autoexpose(&metering, BASE, &Metering::default())?;
    info!(?gain, "Metered");

    let scanned = scanner.scan_image(&settings, gain)?.rgb;

    // A preview steps the stage at half the sensor rate, so its pixels are 2:1. A square Scan
    // needs no stretch, and resampling 39M pixels to get the same image back is worth skipping.
    let (width, height) = scanned.dimensions();
    let square = width * settings.stage_divisor() / settings.dpi.divisor();
    let mut out = BufWriter::new(File::create("scan.tiff")?);
    if square == width {
        scanned.write_to(&mut out, ImageFormat::Tiff)?;
    } else {
        imageops::resize(&scanned, square, height, FilterType::Triangle)
            .write_to(&mut out, ImageFormat::Tiff)?;
    }
    info!(sampled = ?(width, height), dimensions = ?(square, height), "Wrote scan.tiff");

    Ok(())
}
