//! Scan film on whichever Nikon Coolscan is attached.
//!
//! The two scanners this drives share a workflow: open, wake the mechanism, work out where the
//! frames are, then focus, expose, scan and write each one. They differ in how some of those
//! steps happen, which is what the [`Job`] trait is for, and in a handful of knobs only one of
//! them has, which are refused rather than ignored.

use anyhow::{Context, Result, bail};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use ls50::Ls50Job;
use ls9000::Ls9000Job;
use nkscan::{
    decode::Image,
    output,
    scanners::ls50ed,
    scsi::{Transport, TransportExt, usb::UsbTransport},
};
use nusb::MaybeFuture;
use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};
use tracing::*;

/// The SCSI transport for this platform
///
/// macOS has an unimplemented stub whose signature differs, so SCSI is not offered there. USB
/// is, so a mac can still drive an LS-50.
#[cfg(target_os = "linux")]
use nkscan::scsi::linux::SgDevice as ScsiDevice;
#[cfg(target_os = "windows")]
use nkscan::scsi::windows::ScsiScanDevice as ScsiDevice;

/// Only the platforms with a SCSI transport ask a device who it is
#[cfg(any(target_os = "linux", target_os = "windows"))]
use nkscan::scsi::cdbs::Inquiry;

#[derive(Parser)]
#[command(version, about)]
/// Scan film on a Nikon Coolscan
pub struct Cli {
    /// Device path, skipping the search. `/dev/sg*` on Linux, `\\.\Scanner0` on Windows.
    ///
    /// Not needed for a USB scanner, which is found by its ids.
    #[arg(long)]
    device: Option<PathBuf>,

    /// Frames on the loaded strip
    ///
    /// Every one is declared to the scanner whether or not it gets scanned, since a pass that
    /// cannot see the whole table leaves the film where it is. Omitted, a USB scanner is asked
    /// how many it can see and a SCSI one assumes one frame.
    #[arg(long)]
    frames: Option<usize>,

    /// Which of those to actually scan, zero-indexed, comma separated. All by default.
    #[arg(long, value_delimiter = ',')]
    frame: Vec<usize>,

    /// Resolution in DPI. One of the firmware's divisions of the sensor's native pitch.
    #[arg(long)]
    dpi: Option<u16>,

    /// Fixed per-channel analog gain as `red,green,blue[,ir]`, which turns autoexposure off
    ///
    /// For scanning a whole roll under one exposure, so frames from different strips stay
    /// comparable. Autoexpose one frame first and use the gains it logs.
    #[arg(long, value_name = "R,G,B[,IR]")]
    gain: Option<String>,

    /// Focus: `auto` to let the scanner find it on each frame, or a fixed setpoint
    ///
    /// A setpoint is in the scanner's own units, not millimetres or dioptres, and its range is
    /// whatever the device reports. `0` parks the motor at its home position.
    #[arg(long, default_value = "auto")]
    focus: FocusMode,

    /// Capture the infrared plane for dust removal
    #[arg(long)]
    ir: bool,

    /// Where to write, as a path prefix. Each frame becomes <basename>_<n>.tiff, and its
    /// infrared mask <basename>_<n>_ir.tiff
    #[arg(long, default_value = "scan")]
    basename: PathBuf,

    /// Where each frame starts along the feed, in mm, comma separated, last value repeating
    ///
    /// Giving this or --pitch places the frames arithmetically rather than looking for them,
    /// which is the way round a strip the search misreads.
    #[arg(long, value_delimiter = ',')]
    offset: Vec<f32>,

    /// Frame pitch in mm, overriding what the scanner would use
    #[arg(long)]
    pitch: Option<f32>,

    /// Hold the white balance during autoexposure, so the film keeps its cast. LS-9000 only.
    #[arg(long)]
    lock_wb: bool,

    /// Multisampling, trading scan time for noise. One of 1,2,4,8,16. LS-9000 only.
    #[arg(long, default_value_t = 1)]
    multisample: usize,

    /// Single-line CCD mode. Slow, but may improve banding. LS-9000 only.
    #[arg(long)]
    singleline: bool,

    /// Send the film back out when everything is done
    #[arg(long)]
    eject: bool,
}

/// Where to put the focus motor before a pass
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusMode {
    /// Let the scanner find focus, once per frame
    Auto,
    /// Drive the motor to this setpoint and leave it there
    At(u16),
}

impl std::str::FromStr for FocusMode {
    type Err = String;

    fn from_str(text: &str) -> std::result::Result<Self, String> {
        match text {
            "auto" => Ok(FocusMode::Auto),
            other => other
                .parse()
                .map(FocusMode::At)
                .map_err(|_| format!("expected `auto` or a setpoint, got `{other}`")),
        }
    }
}

impl Cli {
    /// Frames the caller asked for, defaulting to all of them
    fn selected(&self, placed: usize) -> Result<Vec<usize>> {
        let selected: Vec<usize> = if self.frame.is_empty() {
            (0..placed).collect()
        } else {
            self.frame.clone()
        };
        if let Some(&past) = selected.iter().find(|&&i| i >= placed) {
            bail!("frame {past} is not on a {placed}-frame strip");
        }
        Ok(selected)
    }

