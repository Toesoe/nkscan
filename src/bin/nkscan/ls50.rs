//! Driving an LS-50 ED (Coolscan V ED)
//!
//! Frames are placed arithmetically from the adapter's pitch, since there is no overview pass,
//! and exposure is measured by the firmware once for the whole strip.

use crate::{Cli, FocusMode, Job, reading};
use anyhow::{Result, bail};
use nkscan::{
    decode::Image,
    scanners::{
        FilmHolder, Focus, Scanner,
        ls50::{
            Ls50,
            boundaries::FrameBoundaries,
            calibration::DEFAULT_GAIN,
            geometry::{Dpi, ScanSettings, native_dots},
        },
        nikon::ChannelExposures,
    },
    scsi::Transport,
};
use tracing::*;

pub struct Ls50Job {
    scanner: Ls50<Box<dyn Transport>>,
    frames: FrameBoundaries,
    gain: ChannelExposures,
    /// Set once the gain is settled, by `--gain` or by the one metering pass
    fixed: bool,
}

impl Ls50Job {
    pub fn open(transport: Box<dyn Transport>) -> Result<Box<dyn Job>> {
        let mut scanner = Ls50::new(transport)?;
        info!(
            identity = ?scanner.identify()?,
            holder = ?scanner.holder()?,
            adapter = ?scanner.adapter_name(),
            "Scanner open"
        );
        Ok(Box::new(Self {
            scanner,
            frames: FrameBoundaries(Vec::new()),
            gain: DEFAULT_GAIN,
            fixed: false,
        }))
    }
}

impl Job for Ls50Job {
    fn reject_unsupported(&self, cli: &Cli) -> Result<()> {
        for (given, flag) in [
            (cli.multisample != 1, "--multisample"),
            (cli.singleline, "--singleline"),
        ] {
            if given {
                bail!("{flag} is an LS-9000 option, and this is an LS-50 ED");
            }
        }
        // Metering here is firmware-side, with no knob for holding the channels together
        if cli.lock_wb {
            bail!("--lock-wb is not controllable on an LS-50 ED; use --gain to fix the ratios");
        }
        // Otherwise only discovered building the first frame's settings, after warming up
        dpi_50(cli.dpi)?;
        Ok(())
    }

    fn eject(&mut self) -> Result<()> {
        Ok(self.scanner.eject()?)
    }

    fn prepare(&mut self, cli: &Cli) -> Result<usize> {
        self.scanner.warm_up()?;

        if let Some(values) = cli.gains()? {
            if values.len() == 4 {
                bail!(
                    "this scanner drives infrared off a zeroed gain, so --gain takes three values"
                );
            }
            self.gain = ChannelExposures {
                red: values[0],
                green: values[1],
                blue: values[2],
                ir: 0,
            };
            self.fixed = true;
            info!(gain = ?self.gain, "Autoexposure off, scanning at a fixed gain");
        }

        let capabilities = self.scanner.capabilities();
        let count = match cli.frames {
            Some(n) => n as u32,
            None => self.scanner.sensed_frames().max(1),
        };
        let pitch = cli.pitch.map_or(capabilities.frame_pitch, native_dots);
        let offsets: &[f32] = if cli.offset.is_empty() {
            &[0.0]
        } else {
            &cli.offset
        };

        self.frames = FrameBoundaries::evenly_spaced(count, pitch, offsets, capabilities.max_x());
        Ok(self.frames.0.len())
    }

    fn scan_frame(&mut self, cli: &Cli, index: usize) -> Result<Image> {
        let capabilities = self.scanner.capabilities();
        let settings = ScanSettings {
            dpi: dpi_50(cli.dpi)?,
            ir: cli.ir,
            samples: 1,
            window: self.frames.0[index].scan_area(capabilities),
            capabilities,
        };

        // Re-declared every pass: one that cannot see the whole table leaves the film where it is
        self.scanner.set_frame_boundaries(&self.frames)?;

        match cli.focus {
            FocusMode::Auto => {
                info!(frame = index, "Autofocusing");
                self.scanner.autofocus(settings.center())?;
            }
            // A setpoint skips the per-frame autofocus pass entirely
            FocusMode::At(setpoint) => self.scanner.set_focus(setpoint)?,
        }

        // Firmware-side and slow, and what it measures is the film rather than the frame, so
        // it runs once and the answer carries down the strip
        if !self.fixed {
            info!("Metering");
            self.gain = self.scanner.autoexpose(&settings, self.gain)?;
            self.fixed = true;
            info!(gain = ?self.gain, "Metered");
        }

        info!(frame = index, resolution = settings.res(), "Scanning");
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

fn dpi_50(requested: Option<u16>) -> Result<Dpi> {
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
