//! An example binary for a CLI program that can scan from a Coolscan 9000 ED

use anyhow::{Context, Result, bail};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use nkscan::{
    output,
    scanners::{
        FilmHolder, Scanner,
        ls9000ed::{
            BaseQuality, CcdMode, ChannelExposures, Dpi, Ls9000ed, Metering, Multisample,
            ScanSettings, boundaries::FrameBoundaries, holder::Holder,
        },
    },
};

#[cfg(target_os = "linux")]
use nkscan::scsi::linux::SgDevice as Device;
#[cfg(target_os = "windows")]
use nkscan::scsi::windows::ScsiScanDevice as Device;
use std::{fs::File, io::BufWriter, path::PathBuf, time::Duration};
use tracing::*;

#[derive(Parser)]
#[command(version, about)]
/// Scans medium format film with the Coolscan 9000 ED
struct Cli {
    /// Device path for the scanner: /dev/sg* on Linux, \\.\Scanner0 on Windows
    scanner: PathBuf,
    /// How many frames to expect in the film holder (needed for frame recognition)
    #[arg(long)]
    frames: usize,
    /// Optional frame number (zero-indexed) to scan, otherwise scan all of them
    #[arg(long)]
    frame: Option<usize>,
    /// Fixed per-channel analog gain as `red,green,blue[,ir]`, which turns autoexposure off
    #[arg(long, value_name = "R,G,B[,IR]", allow_hyphen_values = false)]
    gain: Option<String>,
    /// Whether to lock the white balance during autoexposure. Ignored with --gain.
    #[arg(long)]
    lock_wb: bool,
    /// Save IR alongside the main scan
    #[arg(long)]
    ir: bool,
    /// Where to write, as a path prefix. Each frame becomes <basename><n>.tiff, and its infrared mask <basename><n>_IR.tiff
    #[arg(long, default_value = "scan")]
    basename: PathBuf,
    /// How much multisampling to perform. This increases scan time at the befenit of lower noise. One of 1,2,4,8,16.
    #[arg(long, default_value_t = 1)]
    multisample: usize,
    /// Single-line CCD mode. Slow, but may improve banding noise
    #[arg(long)]
    singleline: bool,
    /// Send the holder back out when everything is done
    #[arg(long)]
    eject: bool,
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

/// `red,green,blue` with an optional infrared, which otherwise keeps its default
fn parse_gain(spec: &str) -> Result<ChannelExposures> {
    let values: Vec<u32> = spec
        .split(',')
        .map(|field| field.trim().parse::<u32>())
        .collect::<Result<_, _>>()
        .with_context(|| format!("{spec} is not a comma-separated list of numbers"))?;

    let default = ChannelExposures::default();
    Ok(match values[..] {
        [red, green, blue] => ChannelExposures {
            red,
            green,
            blue,
            ..default
        },
        [red, green, blue, ir] => ChannelExposures {
            red,
            green,
            blue,
            ir,
        },
        _ => bail!("--gain takes three or four values, got {}", values.len()),
    })
}

/// How often to look for a holder while waiting on one to be pushed in
const HOLDER_POLL: Duration = Duration::from_millis(500);

/// Block until a film holder is loaded, and report which one
///
/// Checked once up front rather than guarded at every call, on the assumption the holder stays
/// put for the run. Without one the scanner refuses frame windows as a bad parameter list,
/// which gives no hint that a missing holder is the reason.
fn wait_for_holder<T: nkscan::scsi::Transport>(scanner: &mut Ls9000ed<T>) -> Result<Holder> {
    // INQUIRY is not blocked by a pending unit attention, so this keeps answering across the
    // holder change that loading one raises
    let mut holder = scanner.holder()?;

    if holder == Holder::None {
        let bar = ProgressBar::new_spinner().with_message("Waiting for a film holder");
        bar.enable_steady_tick(HOLDER_POLL);
        while holder == Holder::None {
            std::thread::sleep(HOLDER_POLL);
            holder = scanner.holder()?;
        }
        bar.finish_and_clear();
    }

    // Loading one raises a holder change and a reset, and those would otherwise surface as a
    // CHECK CONDITION on whatever ran next
    scanner.drain_unit_attentions()?;

    // The page above reports the holder the moment it is detected, but the scanner spends
    // seconds afterwards coming up. That shows as NotReady/Initializing, which is not a unit
    // attention and so survives the drain, and the first real command then gets refused with
    // CommandSequenceError several seconds later.
    scanner.wait_until_ready()?;
    Ok(holder)
}

/// Find every frame on the strip, then focus, expose, scan and save each one
fn scan<T: nkscan::scsi::Transport>(scanner: &mut Ls9000ed<T>, cli: &Cli) -> Result<()> {
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

    let fixed = cli.gain.as_deref().map(parse_gain).transpose()?;
    if let Some(gain) = fixed {
        info!(
            ?gain,
            "Autoexposure off, scanning every frame at a fixed gain"
        );
    }

    // Where a gain search starts, and what the overview is taken at. The scanner's channels
    // are not equally sensitive, so this is the imbalance that makes a neutral subject scan
    // neutral rather than a starting point picked for a particular film.
    let white = ChannelExposures::default();

    // Perform the initial sweep to grab the thumbnails
    info!("Performing overview scan to find frames");
    let bar = reading("Overview");
    let overview = scanner.overview_with(white, |read, total| {
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

    // For each frame, we need to autofocus, expose, scan, and save to disk
    let frame_idxs = if let Some(frame) = cli.frame {
        vec![frame]
    } else {
        (0..cli.frames).collect()
    };

    for idx in frame_idxs {
        let frame = frames.0[idx];
        // First autofocus at the center of the frame. Still per frame even at a fixed gain:
        // film does not sit flat, so focus is not a property of the roll the way gain is.
        info!("Performing AF for frame {}", idx);
        scanner.autofocus(frame.center())?;

        let base = ScanSettings::autoexposure(frame.scan_area(), cli.ir);
        let gain = match fixed {
            Some(gain) => gain,
            None => {
                info!("Performing AE for frame {}", idx);
                let bar = reading("Metering");
                let (gain, _) = scanner.autoexpose_with(
                    &base,
                    white,
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
                info!(idx, ?gain, "Metered");
                gain
            }
        };

        // Finally perform a scan with these gains
        info!("Scanning frame {}", idx);
        let scan_settings = ScanSettings {
            dpi: Dpi::_4000,
            quality: BaseQuality::Scan,
            ir: cli.ir,
            multisample,
            ccd_mode,
            ..base
        };
        let bar = reading("Scanning");
        let scan = scanner.scan_image_with(&scan_settings, gain, |read, total| {
            bar.set_length(total);
            bar.set_position(read);
        })?;
        bar.finish_and_clear();

        // Write the output
        let path = cli.basename.with_file_name(format!(
            "{}{idx}.tiff",
            cli.basename
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ));
        output::write_rgb16_tiff(&mut BufWriter::new(File::create(&path)?), &scan.rgb)?;
        info!(?path, "Wrote");

        if let Some(ir) = scan.ir {
            let path = path.with_file_name(format!(
                "{}_IR.tiff",
                path.file_stem().unwrap_or_default().to_string_lossy()
            ));
            output::write_luma16_tiff(&mut BufWriter::new(File::create(&path)?), &ir)?;
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
    let sg = Device::open(&cli.scanner)?;
    let mut scanner = Ls9000ed::new(sg)?;
    let identity = scanner.identify()?;
    info!("Connected to {} {}", identity.vendor, identity.product);

    let holder = wait_for_holder(&mut scanner)?;
    info!(?holder, "Holder loaded");

    // The session preamble, which gates the first scan
    scanner.calibrate(ChannelExposures::default())?;

    let outcome = scan(&mut scanner, &cli);

    // Worth getting the film back even when the pass failed, so this runs either way and
    // whatever went wrong first is still what gets reported
    if cli.eject {
        match scanner.eject() {
            Ok(()) => info!("Ejected"),
            Err(e) if outcome.is_ok() => return Err(e.into()),
            Err(e) => warn!(%e, "Could not eject"),
        }
    }

    outcome
}
