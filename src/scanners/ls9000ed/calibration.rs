//! The pre-scan calibration pass
//!
//! A session-level preamble, not part of any scan. Nikon Scan runs it once, gating the first SCAN.

use super::{
    Channel, Ls9000ed, ScanArea,
    boundaries::FrameBoundaries,
    dtc::{self, Dtc},
    geometry::{CcdMode, Multisample},
    window::{BaseQuality, WindowKind, WindowParams},
};
use crate::{
    scanners::Focus,
    scsi::{self, Transport},
};
use image::{ImageBuffer, Rgb};
use std::ops::Deref;
use tracing::*;

/// Per-channel analog gain, as the window descriptor tail carries it
///
/// Persists in the scanner across sessions, so [`Ls9000ed::channel_exposures`] returns
/// whatever was last written rather than a power-on default. Metering that starts from it
/// compounds run over run; start from a fixed value instead.
#[derive(Debug, Clone, Copy)]
pub struct ChannelExposures {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub ir: u32,
}

impl Default for ChannelExposures {
    /// One capture's values, for a channel the scanner does not report. Not a good starting
    /// point for anything: gain depends on the film.
    fn default() -> Self {
        Self {
            red: 0x0001_14D9,
            green: 0x0000_C4F3,
            blue: 0x0000_A1CD,
            ir: 0x0001_6151,
        }
    }
}

impl ChannelExposures {
    /// The gain staged for one channel. `Channel::All` has none of its own, so it reports the
    /// red one the scanner leads with.
    pub fn get(&self, channel: Channel) -> u32 {
        match channel {
            Channel::Red | Channel::All => self.red,
            Channel::Green => self.green,
            Channel::Blue => self.blue,
            Channel::Ir => self.ir,
        }
    }

    fn set(&mut self, channel: Channel, exposure: u32) {
        match channel {
            Channel::Red | Channel::All => self.red = exposure,
            Channel::Green => self.green = exposure,
            Channel::Blue => self.blue = exposure,
            Channel::Ir => self.ir = exposure,
        }
    }
}

/// The full-area window staged on every channel before calibrating, identical in all captures regardless of film format
fn calibration_window() -> ScanArea {
    ScanArea {
        x_pos: 0,
        y_pos: 0,
        x_size: ScanArea::FILM_WIDTH_DOTS,
        y_size: 13176,
    }
}

impl<T> Ls9000ed<T>
where
    T: Transport,
{
    /// The gain the scanner currently has staged, read back off its own window descriptors
    ///
    /// Any channel the scanner does not report keeps its [`Default`] value
    pub fn channel_exposures(&mut self) -> Result<ChannelExposures, scsi::Error> {
        let mut exposures = ChannelExposures::default();
        for descriptor in self.get_window(None)? {
            let Some(tail) = descriptor.vendor.get(6..10) else {
                continue;
            };
            let exposure = u32::from_be_bytes(tail.try_into().expect("6..10 is four bytes"));
            // ScanArea 0 is the composite and carries no exposure of its own
            if let Some(channel) = Channel::from_id(descriptor.id).filter(|c| *c != Channel::All) {
                exposures.set(channel, exposure);
            }
        }
        Ok(exposures)
    }

    /// Run the pre-scan calibration, after the holder is loaded and before the first [`scan`](Self::scan)
    ///
    /// Writes the nominal frame table, because nothing is known about the real frames until an
    /// overview pass has found them. Follow that with
    /// [`set_frame_boundaries`](Self::set_frame_boundaries), as Nikon Scan does.
    pub fn calibrate(&mut self, exposures: ChannelExposures) -> Result<(), scsi::Error> {
        debug!("Calibrating");

        // Nikon Scan reads this per channel immediately before staging the windows. We don't
        // use the payload, but the read may be what latches the frame setup firmware-side.
        for channel in Channel::RGBI {
            let setup = self.read_framed_dtc(Dtc::FrameSetup, Some(channel), dtc::HEADER_LEN)?;
            trace!(?channel, len = setup.len(), "Frame setup");
        }

        // Stage a full-area window on every channel, IR included
        for channel in Channel::RGBI {
            let params = WindowParams {
                ccd: CcdMode::SingleLine,
                multisample: Multisample::X1,
                quality: BaseQuality::Scan,
                window_kind: WindowKind::Frame,
                exposure: exposures.get(channel),
            };
            self.set_window(channel, params.descriptor(4000, calibration_window()))?;
        }

        // Commit the staged focus by writing back whatever the scanner reports
        let focus = self.focus()?;
        self.set_focus(focus)?;

        // The scanner wants its current frame table read before it accepts a new one
        let current = self.frame_boundaries()?;
        trace!(?current, "Frame boundaries before calibration");
        self.set_frame_boundaries(&FrameBoundaries::nominal())?;

        // Nothing consumes these yet, but they're part of the observed sequence
        for channel in Channel::RGB {
            let channel = Some(channel);
            let dark = self.read_dtc(Dtc::DarkCurrent, channel, dtc::HEADER_LEN + 4)?;
            let line = self.read_framed_dtc(Dtc::ExtendedLine, channel, 20)?;
            trace!(?channel, ?dark, ?line, "Channel calibration readback");
        }

        Ok(())
    }
}

