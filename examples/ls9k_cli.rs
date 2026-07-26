//! An example binary for a CLI program that can scan from a Coolscan 9000 ED

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use image::ImageFormat;
use indicatif::{ProgressBar, ProgressStyle};
use nkscan::{
    scanners::{
        Scanner,
        ls9000ed::{
            BaseQuality, CcdMode, ChannelExposures, Dpi, Ls9000ed, Metering, Multisample,
            ScanSettings, boundaries::FrameBoundaries,
        },
    },
    scsi::linux::SgDevice,
};
use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};
use tracing::*;

#[derive(Parser)]
#[command(version, about)]
/// Scans medium format film with the Coolscan 9000 ED on Linux
struct Cli {
    /// Linux /dev/sg* for the scanner
    scanner: PathBuf,
    /// Where the bare-light white balance lives, as `calibrate` writes it and `scan` reads it
    #[arg(long, default_value = "nkscan-wb.txt")]
    wb_file: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scan every frame from the loaded strip at 4000 DPI
    Scan(ScanArgs),
    /// Measure the backlight with an EMPTY holder loaded, and write the neutral gains
    ///
    /// Equal gain on every channel does not scan neutral, because the LEDs and the CCD are not
    /// equally strong across the three bands. This measures that imbalance so a scan can undo
    /// it, and what is left is the film. Run it once per scanner: the result is a property of
    /// the hardware, not of whatever is loaded.
    Calibrate,
}

#[derive(Args)]
struct ScanArgs {
    /// How many frames to expect in the film holder (needed for frame recognition)
    #[arg(long)]
    frames: usize,
    /// Whether to lock the white balance during autoexposure
    #[arg(long)]
    lock_wb: bool,
    /// Optional frame number (zero-indexed) to scan, otherwise scan all of them
    #[arg(long)]
    frame: Option<usize>,
    /// Save IR alongside the main scan
    #[arg(long)]
    ir: bool,
    /// Where to write, as a path prefix. Each frame becomes <basename>_<n>.tiff, and its
    /// infrared mask <basename>_<n>_ir.tiff
    #[arg(long, default_value = "scan")]
    basename: PathBuf,
    /// How much multisampling to perform. This increases scan time at the befenit of lower noise. One of 1,2,4,8,16.
    #[arg(long, default_value_t = 1)]
    multisample: usize,
    /// Single-line CCD mode. Slow, but may improve banding noise
    #[arg(long)]
    singleline: bool,
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

/// The white balance as `calibrate` leaves it: the three visible gains, in order
///
/// Infrared is not part of a white balance and is not measured, so it is not stored either.
fn write_white_balance(path: &Path, gains: ChannelExposures) -> Result<()> {
    let contents = format!("{} {} {}\n", gains.red, gains.green, gains.blue);
    std::fs::write(path, contents).with_context(|| format!("writing {}", path.display()))
}

/// `Ok(None)` when there is simply no calibration yet, which is not an error
fn read_white_balance(path: &Path) -> Result<Option<ChannelExposures>> {
    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
    };

    let values: Vec<u32> = contents
        .split_whitespace()
        .map(|field| field.parse::<u32>())
        .collect::<Result<_, _>>()
        .with_context(|| format!("{} is not three numbers", path.display()))?;
    let [red, green, blue] = values[..] else {
        bail!(
            "{} holds {} gains, expected 3",
            path.display(),
            values.len()
        );
    };

    Ok(Some(ChannelExposures {
        red,
        green,
        blue,
        ..ChannelExposures::default()
    }))
}

/// Measure the bare backlight and write it out
fn calibrate<T: nkscan::scsi::Transport>(scanner: &mut Ls9000ed<T>, wb_file: &Path) -> Result<()> {
    info!("Measuring the backlight. The holder must be loaded and EMPTY");
    let bar = reading("Backlight");
    let white = scanner.white_balance_with(
        &Metering {
            // Three passes, because the search deliberately starts far under
            passes: 3,
            ..Metering::default()
        },
        |read, total| {
            bar.set_length(total);
            bar.set_position(read);
        },
    )?;
    bar.finish_and_clear();

    write_white_balance(wb_file, white)?;
    info!(?white, path = ?wb_file, "Measured white balance");
    Ok(())
}

