//! An example binary for a CLI program that can scan from a Coolscan 9000 ED

use anyhow::{Result, bail};
use clap::Parser;
use image::ImageFormat;
use indicatif::{ProgressBar, ProgressStyle};
use nkscan::{
    scanners::ls9000ed::{
        BaseQuality, CcdMode, ChannelExposures, Dpi, Ls9000ed, Metering, Multisample, ScanSettings,
        boundaries::FrameBoundaries,
    },
    scsi::linux::SgDevice,
};
use std::{fs::File, io::BufWriter, path::PathBuf};
use tracing::*;

#[derive(Parser)]
#[command(version, about)]
/// Automatically scans each medium format frame from a strip with the Coolscan 9000 on Linux at 4000 DPI
struct Cli {
    /// Linux /dev/sg* for the scanner
    scanner: PathBuf,
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

fn main() -> Result<()> {
    // Set up tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    // Validate inputs
    if cli.frames < 1 {
        bail!("Frames must be a positive integer");
    }

    if let Some(frame) = cli.frame
        && frame >= cli.frames
    {
        bail!("Selected frame must lie within the number of frames");
    }

    let multisample = match cli.multisample {
        1 => Multisample::X1,
        2 => Multisample::X2,
        4 => Multisample::X4,
        8 => Multisample::X8,
        16 => Multisample::X16,
        _ => bail!("Multisample must be one of 1,2,4,8,16"),
    };

    let ccd_mode = if cli.singleline {
        CcdMode::SingleLine
    } else {
        CcdMode::ThreeLine
    };

    // Open the scanner
    let sg = SgDevice::open(cli.scanner)?;
    let mut scanner = Ls9000ed::new(sg)?;
    info!("Connected to LS9000ED");

    // Perform the inital calibration
    scanner.calibrate(ChannelExposures::default())?;

    // Perform the initial sweep to grab the thumbnails
    info!("Performing overview scan to find frames");
    let bar = reading("Overview");
    let overview = scanner.overview_with(ChannelExposures::default(), |read, total| {
        bar.set_length(total);
        bar.set_position(read);
    })?;
    bar.finish_and_clear();

    // Detect the frames within the overview
    let Some(frames) = FrameBoundaries::detect(&overview, cli.frames) else {
        bail!("No frames found on strip!");
    };
    info!("Found frames");

    // Tell the scanner about the frame boundaries (it caches details we write about each frame)
    scanner.set_frame_boundaries(&frames)?;

    // For each frame, we need to autofocus, autoexpose, scan, and save to disk
    let frame_idxs = if let Some(frame) = cli.frame {
        vec![frame]
    } else {
        (0..cli.frames).into_iter().collect()
    };

    for idx in frame_idxs {
        let frame = frames.0[idx];
        // First autofocus at the center of the frame
        info!("Performing AF for frame {}", idx);
        scanner.autofocus(frame.center())?;

        // Then perform autoexposure
        info!("Performing AE for frame {}", idx);
        let ae_settings = ScanSettings::autoexposure(frame.scan_area());
        let bar = reading("Metering");
        let (gain, _) = scanner.autoexpose_with(
            &ae_settings,
            ChannelExposures::default(),
            &Metering {
                target: 58_000, // Target channel ADC counts
                percentile: 0.999,
                passes: 2,
                lock_white_balance: cli.lock_wb,
            },
            |read, total| {
                bar.set_length(total);
                bar.set_position(read);
            },
        )?;
        bar.finish_and_clear();

        // Finally perform a scan with these gains
        info!("Scanning frame {}", idx);
        let scan_settings = ScanSettings {
            dpi: Dpi::_4000,
            quality: BaseQuality::Scan,
            ir: cli.ir,
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
        let path = cli.basename.with_file_name(format!(
            "{}_{idx}.tiff",
            cli.basename
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