/// The `percentile` brightest sample of one channel
fn tail<C>(image: &ImageBuffer<Rgb<u16>, C>, channel: usize, percentile: f32) -> Option<u16>
where
    C: Deref<Target = [u16]>,
{
    let mut samples: Vec<u16> = image.pixels().map(|p| p.0[channel]).collect();
    samples.sort_unstable();
    let at = (samples.len().saturating_sub(1) as f32 * percentile.clamp(0.0, 1.0)) as usize;
    samples.get(at).copied()
}

/// Gain that would put `level` at `target`, or `None` if it cannot be known from this level
///
/// A channel pinned to 65535 says nothing about how far over it is, so it is halved and wants
/// another pass. A channel reading zero has nothing to scale against.
fn step(level: u16, target: u16) -> Option<f64> {
    match level {
        u16::MAX => Some(0.5),
        0 => None,
        _ => Some(f64::from(target) / f64::from(level)),
    }
}

/// Gain that puts the `percentile` brightest sample of each channel at `target`
///
/// Gain is linear in the value, so one proportional step lands it. Nikon Scan's Analog Gain
/// palette is the same knob in EV, where 1 EV is a factor of two.
///
/// Each channel is scaled on its own, which equalises their full-scale points: on a negative
/// that neutralises the orange mask, since the brightest thing there is the film base. Use
/// [`meter_locked`] to keep the channels' relative gains instead.
///
/// Visible channels only, IR is left alone.
pub fn meter<C>(
    image: &ImageBuffer<Rgb<u16>, C>,
    current: ChannelExposures,
    percentile: f32,
    target: u16,
) -> ChannelExposures
where
    C: Deref<Target = [u16]>,
{
    let mut metered = current;
    for (index, channel) in Channel::RGB.into_iter().enumerate() {
        let Some(scale) = tail(image, index, percentile).and_then(|level| step(level, target))
        else {
            continue;
        };
        let scaled = (f64::from(current.get(channel)) * scale).round();
        metered.set(channel, scaled.clamp(1.0, f64::from(u32::MAX)) as u32);
    }
    metered
}

/// Like [`meter`], but one factor for all three channels, so their relative gains survive
///
/// The factor is the smallest any channel asks for, so the most constrained one lands on
/// `target` and none go past it. That leaves whatever cast the film has, which is what you
/// want for slides, or for negatives you intend to invert and balance yourself.
///
/// Visible channels only, IR is left alone.
pub fn meter_locked<C>(
    image: &ImageBuffer<Rgb<u16>, C>,
    current: ChannelExposures,
    percentile: f32,
    target: u16,
) -> ChannelExposures
where
    C: Deref<Target = [u16]>,
{
    let factor = Channel::RGB
        .into_iter()
        .enumerate()
        .filter_map(|(index, _)| tail(image, index, percentile).and_then(|l| step(l, target)))
        .fold(f64::INFINITY, f64::min);
    if !factor.is_finite() {
        return current;
    }

    let mut metered = current;
    for channel in Channel::RGB {
        let scaled = (f64::from(current.get(channel)) * factor).round();
        metered.set(channel, scaled.clamp(1.0, f64::from(u32::MAX)) as u32);
    }
    metered
}

#[cfg(test)]
mod meter_tests {
    use super::*;
    use image::Rgb;

