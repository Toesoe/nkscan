//! Locate the frames on a strip, then meter and scan one of them.
//!
//! The 83-DPI overview locates frames and is fixed at that resolution, so the frame window is
//! the only imaging path. It covers at most the device's `boundary_y`, 13176 dots or 83.7 mm.
//!
//! Gain persists in the scanner across sessions, so metering never starts from what is staged:
//! it starts from [`BASE`] every time, or each run would compound the last.

use image::{
    ImageFormat,
    imageops::{self, FilterType},
};
use nkscan::{
    decode::StreamDecoder,
    scanners::{
        FilmHolder, Focus, Scanner,
        ls9000ed::{
            CcdMode, Channel, ChannelExposures, Dpi, Ls9000ed, Multisample, POLL_INTERVAL,
            ScanArea, ScanSettings,
            boundaries::FrameBoundaries,
            calibration::meter,
            decode::{FrameDecoder, OverviewDecoder, Rgb16},
            holder::Holder,
            status::Status,
            window::{BaseQuality, WindowKind, WindowParams},
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

/// Metering geometry: cheap, and gain is what we are after rather than pixels. This is the
/// 666x333 prescan Nikon Scan meters on.
const METER_DPI: Dpi = Dpi::_666;
const METER_QUALITY: BaseQuality = BaseQuality::Preview;

/// Scan geometry. Nikon Scan meters at 666x333 and scans at 4000 on the same frame.
///
/// One 6x6 frame, RGB, no IR: 666 preview is about 7 MB, 4000 square about 523 MB. The latter
/// takes about 3 minutes, running at 2.65 MB/s against the preview's 310 KB/s: the scanner
/// streams as the stage steps, so more data per step is a higher rate.
const SCAN_DPI: Dpi = Dpi::_4000;
const SCAN_QUALITY: BaseQuality = BaseQuality::Scan;

/// Where metering starts, well under clipping so the first pass is measurable
const BASE: ChannelExposures = ChannelExposures {
    red: 71_890,
    green: 50_732,
    blue: 41_419,
    ir: 93_634,
};
/// Where to put the high tail of each channel. The ADC saturates at 65535.
///
/// Not closer: the second metering pass overshoots small corrections by about 5 percent, and
/// 62000 once landed blue at 65033. Clipping loses data for good, which is the whole point of
/// metering, so the margin is worth 0.1 stop.
const TARGET: u16 = 58_000;
/// Which sample counts as the high tail, so a few blown pixels do not set the gain
const PERCENTILE: f32 = 0.999;
/// One pass lands 3-10 percent under, so a second measures from where it actually got to.
/// That one tends to overshoot by about 5 percent, which [`TARGET`] leaves room for.
const METERING_PASSES: usize = 2;

/// Scan the window at these exposures and decode it
fn pass<T: Transport>(
    scanner: &mut Ls9000ed<T>,
    settings: &ScanSettings,
    exposures: ChannelExposures,
    chunk: u32,
) -> anyhow::Result<Rgb16> {
    let channels = Channel::RGB;
    for channel in channels {
        let params = WindowParams {
            ccd: settings.ccd_mode,
            multisample: settings.multisample,
            quality: settings.quality,
            window_kind: WindowKind::Frame,
            exposure: exposures.get(channel),
        };
        scanner.set_window(
            channel,
            params.descriptor(settings.dpi.to_dpi(), settings.window),
        )?;
    }

    scanner.scan(&channels)?;
    scanner.wait_until_ready()?;

    let mut decoder = FrameDecoder::new(settings)?;
    let mut last = 0;
    scanner.read_into_with(&mut decoder, chunk, |received, expected| {
        let percent = received * 100 / expected;
        if percent >= last + 25 {
            last = percent;
            debug!(percent, "Reading");
        }
    })?;

    let view = decoder.finish()?;
    Ok(
        Rgb16::from_raw(view.rgb.width(), view.rgb.height(), view.rgb.to_vec())
            .expect("view is well formed"),
    )
}

/// The level `meter` saw, per channel, so the log shows what it acted on
fn levels(image: &Rgb16, percentile: f32) -> [u16; 3] {
    let mut out = [0u16; 3];
    for (channel, level) in out.iter_mut().enumerate() {
        let mut samples: Vec<u16> = image.pixels().map(|p| p.0[channel]).collect();
        samples.sort_unstable();
        let at = (samples.len().saturating_sub(1) as f32 * percentile) as usize;
        *level = samples[at];
    }
    out
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Boxed so the backend could be chosen at runtime
    let sg = SgDevice::open("/dev/sg4")?;
    let chunk = sg.max_transfer();
    let transport: Box<dyn Transport> = Box::new(sg);
    let mut scanner = Ls9000ed::new(transport)?;
    let capabilities = scanner.capabilities();
    info!(?capabilities, chunk, "Scanner");

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

    info!(staged = ?scanner.channel_exposures()?, base = ?BASE, "Gain");
    scanner.calibrate(BASE)?;

    // The 83-DPI thumbnail, only to find where the frames sit
    let channels = Channel::RGB;
    for channel in channels {
        let params = WindowParams {
            ccd: CcdMode::SingleLine,
            multisample: Multisample::X1,
            quality: BaseQuality::Scan,
            window_kind: WindowKind::Overview,
            exposure: BASE.get(channel),
        };
        scanner.set_window(channel, params.descriptor(83, ScanArea::overview()))?;
    }
    info!("Thumbnail");
    scanner.scan(&channels)?;
    scanner.wait_until_ready()?;

    let mut overview = OverviewDecoder::new();
    scanner.read_into(&mut overview, chunk)?;
    let thumbnail = overview.finish()?;

    let Some(found) = FrameBoundaries::detect(&thumbnail, FRAMES_ON_STRIP) else {
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
    info!(FRAME, ?metering, ?settings, "Windows");

    let before = scanner.focus()?;
    let after = scanner.autofocus(frame.center())?;
    info!(point = ?frame.center(), before, after, "Autofocused");

    let mut exposures = BASE;
    for attempt in 0..METERING_PASSES {
        let image = pass(&mut scanner, &metering, exposures, chunk)?;
        let metered = meter(&image, exposures, PERCENTILE, TARGET);
        info!(
            attempt,
            saw = ?levels(&image, PERCENTILE),
            from = ?exposures,
            to = ?metered,
            "Metered"
        );
        exposures = metered;
    }

    // Metered at 666x333, scanned at 4000, and measured at 0.95-0.99, so gain carries between
    // the two geometries unchanged. Logged to catch that changing.
    let scanned = pass(&mut scanner, &settings, exposures, chunk)?;
    let landed = levels(&scanned, PERCENTILE);
    let correction = landed.map(|l| f32::from(TARGET) / f32::from(l.max(1)));
    info!(?landed, target = TARGET, ?correction, "Scanned");

    // A preview steps the stage at half the sensor rate, so its pixels are 2:1. A square Scan
    // needs no stretch, and at 4000 DPI resampling 39M pixels to get the same image back is
    // worth skipping.
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