/// Find every frame on the strip, then focus, meter, scan and save each one
fn scan<T: nkscan::scsi::Transport>(
    scanner: &mut Ls9000ed<T>,
    wb_file: &Path,
    args: &ScanArgs,
) -> Result<()> {
    // Validate inputs
    if args.frames < 1 {
        bail!("Frames must be a positive integer");
    }

    if let Some(frame) = args.frame
        && frame >= args.frames
    {
        bail!("Selected frame must lie within the number of frames");
    }

    let multisample = match args.multisample {
        1 => Multisample::X1,
        2 => Multisample::X2,
        4 => Multisample::X4,
        8 => Multisample::X8,
        16 => Multisample::X16,
        _ => bail!("Multisample must be one of 1,2,4,8,16"),
    };

    let ccd_mode = if args.singleline {
        CcdMode::SingleLine
    } else {
        CcdMode::ThreeLine
    };

    // Where every gain search starts. Under --lock-wb this is the white balance outright,
    // since metering then scales all three channels by one factor and cannot change their
    // ratios. The built-in default is one scanner's measurement, so --wb-file is how you use
    // your own.
    let white = match read_white_balance(wb_file)? {
        Some(white) => {
            info!(?white, path = ?wb_file, "Using measured white balance");
            white
        }
        None => {
            let white = ChannelExposures::default();
            debug!(?white, path = ?wb_file, "No white balance measured, using the built-in one");
            white
        }
    };

    // Perform the initial sweep to grab the thumbnails
    info!("Performing overview scan to find frames");
    let bar = reading("Overview");
    let overview = scanner.overview_with(white, |read, total| {
        bar.set_length(total);
        bar.set_position(read);
    })?;
    bar.finish_and_clear();

    // Detect the frames within the overview
    let Some(frames) = FrameBoundaries::detect(&overview, args.frames) else {
        bail!("No frames found on strip!");
    };
    info!("Found frames");

    // Tell the scanner about the frame boundaries (it caches details we write about each frame)
    scanner.set_frame_boundaries(&frames)?;

    // For each frame, we need to autofocus, autoexpose, scan, and save to disk
    let frame_idxs = if let Some(frame) = args.frame {
        vec![frame]
    } else {
        (0..args.frames).collect()
    };

    for idx in frame_idxs {
        let frame = frames.0[idx];
        // First autofocus at the center of the frame
        info!("Performing AF for frame {}", idx);
        scanner.autofocus(frame.center())?;

        // Then perform autoexposure
        info!("Performing AE for frame {}", idx);
        let ae_settings = ScanSettings::autoexposure(frame.scan_area(), args.ir);
        let bar = reading("Metering");
        let (gain, _) = scanner.autoexpose_with(
            &ae_settings,
            white,
            &Metering {
                target: 58_000, // Target channel ADC counts
                percentile: 0.999,
                passes: 2,
                lock_white_balance: args.lock_wb,
            },
            |read, total| {
                bar.set_length(total);
                bar.set_position(read);
            },
        )?;
        bar.finish_and_clear();
        info!(idx, ?gain, "Metered");

        // Finally perform a scan with these gains
        info!("Scanning frame {}", idx);
        let scan_settings = ScanSettings {
            dpi: Dpi::_4000,
            quality: BaseQuality::Scan,
            ir: args.ir,
            multisample,
            ccd_mode,
            ..ae_settings
        };
        let bar = reading("Scanning");
        let scan = scanner.scan_image_with(&scan_settings, gain, |read, total| {
            bar.set_length(total);
            bar.set_position(read);
        })?;
        bar.finish_and_clear();

        // Write the output
        let path = args.basename.with_file_name(format!(
            "{}_{idx}.tiff",
            args.basename
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ));
        scan.rgb
            .write_to(&mut BufWriter::new(File::create(&path)?), ImageFormat::Tiff)?;
        info!(?path, "Wrote");

        if let Some(ir) = scan.ir {
            let path = path.with_file_name(format!(
                "{}_ir.tiff",
                path.file_stem().unwrap_or_default().to_string_lossy()
            ));
            ir.write_to(&mut BufWriter::new(File::create(&path)?), ImageFormat::Tiff)?;
            info!(?path, "Wrote infrared");
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    // Set up tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    // Open the scanner
    let sg = SgDevice::open(cli.scanner)?;
    let mut scanner = Ls9000ed::new(sg)?;
    let identity = scanner.identify()?;
    info!("Connected to {} {}", identity.vendor, identity.product);

    // Perform the inital calibration, which both passes need before their first scan
    scanner.calibrate(ChannelExposures::default())?;

    match &cli.command {
        Command::Scan(args) => scan(&mut scanner, &cli.wb_file, args),
        Command::Calibrate => calibrate(&mut scanner, &cli.wb_file),
    }
}
