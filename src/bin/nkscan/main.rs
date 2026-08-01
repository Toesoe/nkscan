//! Scan film on whichever Nikon Coolscan is attached
use anyhow::{Context, Result, bail};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use nkscan::{
    capability::{Capabilities, FrameLocation},
    decode::Image,
    devices::{self, Attach, DeviceInfo},
    output,
    scanners::Flow,
    session::{Exposure, FocusMode, FrameSettings, Placement, Prepare, Session},
};
use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
    time::Duration,
};
use tracing::*;

#[derive(Parser)]
#[command(version, about)]
/// Scan film on a Nikon Coolscan
pub struct Cli {
    /// Which scanner to use, as `--list` reports it. Only needed when more than one is attached.
    ///
    /// A bare device path or `usb` also works.
    #[arg(long)]
    device: Option<PathBuf>,

    /// List the scanners attached and exit, without touching any of them
    #[arg(long)]
    list: bool,

    /// Frames on the loaded strip
    ///
    /// Every one is declared to the scanner whether or not it gets scanned, since a pass that
    /// cannot see the whole table leaves the film where it is. Left out, a USB scanner is asked
    /// how many it can see; a SCSI one needs either this or --pitch.
    #[arg(long)]
    frames: Option<usize>,

    /// Which frames to actually scan, zero-indexed and comma separated. All of them by default.
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
    /// A setpoint is in the scanner's own units, not millimeters or diopters, and its range is
    /// whatever the device reports. `0` parks the motor at its home position.
    #[arg(long, default_value = "auto")]
    focus: FocusMode,

    /// Capture the infrared plane for dust removal
    #[arg(long)]
    ir: bool,

    /// Where to write, as a path prefix. Each frame becomes <basename>_<n>.tiff, and its
    /// infrared mask <basename>_<n>_ir.tiff
    ///
    /// `n` continues from whatever is already on disk under this basename, rather than
    /// restarting at 0, so scanning several strips into the same directory does not overwrite
    /// an earlier batch.
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

    /// Hold the white balance during autoexposure, so the film keeps its cast. Not on the LS-50.
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

    /// Scan every strip of the roll, not just the one that's loaded
    ///
    /// Automates the roll-analysis workflow: the first frame scanned is autoexposed as usual,
    /// then that exposure holds for every frame after it, on this strip and the ones that
    /// follow, so the whole roll comes back under one gain. Autofocus still runs per frame.
    /// Between strips it ejects the film and pauses; load the next strip and press Enter to
    /// continue, or Ctrl-C to stop when the roll is done.
    #[arg(long)]
    batch: bool,
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

    /// How many frames to look for, for a scanner that cannot count them itself
    fn frames_to_find(&self) -> Result<usize> {
        self.frames.context(
            "--frames says how many to look for; give --pitch to place them by hand instead",
        )
    }

    /// What the flags ask for before any frame is scanned
    ///
    /// Which discovery to use comes from what the scanner can do rather than from which model it
    /// is: one finds frames with an overview pass, one has a transport that senses them, and one
    /// can only be told where they are.
    fn prepare(&self, capabilities: &Capabilities) -> Result<Prepare> {
        let placement = if self.placed_by_hand() {
            Placement::Pitch {
                frames: self.frames.map(|n| n as u32),
                pitch_mm: self.pitch,
                offsets_mm: self.offset.clone(),
            }
        } else if capabilities.overview && capabilities.frames == FrameLocation::Detected {
            Placement::Detect {
                frames: self.frames_to_find()?,
            }
        } else if capabilities.frames == FrameLocation::Reported {
            Placement::Sensed {
                frames: self.frames.map(|n| n as u32),
            }
        } else {
            Placement::Pitch {
                frames: self.frames.map(|n| n as u32),
                pitch_mm: None,
                offsets_mm: Vec::new(),
            }
        };

        let exposure = match self.gains()? {
            Some(values) => Exposure::Fixed {
                visible: [values[0], values[1], values[2]],
                ir: values.get(3).copied(),
            },
            None => Exposure::Auto {
                lock_white_balance: self.lock_wb,
            },
        };

        Ok(Prepare {
            placement,
            exposure,
            wait_for_media: WAIT_FOR_MEDIA,
        })
    }

