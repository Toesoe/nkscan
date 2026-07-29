//! Driving an LS-5000 ED (Super Coolscan 5000 ED)
//!
//! The roll feeder senses where the frames are and reports a transport table, so frames are
//! placed from that rather than arithmetically. An adapter that reports no table falls back on
//! the frame pitch.

use crate::{Cli, FocusMode, Job, reading};
use anyhow::{Result, bail};
use nkscan::scanners::nikon::capabilities::ResolutionRange;
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
        let identity = scanner.identify()?;
        info!("Connected to {} {}", identity.vendor, identity.product);
        debug!(holder = ?scanner.holder()?, adapter = ?scanner.adapter_name(), "Adapter");
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
            resolution: dpi_5000(cli.dpi, capabilities.x_resolution)?.to_dpi(),
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
        dpi_5000(cli.dpi, self.scanner.capabilities().x_resolution)?;
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
            info!(
                "Autoexposure off, holding gain {}",
                crate::gain_spec(&self.gain, cli.ir)
            );
        }

        let capabilities = self.scanner.capabilities();

        // The feeder senses frames itself, so the table it reports is the truth about where
        // they are. Placing them by pitch is the fallback, not the default.
        let sensed = match self.scanner.roll_table() {
            Ok(table) if !table.0.is_empty() => {
                info!("Roll transport reports {} frames", table.0.len());
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
                info!("Frame {index}: autofocusing");
                self.scanner.autofocus(settings.center())?;
            }
            // A setpoint skips the per-frame autofocus pass entirely
            FocusMode::At(setpoint) => self.scanner.set_focus(setpoint)?,
        }

        // Slow, and what it measures is the film rather than the frame, so it runs once and
        // the answer carries down the roll
        if !self.fixed {
            info!("Metering");
            let metering = Metering {
                lock_white_balance: cli.lock_wb,
                ..Metering::default()
            };
            self.gain = self
                .scanner
                .autoexpose(settings.window, self.gain, metering)?;
            self.fixed = true;
            info!("Metered gain {}", crate::gain_spec(&self.gain, cli.ir));
        }

        info!(
            "Frame {index}: scanning at {} DPI, {}x sampled",
            settings.res(),
            settings.samples.count()
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

fn dpi_5000(requested: Option<u16>, offered: ResolutionRange) -> Result<Dpi> {
    let Some(requested) = requested else {
        return Ok(Dpi::_4000);
    };
    crate::resolve_dpi(requested, &Dpi::ALL, offered, Dpi::to_dpi)
}