    fn flat(r: u16, g: u16, b: u16) -> ImageBuffer<Rgb<u16>, Vec<u16>> {
        ImageBuffer::from_pixel(8, 8, Rgb([r, g, b]))
    }

    fn exposures(v: u32) -> ChannelExposures {
        ChannelExposures {
            red: v,
            green: v,
            blue: v,
            ir: v,
        }
    }

    #[test]
    fn scales_each_channel_toward_the_target() {
        let metered = meter(&flat(8000, 16000, 32000), exposures(1000), 1.0, 32000);
        assert_eq!(
            (metered.red, metered.green, metered.blue),
            (4000, 2000, 1000)
        );
    }

    #[test]
    fn infrared_is_untouched() {
        let metered = meter(&flat(8000, 8000, 8000), exposures(1234), 1.0, 32000);
        assert_eq!(metered.ir, 1234);
    }

    /// A pinned channel says nothing about how far over it is, so back off and re-meter
    #[test]
    fn a_clipped_channel_is_halved() {
        let metered = meter(&flat(u16::MAX, 8000, 8000), exposures(1000), 1.0, 60000);
        assert_eq!(metered.red, 500);
    }

    /// One blown pixel must not cost the whole channel
    #[test]
    fn outliers_do_not_set_the_exposure() {
        let mut image = flat(8000, 8000, 8000);
        image.put_pixel(0, 0, Rgb([u16::MAX; 3]));
        let metered = meter(&image, exposures(1000), 0.95, 32000);
        assert_eq!(metered.red, 4000);
    }

    #[test]
    fn a_dark_channel_is_left_alone() {
        let metered = meter(&flat(0, 0, 0), exposures(777), 1.0, 60000);
        assert_eq!((metered.red, metered.green, metered.blue), (777, 777, 777));
    }

    /// The most constrained channel lands on target and the others stay under it
    #[test]
    fn locked_scales_every_channel_by_the_same_factor() {
        let metered = meter_locked(&flat(8000, 8000, 16000), exposures(1000), 1.0, 32000);
        assert_eq!(
            (metered.red, metered.green, metered.blue),
            (2000, 2000, 2000)
        );
    }

    /// Which is the whole point: the gains keep their ratios
    #[test]
    fn locked_preserves_the_ratios_between_channels() {
        let current = ChannelExposures {
            red: 71_890,
            green: 50_732,
            blue: 41_419,
            ir: 93_634,
        };
        let metered = meter_locked(&flat(4000, 8000, 16000), current, 1.0, 32000);

        let before = f64::from(current.red) / f64::from(current.blue);
        let after = f64::from(metered.red) / f64::from(metered.blue);
        assert!((before - after).abs() < 1e-4, "{before} became {after}");

        // Blue asked for the smallest step, so it is the one that lands on target
        assert_eq!(metered.blue, current.blue * 2);
    }

    /// A clipped channel drags everything down together, ratios intact
    #[test]
    fn locked_halves_when_a_channel_is_pinned() {
        let metered = meter_locked(&flat(u16::MAX, 8000, 8000), exposures(1000), 1.0, 60000);
        assert_eq!((metered.red, metered.green, metered.blue), (500, 500, 500));
    }

    #[test]
    fn locked_leaves_a_dark_frame_alone() {
        let metered = meter_locked(&flat(0, 0, 0), exposures(777), 1.0, 60000);
        assert_eq!((metered.red, metered.green, metered.blue), (777, 777, 777));
    }
}

/// How to meter a frame
#[derive(Debug, Clone, Copy)]
pub struct Metering {
    /// Where to put each channel's high tail. The ADC saturates at 65535, and the second pass
    /// can overshoot a small correction by a few percent, so leave headroom.
    pub target: u16,
    /// Which sample counts as the high tail, so a few blown pixels do not set the gain
    pub percentile: f32,
    /// One pass lands 3-10 percent under, a second measures from where it got to
    pub passes: usize,
    /// Scale the channels together, keeping their relative gains and whatever cast the film
    /// has. Off means each fills the range on its own, which neutralises a negative's mask.
    pub lock_white_balance: bool,
}

impl Default for Metering {
    fn default() -> Self {
        Self {
            target: 58_000,
            percentile: 0.999,
            passes: 2,
            lock_white_balance: false,
        }
    }
}