    /// What the flags ask of each frame's pass
    fn frame_settings(&self) -> FrameSettings {
        FrameSettings {
            dpi: self.dpi.unwrap_or(4000),
            ir: self.ir,
            focus: self.focus,
            multisample: self.multisample as u8,
            single_line: self.singleline,
            window: None,
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

/// How long to wait for a holder before giving up, matching how long a person takes to load one
const WAIT_FOR_MEDIA: Duration = Duration::from_secs(300);

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

/// The next unused frame number for `basename`
///
/// One past the highest `<basename>_<n>.tiff` already on disk, or 0 if there is none, so a batch
/// of scans keeps numbering across runs instead of restarting at the current frame's index.
fn next_index(basename: &Path) -> usize {
    let dir = basename.parent().filter(|p| !p.as_os_str().is_empty());
    let dir = dir.unwrap_or_else(|| Path::new("."));
    let prefix = format!(
        "{}_",
        basename.file_name().unwrap_or_default().to_string_lossy()
    );

    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name();
            let rest = name.to_string_lossy().strip_prefix(&prefix)?.to_owned();
            rest.strip_suffix(".tiff")?.parse::<usize>().ok()
        })
        .max()
        .map_or(0, |n| n + 1)
}

fn write_frame(frame: &Image, basename: &Path, index: usize) -> Result<()> {
    let path = basename.with_file_name(format!(
        "{}_{index}.tiff",
        basename.file_name().unwrap_or_default().to_string_lossy()
    ));
    output::write_rgb16_tiff(&mut BufWriter::new(File::create(&path)?), &frame.rgb)?;
    let (width, height) = frame.rgb.dimensions();
    info!("Wrote {} ({width}x{height})", path.display());

    if let Some(ir) = &frame.ir {
        let ir_path = path.with_file_name(format!(
            "{}_ir.tiff",
            path.file_stem().unwrap_or_default().to_string_lossy()
        ));
        output::write_luma16_tiff(&mut BufWriter::new(File::create(&ir_path)?), ir)?;
        info!("Wrote {} (infrared)", ir_path.display());
    }
    Ok(())
}

// ---- finding a scanner

/// Report what is attached, as `--device` would have to name it
///
/// Enumeration asks each device who it is and nothing more, so this is safe to run against a
/// scanner somebody else is using.
fn list_devices() -> Result<()> {
    let found = devices::list();
    if found.is_empty() {
        println!("No scanners found.");
        return Ok(());
    }
    for device in found {
        // Enumeration recognizes more models than this library can drive, so say which is which
        // rather than listing a scanner that will refuse to open
        let note = if device.model.is_driven() {
            String::new()
        } else {
            "  (recognized, no driver)".to_owned()
        };
        println!("{}  {}{note}", device.id, device.model.name());
    }
    Ok(())
}

/// Open the scanner the caller meant
///
/// One attached scanner needs no argument. More than one is ambiguous and says so rather than
/// picking, since which one it picked would be invisible until the wrong film came back.
fn open(cli: &Cli) -> Result<Session> {
    let mut found = devices::list();

    if let Some(wanted) = &cli.device {
        let wanted = wanted.to_string_lossy().to_string();
        found.retain(|device| names(device, &wanted));
        return match found.len() {
            1 => claim(found.remove(0)),
            _ => bail!("no scanner at {wanted}"),
        };
    }

    match found.len() {
        1 => claim(found.remove(0)),
        0 => bail!(
            "no scanner found. Point --device at one; on Windows run from an elevated prompt, \
             where a device path fails to open at all without one."
        ),
        _ => {
            let list: Vec<String> = found
                .iter()
                .map(|device| format!("\n  {} ({})", device.model.name(), device.id))
                .collect();
            bail!(
                "more than one scanner is attached, so pick one with --device:{}",
                list.concat()
            )
        }
    }
}

/// Whether `--device` picks this scanner
///
/// Its full id is the precise way. A bare path or `usb` also works, since that is what there was
/// to name a scanner by before ids existed.
fn names(device: &DeviceInfo, wanted: &str) -> bool {
    device.id.eq_ignore_ascii_case(wanted)
        || device.attach.location().eq_ignore_ascii_case(wanted)
        || (wanted.eq_ignore_ascii_case("usb") && matches!(device.attach, Attach::Usb { .. }))
}

/// Claim a scanner the search found
fn claim(device: DeviceInfo) -> Result<Session> {
    Session::open(&device.id).with_context(|| format!("opening {}", device.id))
}

// ---- the workflow

fn main() -> Result<()> {
    init_logging();
    let cli = Cli::parse();

    if cli.list {
        return list_devices();
    }

    let mut session = open(&cli)?;
    let outcome = run(&mut session, &cli);

    // Worth getting the film back even when the pass failed, so this runs either way and
    // whatever went wrong first is still what gets reported
    if cli.eject {
        match session.eject() {
            // Says which of the five things it did, since on this range of scanners "eject"
            // rewinds a cartridge on one adapter and swaps a slide on another
            Ok(action) => info!("Ejected: {action:?}"),
            Err(e) if outcome.is_ok() => return Err(e.into()),
            Err(e) => warn!("Could not eject: {e}"),
        }
    }

    outcome
}

/// Progress on stderr, so a caller can redirect it away from the images without losing it
///
/// `RUST_LOG` is taken as a request for the developer view, timestamps and targets and all.
/// Without it the levels below `info` are off and the format is trimmed to what a scan is
/// actually telling you.
fn init_logging() {
    let subscriber = tracing_subscriber::fmt().with_writer(std::io::stderr);
    match tracing_subscriber::EnvFilter::try_from_default_env() {
        Ok(filter) => subscriber.with_env_filter(filter).init(),
        Err(_) => subscriber
            .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
            .without_time()
            .with_target(false)
            .init(),
    }
}

/// The workflow, whichever scanner is on the other end, once around for `--batch`'s every
/// strip and just the once otherwise
fn run(session: &mut Session, cli: &Cli) -> Result<()> {
    let capabilities = session.capabilities()?;
    println!("Adapter: {}", capabilities.adapter_name());
    let prepare = cli.prepare(&capabilities)?;
    let settings = cli.frame_settings();
    // Before anything mechanical happens, since a scan discovers these only once it is building
    // the pass, which is after a focus and a metering run have already taken a minute
    session.check(&prepare, &settings)?;

    let mut next = next_index(&cli.basename);
    let mut strip = 1usize;
    loop {
        // Frame indices restart with every strip, so a batch run says which one it is on
        if cli.batch {
            info!("Strip {strip}");
        }

        let bar = reading("Preparing");
        let placed = session.prepare(&prepare, &mut bar_progress(&bar))?;
        bar.finish_and_clear();

        let selected = cli.selected(placed)?;
        info!("{placed} frames placed, scanning {}", selected.len());

        for index in selected {
            let bar = reading("Scanning");
            let frame = session.scan_frame(index, &settings, &mut bar_progress(&bar))?;
            bar.finish_and_clear();

            write_frame(&frame, &cli.basename, next)?;
            next += 1;
            // The first frame scanned sets the roll's exposure; everything after reuses it.
            // A no-op once it is already locked, so calling this per frame is harmless.
            if cli.batch {
                session.lock_gain();
            }
        }

        if !cli.batch {
            return Ok(());
        }
        // Film comes out under motor control, not by hand, so it has to be ejected before the
        // next strip can go in
        session.eject()?;
        pause_for_reload()?;
        strip += 1;
    }
}

/// Drive `bar` from a pass, without ever asking it to stop
fn bar_progress(bar: &ProgressBar) -> impl FnMut(u64, u64) -> Flow + '_ {
    move |read, total| {
        bar.set_length(total);
        bar.set_position(read);
        Flow::Continue
    }
}

/// Block for the caller to load the next strip before `--batch` moves on to it
fn pause_for_reload() -> Result<()> {
    use std::io::Write;
    eprint!(
        "Strip finished and ejected. Load the next one and press Enter to continue, or Ctrl-C to stop: "
    );
    std::io::stderr().flush()?;
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}