    /// Whether the caller placed the frames rather than asking for a search
    fn placed_by_hand(&self) -> bool {
        self.pitch.is_some() || !self.offset.is_empty()
    }

    /// Where frame `index` starts, in mm, with the last given offset repeating
    fn offset_mm(&self, index: usize) -> f32 {
        match self.offset.len() {
            0 => 0.0,
            n => self.offset[index.min(n - 1)],
        }
    }

    /// The three visible gains and an optional infrared one
    fn gains(&self) -> Result<Option<Vec<u32>>> {
        let Some(spec) = &self.gain else {
            return Ok(None);
        };
        let values: Vec<u32> = spec
            .split(',')
            .map(|field| field.trim().parse::<u32>())
            .collect::<std::result::Result<_, _>>()
            .with_context(|| format!("{spec} is not a comma-separated list of numbers"))?;
        if !matches!(values.len(), 3 | 4) {
            bail!("--gain takes three or four values, got {}", values.len());
        }
        Ok(Some(values))
    }
}

/// A bar that fills as a pass is read off the scanner
pub fn reading(what: &str) -> ProgressBar {
    let bar = ProgressBar::no_length().with_message(what.to_owned());
    bar.set_style(
        ProgressStyle::with_template("{msg:>12} [{bar:40}] {bytes}/{total_bytes} {eta}")
            .expect("template is valid")
            .progress_chars("=> "),
    );
    bar
}

fn write_frame(frame: &Image, basename: &Path, index: usize) -> Result<()> {
    let path = basename.with_file_name(format!(
        "{}_{index}.tiff",
        basename.file_name().unwrap_or_default().to_string_lossy()
    ));
    output::write_rgb16_tiff(&mut BufWriter::new(File::create(&path)?), &frame.rgb)?;
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

// ---- finding a scanner

/// Wraps an opened transport in the driver for one model
type Opener = fn(Box<dyn Transport>) -> Result<Box<dyn Job>>;

/// A scanner this CLI knows how to drive
///
/// Adding a model is an entry here plus a [`Job`] implementation. Everything else, discovery
/// and the workflow both, is written against these rather than against any one scanner.
struct Model {
    /// Matched against the INQUIRY product string, or shown when a USB scanner is found
    name: &'static str,
    attach: Attach,
    open: Opener,
}

/// How a model turns up
enum Attach {
    /// Enumerated by its USB ids
    Usb { vendor: u16, product: u16 },
    /// Found by sweeping device paths and asking each who it is
    Scsi,
}

const MODELS: &[Model] = &[
    Model {
        name: "LS-9000 ED",
        attach: Attach::Scsi,
        open: Ls9000Job::open,
    },
    Model {
        name: "LS-50 ED",
        attach: Attach::Usb {
            vendor: ls50ed::VENDOR_ID,
            product: ls50ed::PRODUCT_ID,
        },
        open: Ls50Job::open,
    },
];

/// A scanner the search turned up, not yet opened
struct Found {
    model: &'static Model,
    /// `None` for a USB scanner, which has no path to name
    path: Option<PathBuf>,
}

impl Found {
    fn open(self) -> Result<Box<dyn Job>> {
        let transport: Box<dyn Transport> = match (&self.model.attach, &self.path) {
            (Attach::Usb { vendor, product }, _) => {
                Box::new(UsbTransport::open(*vendor, *product).context("opening the USB scanner")?)
            }
            (Attach::Scsi, Some(path)) => open_scsi(path)?,
            (Attach::Scsi, None) => bail!("a SCSI scanner needs a device path"),
        };
        (self.model.open)(transport)
    }

    /// What `--device` would have to say to pick this one
    fn selector(&self) -> String {
        match &self.path {
            Some(path) => path.display().to_string(),
            None => "usb".to_owned(),
        }
    }
}

/// Open the scanner the caller meant
///
/// One attached scanner needs no argument. More than one is ambiguous and says so rather than
/// picking, since which one it picked would be invisible until the wrong film came back.
fn open(cli: &Cli) -> Result<Box<dyn Job>> {
    let mut found = discover();

    if let Some(device) = &cli.device {
        let wants_usb = device.as_os_str().eq_ignore_ascii_case("usb");
        found.retain(|f| match &f.path {
            Some(path) => path == device,
            None => wants_usb,
        });
        return match found.len() {
            1 => found.remove(0).open(),
            _ => bail!("no scanner at {}", device.display()),
        };
    }

    match found.len() {
        1 => found.remove(0).open(),
        0 => bail!(
            "no scanner found. Point --device at one; on Windows run from an elevated prompt, \
             where a device path fails to open at all without one."
        ),
        _ => {
            let list: Vec<String> = found
                .iter()
                .map(|f| format!("\n  {} ({})", f.model.name, f.selector()))
                .collect();
            bail!(
                "more than one scanner is attached, so pick one with --device:{}",
                list.concat()
            )
        }
    }
}

/// Every scanner attached that this CLI knows how to drive
fn discover() -> Vec<Found> {
    let mut found = Vec::new();

    // Presence only, since claiming the interface is what opening would do
    let usb: Vec<(u16, u16)> = nusb::list_devices()
        .wait()
        .map(|devices| devices.map(|d| (d.vendor_id(), d.product_id())).collect())
        .unwrap_or_default();

    for model in MODELS {
        match model.attach {
            Attach::Usb { vendor, product } => {
                found.extend(
                    usb.iter()
                        .filter(|ids| **ids == (vendor, product))
                        .map(|_| Found { model, path: None }),
                );
            }
            // Swept once below, since one INQUIRY answers for every SCSI model at once
            Attach::Scsi => {}
        }
    }

    for path in scsi_paths() {
        let Some(product) = probe_scsi(&path) else {
            continue;
        };
        if let Some(model) = MODELS.iter().find(|m| {
            matches!(m.attach, Attach::Scsi)
                && product
                    .to_ascii_lowercase()
                    .contains(&m.name.to_ascii_lowercase())
        }) {
            found.push(Found {
                model,
                path: Some(path),
            });
        } else {
            debug!(%product, path = %path.display(), "A Nikon we do not drive");
        }
    }
    found
}

/// Device paths worth asking who they are
#[cfg(target_os = "linux")]
fn scsi_paths() -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir("/dev") else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let name = path.file_name()?.to_str()?.to_owned();
            let index = name.strip_prefix("sg")?;
            (!index.is_empty() && index.chars().all(|c| c.is_ascii_digit())).then_some(path)
        })
        .collect();
    paths.sort();
    paths
}

