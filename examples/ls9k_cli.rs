//! An example binary for a CLI program that can scan from a Coolscan 9000 ED

use anyhow::{Result, bail};
use clap::Parser;
use image::ImageFormat;
use indicatif::ProgressBar;
use nkscan::{
    scanners::ls9000ed::{
        BaseQuality, ChannelExposures, Dpi, Ls9000ed, Metering, ScanSettings,
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

    // Open the scanner
    let sg = SgDevice::open(cli.scanner)?;
    let mut scanner = Ls9000ed::new(sg)?;
    info!("Connected to LS9000ED");

    // Perform the inital calibration
    scanner.calibrate(ChannelExposures::default())?;

    // Perform the initial sweep to grab the thumbnails
    let overview = scanner.overview(ChannelExposures::default())?;

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
        let (gain, _) = scanner.autoexpose(
            &ae_settings,
            ChannelExposures::default(),
            &Metering {
                target: 58_000, // Target channel ADC counts
                percentile: 0.999,
                passes: 1,
                lock_white_balance: cli.lock_wb,
            },
        )?;

        // Finally perform a scan with these gains
        info!("Scanning frame {}", idx);
        let scan_settings = ScanSettings {
            dpi: Dpi::_4000,
            quality: BaseQuality::Scan,
            ir: cli.ir,
            ..ae_settings
        };
        let scan = scanner.scan_image(&scan_settings, gain)?;

        // Write the output
        let mut out = BufWriter::new(File::create(format!("scan{}.tiff", idx))?);
        scan.rgb.write_to(&mut out, ImageFormat::Tiff)?;

        if let Some(ir) = scan.ir {
            let mut out = BufWriter::new(File::create(format!("scan{}_ir.tiff", idx))?);
            ir.write_to(&mut out, ImageFormat::Tiff)?;
        }
    }

    Ok(())
}
