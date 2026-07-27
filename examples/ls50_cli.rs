//! Drive a real LS-50 ED through a scan and write 16-bit linear TIFFs.
//!
//! One frame by default; `--frames N` scans a loaded strip and `--frames 0` takes the count the
//! adapter reports. Each frame becomes `<basename>_<n>.tiff`, with any infrared plane beside it
//! as `<basename>_<n>_ir.tiff`.

use anyhow::{Result, bail};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use nkscan::{
    output,
    scanners::{
        FilmHolder, Focus, Scanner,
        ls50ed::{
            ChannelExposures, Dpi, Ls50ed, PRODUCT_ID, ScanSettings, VENDOR_ID,
            boundaries::FrameBoundaries, decode::Image, native_dots,
        },
    },
    scsi::usb::UsbTransport,
};
use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};
use tracing::*;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Where to write, as a path prefix. Each frame becomes <basename>_<n>.tiff, and its
    /// infrared mask <basename>_<n>_ir.tiff
    #[arg(long, default_value = "scan")]
    basename: PathBuf,
    /// Resolution in DPI. One of the firmware's divisions of the 4000-DPI sensor.
    #[arg(long, default_value = "4000", value_parser = dpi_mode)]
    dpi: Dpi,
    /// Frames on the loaded strip: 1 for a single frame, 0 to take the adapter's count
    ///
    /// Every one of them is declared to the scanner whether or not it gets scanned, since a
    /// pass that cannot see the whole table leaves the feed where it is.
    #[arg(long, default_value_t = 1)]
    frames: u32,
    /// Which of those frames to actually scan, zero-indexed, comma separated. All by default.
    #[arg(long, value_delimiter = ',')]
    frame: Vec<usize>,
    /// Capture the infrared plane for dust removal
    #[arg(long)]
    ir: bool,
    /// Run the autoexposure pre-pass
    #[arg(long)]
    ae: bool,
    /// Run firmware autofocus at the frame center
    #[arg(long)]
    af: bool,
    /// Fixed focus setpoint. Ignored with --af; without either the motor parks at 0.
    #[arg(long)]
    focus: Option<u16>,
    /// Where each frame starts along the feed axis, in mm, comma separated, last value
    /// repeating. One per frame, since the feed does not place them evenly.
    ///
    /// Measure rather than guess: scan at zero and read off how far down each window the
    /// interframe gap sits. Check both edges, since a gap at the bottom means too far along.
    /// Expect the shape `0,5.6`, but the figures belong to the film and the load.
    #[arg(long, value_delimiter = ',', default_values_t = [0.0])]
    offset: Vec<f32>,
    /// Override the frame pitch, in mm. Omitted, the adapter's reported pitch is used, which
    /// is what advances the film. Zero holds every window in one place.
    #[arg(long)]
    pitch: Option<f32>,
    /// Eject the film once the batch is done
    #[arg(long)]
    eject: bool,
}

/// A DPI figure the firmware will actually scan at, rather than one it would have to round
fn dpi_mode(text: &str) -> Result<Dpi, String> {
    let requested: u16 = text
        .parse()
        .map_err(|_| format!("{text} is not a number"))?;
    Dpi::ALL
        .into_iter()
        .find(|mode| mode.to_dpi() == requested)
        .ok_or_else(|| {
            let legal: Vec<String> = Dpi::ALL.iter().map(|m| m.to_dpi().to_string()).collect();
            format!("expected one of {}", legal.join(", "))
        })
}

/// A bar that fills as a pass is read off the scanner
fn reading(what: &str) -> ProgressBar {
    let bar = ProgressBar::no_length().with_message(what.to_owned());
    bar.set_style(
        ProgressStyle::with_template("{msg:>12} [{bar:40}] {bytes}/{total_bytes} {eta}")
            .expect("template is valid")
            .progress_chars("=> "),
    );
    bar
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let args = Args::parse();

    let transport = UsbTransport::open(VENDOR_ID, PRODUCT_ID)?;
    let mut scanner = Ls50ed::new(transport)?;

    info!(
        identity = ?scanner.identify()?,
        holder = ?scanner.holder()?,
        adapter = ?scanner.adapter_name(),
        "Scanner open"
    );
    let capabilities = scanner.capabilities();
    let frames = match args.frames {
        0 => scanner.sensed_frames().max(1),
        n => n,
    };

    scanner.warm_up()?;

    let pitch = args.pitch.map_or(capabilities.frame_pitch, native_dots);
    let boundaries =
        FrameBoundaries::evenly_spaced(frames, pitch, &args.offset, capabilities.max_x());

    // Which frames to take windows from. The table above still declares all of them.
    let selected: Vec<usize> = if args.frame.is_empty() {
        (0..boundaries.0.len()).collect()
    } else {
        args.frame.clone()
    };
    if let Some(past_end) = selected.iter().find(|&&i| i >= boundaries.0.len()) {
        bail!("frame {past_end} is not on a {frames}-frame strip");
    }

    // Measured once on the first frame scanned, reused for the rest of the strip
    let mut gain = ChannelExposures::default();

    for (nth, &index) in selected.iter().enumerate() {
        let settings = ScanSettings {
            dpi: args.dpi,
            ir: args.ir,
            samples: 1,
            window: boundaries.0[index].scan_area(capabilities),
            capabilities,
        };
        if nth == 0 {
            info!(
                resolution = settings.res(),
                dimensions = ?settings.output_dims(),
                frames,
                "Scanning"
            );
        }

        scanner.set_frame_boundaries(&boundaries)?;

        if args.af {
            info!(frame = index, "Autofocusing");
            scanner.autofocus(settings.center())?;
        } else {
            scanner.set_focus(args.focus.unwrap_or(0))?;
        }

        if args.ae && nth == 0 {
            info!("Metering");
            gain = scanner.autoexpose(&settings, gain)?;
        }

        let bar = reading("Scanning");
        let frame = scanner.scan_image_with(&settings, gain, |read, total| {
            bar.set_length(total);
            bar.set_position(read);
        })?;
        bar.finish_and_clear();

        let path = args.basename.with_file_name(format!(
            "{}_{index}.tiff",
            args.basename
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ));
        write_frame(&frame, &path)?;
    }

    if args.eject {
        scanner.eject()?;
        info!("Ejected");
    }
    Ok(())
}

fn write_frame(frame: &Image, path: &Path) -> Result<()> {
    output::write_rgb16_tiff(&mut BufWriter::new(File::create(path)?), &frame.rgb)?;
    info!(path = %path.display(), dimensions = ?frame.rgb.dimensions(), "Wrote TIFF");

    if let Some(ir) = &frame.ir {
        let ir_path = path.with_file_name(format!(
            "{}_ir.tiff",
            path.file_stem().unwrap_or_default().to_string_lossy()
        ));
        output::write_luma16_tiff(&mut BufWriter::new(File::create(&ir_path)?), ir)?;
        info!(path = %ir_path.display(), "Wrote the infrared plane");
    }
    Ok(())
}