#[cfg(target_os = "windows")]
fn scsi_paths() -> Vec<PathBuf> {
    (0..10)
        .map(|n| PathBuf::from(format!(r"\\.\Scanner{n}")))
        .collect()
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn scsi_paths() -> Vec<PathBuf> {
    Vec::new()
}

/// How the device at `path` introduces itself, if it is a Nikon at all
///
/// An INQUIRY and nothing else. This sweeps devices that have nothing to do with us, so it must
/// not change any of them: notably it does not build a driver, which would reserve the unit and
/// write a mode page on its way up. Anything that fails to open or answers as something else is
/// not a match rather than an error, since being refused by an unrelated device is normal.
#[cfg(any(target_os = "linux", target_os = "windows"))]
fn probe_scsi(path: &Path) -> Option<String> {
    let mut device = ScsiDevice::open(path).ok()?;
    let identity = device.send(&Inquiry::new()).ok()?;
    identity
        .vendor
        .trim()
        .eq_ignore_ascii_case("nikon")
        .then(|| identity.product.trim().to_owned())
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn probe_scsi(_path: &Path) -> Option<String> {
    None
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn open_scsi(path: &Path) -> Result<Box<dyn Transport>> {
    let device = ScsiDevice::open(path).with_context(|| format!("opening {}", path.display()))?;
    Ok(Box::new(device))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn open_scsi(_path: &Path) -> Result<Box<dyn Transport>> {
    bail!("SCSI is not implemented on this platform, so only a USB scanner will work here")
}

// ---- the workflow

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let cli = Cli::parse();

    let mut job = open(&cli)?;
    let outcome = run(job.as_mut(), &cli);

    // Worth getting the film back even when the pass failed, so this runs either way and
    // whatever went wrong first is still what gets reported
    if cli.eject {
        match job.eject() {
            Ok(()) => info!("Ejected"),
            Err(e) if outcome.is_ok() => return Err(e),
            Err(e) => warn!(%e, "Could not eject"),
        }
    }

    outcome
}

/// The workflow, once, whichever scanner is on the other end
fn run(job: &mut dyn Job, cli: &Cli) -> Result<()> {
    job.reject_unsupported(cli)?;
    let placed = job.prepare(cli)?;
    info!(frames = placed, "Frames placed");

    for index in cli.selected(placed)? {
        let frame = job.scan_frame(cli, index)?;
        write_frame(&frame, &cli.basename, index)?;
    }
    Ok(())
}

/// A scanner part way through a session
///
/// The frame table, gain type and scan settings differ per model and none of them appear here:
/// each implementation keeps its own, and this only names the steps.
pub trait Job {
    /// Wake the mechanism and work out where the frames are, returning how many were placed
    fn prepare(&mut self, cli: &Cli) -> Result<usize>;

    /// Focus, expose and scan one frame
    fn scan_frame(&mut self, cli: &Cli, index: usize) -> Result<Image>;

    fn eject(&mut self) -> Result<()>;

    /// Refuse a knob this scanner does not have, rather than quietly doing something else
    fn reject_unsupported(&self, _cli: &Cli) -> Result<()> {
        Ok(())
    }
}

mod ls50;
mod ls9000;
