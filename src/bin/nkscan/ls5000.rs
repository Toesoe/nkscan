//! Driving an LS-5000 ED (Super Coolscan 5000 ED)
//!
//! The roll feeder senses where the frames are and reports a transport table, so frames are
//! placed from that rather than arithmetically. An adapter that reports no table falls back on
//! the frame pitch.

use crate::{Cli, FocusMode, Job, reading};
use anyhow::{Result, bail};
use nkscan::{
    decode::Image,
    scanners::{
        FilmHolder, Focus, Scanner,
        ls5000::{
            Ls5000,
            boundaries::FrameBoundaries,
            calibration,
            geometry::{Dpi, Samples, ScanSettings, native_dots},
        },
        nikon::{ChannelExposures, metering::Metering},
    },
    scsi::Transport,
};
use tracing::*;

pub struct Ls5000Job {
    scanner: Ls5000<Box<dyn Transport>>,
    frames: FrameBoundaries,
    gain: ChannelExposures,
    /// Set once the gain is settled, by `--gain` or by the one metering pass
    fixed: bool,
}

impl Ls5000Job {
    pub fn open(transport: Box<dyn Transport>) -> Result<Box<dyn Job>> {
        let mut scanner = Ls5000::new(transport)?;
        info!(
            identity = ?scanner.identify()?,
            holder = ?scanner.holder()?,
            adapter = ?scanner.adapter_name(),
            "Scanner open"
        );
        Ok(Box::new(Self {
            scanner,
            frames: FrameBoundaries(Vec::new()),
            gain: calibration::DEFAULT_GAIN,
            fixed: false,
        }))
    }

    fn settings(&self, cli: &Cli, index: usize) -> Result<ScanSettings> {
        let capabilities = self.scanner.capabilities();
        Ok(ScanSettings {
            resolution: dpi_5000(cli.dpi)?.to_dpi(),
            ir: cli.ir,
            samples: Samples::new(cli.multisample as u8).ok_or_else(|| {
                let legal: Vec<String> = Samples::ALL.iter().map(u8::to_string).collect();
                anyhow::anyhow!("--multisample expected one of {}", legal.join(", "))
            })?,
            window: self.frames.0[index].scan_area(capabilities),
            capabilities,
        })
    }
}

impl Job for Ls5000Job {
    fn reject_unsupported(&self, cli: &Cli) -> Result<()> {
        if cli.singleline {
            bail!("--singleline is an LS-9000 option, and this is an LS-5000 ED");
        }
        // Armed correctly but never decoded: a multi-sampled pass streams every sample rather
        // than one averaged image, and that readout is not implemented
        if cli.multisample != 1 {
            bail!(
                "--multisample is not implemented on an LS-5000 ED: the scanner streams every \
                 sample rather than averaging them, and that readout has not been written"
            );
        }
        // Otherwise only discovered building the first frame's settings, after warming up
        dpi_5000(cli.dpi)?;
        Ok(())
    }

    fn eject(&mut self) -> Result<()> {
        Ok(self.scanner.eject()?)
    }

    fn prepare(&mut self, cli: &Cli) -> Result<usize> {
        self.scanner.warm_up()?;

        if let Some(values) = cli.gains()? {
            self.gain = ChannelExposures {
                red: values[0],
                green: values[1],
                blue: values[2],
                // This model meters infrared rather than driving it off a zeroed field, so an
                // omitted fourth value keeps the default instead of blacking the plane out
                ir: values
                    .get(3)
                    .copied()
                    .unwrap_or(calibration::DEFAULT_GAIN.ir),
            };
            self.fixed = true;
            info!(gain = ?self.gain, "Autoexposure off, scanning at a fixed gain");
        }

        let capabilities = self.scanner.capabilities();

        // The feeder senses frames itself, so the table it reports is the truth about where
        // they are. Placing them by pitch is the fallback, not the default.
        let sensed = match self.scanner.roll_table() {
            Ok(table) if !table.0.is_empty() => {
                info!(frames = table.0.len(), "Read the roll transport table");
                Some(table)
            }
            Ok(_) => None,
            Err(e) => {
                debug!(%e, "No roll transport table, placing frames on the pitch");
                None
            }
        };

        self.frames = match sensed {
            Some(table) if !cli.placed_by_hand() => table,
            _ => {
                let count = match cli.frames {
                    Some(n) => n as u32,
                    None => self.scanner.sensed_frames().max(1),
                };
                let pitch = cli.pitch.map_or(capabilities.frame_pitch, native_dots);
                FrameBoundaries::evenly_spaced(count, pitch)
            }
        };

        if let Some(n) = cli.frames {
            self.frames.0.truncate(n);
        }
        Ok(self.frames.0.len())
    }

    fn scan_frame(&mut self, cli: &Cli, index: usize) -> Result<Image> {
        let settings = self.settings(cli, index)?;

        match cli.focus {
            FocusMode::Auto => {
                info!(frame = index, "Autofocusing");
                self.scanner.autofocus(settings.center())?;
            }
            // A setpoint skips the per-frame autofocus pass entirely
            FocusMode::At(setpoint) => self.scanner.set_focus(setpoint)?,
        }

        // Slow, and what it measures is the film rather than the frame, so it runs once and
        // the answer carries down the roll
        if !self.fixed {
            info!(lock_wb = cli.lock_wb, "Metering");
            let metering = Metering {
                lock_white_balance: cli.lock_wb,
                ..Metering::default()
            };
            self.gain = self
                .scanner
                .autoexpose(settings.window, self.gain, metering)?;
            self.fixed = true;
            info!(gain = ?self.gain, "Metered");
        }

        info!(
            frame = index,
            resolution = settings.res(),
            samples = settings.samples.count(),
            "Scanning"
        );
        let bar = reading("Scanning");
        let frame = self
            .scanner
            .scan_image_with(&settings, self.gain, |read, total| {
                bar.set_length(total);
                bar.set_position(read);
            })?;
        bar.finish_and_clear();
        Ok(frame)
    }
}

fn dpi_5000(requested: Option<u16>) -> Result<Dpi> {
    let Some(requested) = requested else {
        return Ok(Dpi::_4000);
    };
    Dpi::ALL
        .into_iter()
        .find(|mode| mode.to_dpi() == requested)
        .ok_or_else(|| {
            let legal: Vec<String> = Dpi::ALL.iter().map(|m| m.to_dpi().to_string()).collect();
            anyhow::anyhow!("--dpi expected one of {}", legal.join(", "))
        })
}
