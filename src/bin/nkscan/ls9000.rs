//! Driving an LS-9000 ED
//!
//! Frames are found by an 83-DPI overview pass unless the caller placed them, exposure is
//! metered host-side on every frame, and the holder has to be in before anything moves.

use crate::{Cli, FocusMode, Job, reading};
use anyhow::{Context, Result, bail};
use indicatif::ProgressBar;
use nkscan::{
    decode::Image,
    scanners::{
        FilmHolder, Focus, ScanArea, Scanner,
        ls9000::{
            Ls9000,
            boundaries::{FrameBoundaries, FrameRect},
            calibration::{DEFAULT_GAIN, Metering},
            geometry::{CcdMode, Dpi, Multisample, ScanSettings, native_dots},
            holder::Holder,
            window::BaseQuality,
        },
        nikon::ChannelExposures,
    },
    scsi::Transport,
};
use std::{fs::File, time::Duration};
use tracing::*;

const HOLDER_POLL: Duration = Duration::from_millis(500);

pub struct Ls9000Job {
    scanner: Ls9000<Box<dyn Transport>>,
    frames: FrameBoundaries,
    gain: ChannelExposures,
    /// Set when `--gain` fixed it, so the per-frame metering is skipped
    fixed: bool,
}

impl Ls9000Job {
    pub fn open(transport: Box<dyn Transport>) -> Result<Box<dyn Job>> {
        let mut scanner = Ls9000::new(transport)?;
        let identity = scanner.identify()?;
        info!("Connected to {} {}", identity.vendor, identity.product);
        Ok(Box::new(Self {
            scanner,
            frames: FrameBoundaries(Vec::new()),
            gain: DEFAULT_GAIN,
            fixed: false,
        }))
    }

    /// Block until a holder is loaded and the scanner has finished coming up
    ///
    /// The VPD page reports a holder the moment it is detected, but the scanner spends seconds
    /// afterwards initializing. That state is not a unit attention, so it survives the drain,
    /// and the first real command is refused several seconds later.
    fn wait_for_holder(&mut self) -> Result<()> {
        let mut holder = self.scanner.holder()?;
        if holder == Holder::None {
            let bar = ProgressBar::new_spinner().with_message("Waiting for a film holder");
            bar.enable_steady_tick(HOLDER_POLL);
            while holder == Holder::None {
                std::thread::sleep(HOLDER_POLL);
                holder = self.scanner.holder()?;
            }
            bar.finish_and_clear();
        }
        self.scanner.drain_unit_attentions()?;
        self.scanner.wait_until_ready()?;
        info!(?holder, "Holder loaded");
        Ok(())
    }

    /// Frames at a given pitch, skipping the overview entirely
    fn place_by_hand(&self, cli: &Cli) -> FrameBoundaries {
        let count = cli.frames.unwrap_or(1) as u32;
        let pitch = cli
            .pitch
            .map(native_dots)
            .unwrap_or(ScanArea::STRIP_DOTS / count.max(1));
        info!(pitch, count, "Placing frames by hand");
        FrameBoundaries(
            (0..count)
                .map(|i| {
                    let top = native_dots(cli.offset_mm(i as usize)) + i * pitch;
                    FrameRect::aligned(top, pitch)
                })
                .collect(),
        )
    }

    /// An 83-DPI pass over the whole strip, then look for the frames in it
    fn search(&mut self, cli: &Cli) -> Result<FrameBoundaries> {
        let Some(count) = cli.frames else {
            bail!("--frames says how many to look for; give --pitch to place them by hand instead")
        };
        info!("Taking an overview to find the frames");
        let bar = reading("Overview");
        let overview = self.scanner.overview_with(DEFAULT_GAIN, |read, total| {
            bar.set_length(total);
            bar.set_position(read);
        })?;
        bar.finish_and_clear();

        FrameBoundaries::detect(&overview, count).context("no frames found on the strip")
    }
}

impl Job for Ls9000Job {
    fn reject_unsupported(&self, cli: &Cli) -> Result<()> {
        // Both are otherwise only discovered building the final frame's settings, after
        // autofocus and a host-side metering pass that can run past a minute
        dpi_9000(cli.dpi)?;
        multisample(cli.multisample)?;
        Ok(())
    }

    fn prepare(&mut self, cli: &Cli) -> Result<usize> {
        self.wait_for_holder()?;

        // The session preamble, which gates the first scan
        self.scanner.calibrate(DEFAULT_GAIN)?;

        if let Some(values) = cli.gains()? {
            self.gain = ChannelExposures {
                red: values[0],
                green: values[1],
                blue: values[2],
                ir: values.get(3).copied().unwrap_or(self.gain.ir),
            };
            self.fixed = true;
            info!(gain = ?self.gain, "Autoexposure off, scanning at a fixed gain");
        }

        self.frames = if cli.placed_by_hand() {
            self.place_by_hand(cli)
        } else {
            self.search(cli)?
        };
        self.scanner.set_frame_boundaries(&self.frames)?;
        Ok(self.frames.0.len())
    }

    fn eject(&mut self) -> Result<()> {
        Ok(self.scanner.eject()?)
    }

    fn lock_gain(&mut self) {
        self.fixed = true;
    }

    fn scan_frame(&mut self, cli: &Cli, index: usize) -> Result<Image> {
        let rect = self.frames.0[index];
        match cli.focus {
            FocusMode::Auto => {
                info!(frame = index, "Autofocusing");
                self.scanner.autofocus(rect.center())?;
            }
            // A setpoint skips the per-frame autofocus pass entirely
            FocusMode::At(setpoint) => self.scanner.set_focus(setpoint)?,
        }

        let base = ScanSettings::autoexposure(rect.scan_area(), cli.ir);

        // Host-side and per frame, since it meters the frame's own content
        if !self.fixed {
            info!(frame = index, "Metering");
            let bar = reading("Metering");
            let (gain, _) = self.scanner.autoexpose_with(
                &base,
                DEFAULT_GAIN,
                &Metering {
                    lock_white_balance: cli.lock_wb,
                    ..Metering::default()
                },
                |read, total| {
                    bar.set_length(total);
                    bar.set_position(read);
                },
            )?;
            bar.finish_and_clear();
            info!(frame = index, ?gain, "Metered");
            self.gain = gain;
        }

        let settings = ScanSettings {
            dpi: dpi_9000(cli.dpi)?,
            quality: BaseQuality::Scan,
            ir: cli.ir,
            multisample: multisample(cli.multisample)?,
            ccd_mode: if cli.singleline {
                CcdMode::SingleLine
            } else {
                CcdMode::ThreeLine
            },
            ..base
        };

        info!(frame = index, "Scanning");
        let bar = reading("Scanning");
        // TEMPORARY: NKSCAN_DUMP_RAW=1 also tees the undecoded stream to disk, alongside
        // whichever <basename>_<n>.tiff this frame becomes, for debugging the three-line
        // interleave corruption at non-native DPI. Remove once that's fixed.
        let frame = if std::env::var_os("NKSCAN_DUMP_RAW").is_some() {
            let next = crate::next_index(&cli.basename);
            let path = cli.basename.with_file_name(format!(
                "{}_{next}.raw",
                cli.basename.file_name().unwrap_or_default().to_string_lossy()
            ));
            info!(path = %path.display(), "Dumping the raw scan stream");
            let mut dump = File::create(&path)
                .with_context(|| format!("creating {} for the raw dump", path.display()))?;
            self.scanner
                .scan_image_with_dump(&settings, self.gain, &mut dump, |read, total| {
                    bar.set_length(total);
                    bar.set_position(read);
                })?
        } else {
            self.scanner
                .scan_image_with(&settings, self.gain, |read, total| {
                    bar.set_length(total);
                    bar.set_position(read);
                })?
        };
        bar.finish_and_clear();
        Ok(frame)
    }
}

fn multisample(count: usize) -> Result<Multisample> {
    Ok(match count {
        1 => Multisample::X1,
        2 => Multisample::X2,
        4 => Multisample::X4,
        8 => Multisample::X8,
        16 => Multisample::X16,
        _ => bail!("--multisample must be one of 1,2,4,8,16"),
    })
}

/// The divisions of the sensor this model offers, which it has no `ALL` of its own for
const LS9000_DPI: [Dpi; 5] = [Dpi::_4000, Dpi::_2000, Dpi::_1333, Dpi::_666, Dpi::_333];

fn dpi_9000(requested: Option<u16>) -> Result<Dpi> {
    let Some(requested) = requested else {
        return Ok(Dpi::_4000);
    };
    LS9000_DPI
        .into_iter()
        .find(|mode| mode.to_dpi() == requested)
        .ok_or_else(|| {
            let legal: Vec<String> = LS9000_DPI.iter().map(|m| m.to_dpi().to_string()).collect();
            anyhow::anyhow!("--dpi expected one of {}", legal.join(", "))
        })
}
