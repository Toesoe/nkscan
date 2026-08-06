//! Metering we do ourselves, for units with no hardware AE
//!
//! [`Exposure`](super::Exposure) decides when this is needed. We take an
//! ordinary low-resolution pass and work out the per-channel exposures from it.
//! Nikon Scan does the same: nothing in the capture corpus uses a setup scan
//! kind or reads `81h`.
//!
//! The sensor is linear in integration time, so one proportional step gets us
//! there. The exception is a clipped pass. A channel sitting at full scale
//! could be anywhere above it, so we halve it and measure again.

use crate::{
    error::Error,
    protocol::{
        caps::{Capabilities, set_window::ColorInterleaving},
        image::Layout,
        window::{IR, Window},
    },
};

/// How to meter a frame
#[derive(Debug, Clone, Copy)]
pub struct Metering {
    /// Where to put each channel's high tail, as a fraction of full scale.
    /// Under 1.0 to leave room for a correction that overshoots
    pub target: f32,
    /// Which sample counts as the high tail. Keeps a dust speck or a few blown
    /// pixels from setting the exposure
    pub percentile: f32,
    /// Move the visible channels by one factor so they keep their proportions,
    /// and the film keeps its cast. Off means each one fills the range by
    /// itself, which takes the orange mask off a negative
    pub lock_white_balance: bool,
}

impl Default for Metering {
    fn default() -> Self {
        Self {
            target: 0.97,
            percentile: 0.999,
            lock_white_balance: false,
        }
    }
}

impl Metering {
    /// New exposures for `windows`, from a pass taken with the old ones
    ///
    /// `raw` is what that pass produced and `layout` describes it. The result
    /// lines up with `windows`. A channel the pass tells us nothing about keeps
    /// the exposure it had.
    pub fn apply(
        &self,
        caps: &Capabilities,
        layout: &Layout,
        raw: &[u8],
        windows: &[Window],
    ) -> Result<Vec<u32>, Error> {
        if layout.channels.len() != windows.len() {
            return Err(Error::Unsupported {
                op: "metering",
                reason: format!(
                    "the pass carried {} channels and there are {} windows",
                    layout.channels.len(),
                    windows.len()
                ),
            });
        }

        // D1h bytes 16-24. Anything outside it comes back as common error 2
        let limit = &caps.set_window.exposure;
        let ceiling = ceiling(layout.bits_per_sample);
        let target = (f32::from(ceiling) * self.target.clamp(0.0, 1.0)) as u16;

        // What each channel asks to be scaled by, before the lock has a say
        let steps: Vec<Option<f64>> = self
            .measure(layout, raw)?
            .into_iter()
            .map(|level| level.and_then(|l| step(l, target, ceiling)))
            .collect();

        // Locked, we move them all by the smallest factor any of them wants.
        // That puts the most constrained channel on target and keeps the rest
        // below it. Infrared measures what is in the way, not color, so it is
        // never part of the lock
        let locked = self.lock_white_balance.then(|| {
            steps
                .iter()
                .zip(windows)
                .filter(|(_, w)| w.id != IR)
                .filter_map(|(s, _)| *s)
                .fold(f64::INFINITY, |a, b| a.min(b))
        });

        Ok(windows
            .iter()
            .zip(&steps)
            .map(|(w, own)| {
                let scale = match locked {
                    Some(f) if w.id != IR && f.is_finite() => Some(f),
                    _ => *own,
                };
                match scale {
                    Some(s) => (f64::from(w.exposure) * s)
                        .round()
                        .clamp(f64::from(limit.start), f64::from(limit.last))
                        as u32,
                    None => w.exposure,
                }
            })
            .collect())
    }
}

impl Metering {
    /// The high tail of each channel, in the order `layout` lists them
    ///
    /// What the exposures get decided from, so it is worth being able to look
    /// at on its own
    pub fn measure(&self, layout: &Layout, raw: &[u8]) -> Result<Vec<Option<u16>>, Error> {
        (0..layout.channels.len())
            .map(|channel| tail(layout, raw, channel, self.percentile))
            .collect()
    }
}

/// Full scale for a sample of `bits` valid bits
fn ceiling(bits: u8) -> u16 {
    match bits {
        0 | 16.. => u16::MAX,
        b => (1u16 << b) - 1,
    }
}

/// What to scale an exposure by to move `level` onto `target`
///
/// A channel at full scale could be anywhere above it, so we halve it and
/// measure again. One reading zero gives us nothing to scale.
fn step(level: u16, target: u16, ceiling: u16) -> Option<f64> {
    match level {
        l if l >= ceiling => Some(0.5),
        0 => None,
        l => Some(f64::from(target) / f64::from(l)),
    }
}

/// The `percentile` brightest sample of one channel of `raw`, or `None` if the
/// pass carried no samples for it
fn tail(
    layout: &Layout,
    raw: &[u8],
    channel: usize,
    percentile: f32,
) -> Result<Option<u16>, Error> {
    let mut samples = plane(layout, raw, channel)?;
    if samples.is_empty() {
        return Ok(None);
    }
    samples.sort_unstable();
    let at = (samples.len() - 1) as f32 * percentile.clamp(0.0, 1.0);
    Ok(samples.get(at as usize).copied())
}

/// Every sample of one channel, pulled out of the interleaved stream
///
/// 2-11-3-1 format 1. Each line holds every channel's row end to end, so one
/// channel is a fixed slice repeated at the line stride.
fn plane(layout: &Layout, raw: &[u8], channel: usize) -> Result<Vec<u16>, Error> {
    if !layout
        .interleaving
        .contains(ColorInterleaving::LINE_WITHOUT_DISTANCE)
    {
        return Err(Error::Unsupported {
            op: "metering",
            reason: format!("{:?} is not a layout this reads yet", layout.interleaving),
        });
    }

    let width = layout.pixels as usize * usize::from(layout.bytes_per_sample);
    let stride = layout.bytes_per_line() as usize;
    let wide = layout.bytes_per_sample == 2;

    let mut out = Vec::with_capacity(layout.pixels as usize * layout.lines as usize);
    for line in raw.chunks_exact(stride) {
        let row = &line[channel * width..(channel + 1) * width];
        if wide {
            out.extend(
                row.chunks_exact(2)
                    .map(|s| u16::from_be_bytes([s[0], s[1]])),
            );
        } else {
            out.extend(row.iter().map(|&s| u16::from(s)));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{
        caps::{
            Page,
            address::Address,
            identity::Identity,
            other::Features,
            set_window::{ColorInterleaving, SetWindowFunction},
        },
        window::{Composition, LENGTH},
    };

    /// Enough of a unit to carry an exposure range wide enough not to clamp
    fn caps() -> Capabilities {
        let mut p = vec![0u8; 91];
        p[1] = Address::PAGE_CODE;
        p[3] = 87;
        p[18..20].copy_from_slice(&4000u16.to_be_bytes());
        p[20..22].copy_from_slice(&4000u16.to_be_bytes());
        let address = Address::try_from(&Page::new(Address::PAGE_CODE, p).unwrap()).unwrap();

        let mut d = vec![0u8; 28];
        d[1] = SetWindowFunction::PAGE_CODE;
        d[3] = 24;
        d[17..21].copy_from_slice(&1u32.to_be_bytes());
        d[21..25].copy_from_slice(&0x3FFFFFFu32.to_be_bytes());
        let set_window =
            SetWindowFunction::try_from(&Page::new(SetWindowFunction::PAGE_CODE, d).unwrap())
                .unwrap();

        let mut e = vec![0u8; 39];
        e[1] = Features::PAGE_CODE;
        e[3] = 35;
        let features = Features::try_from(&Page::new(Features::PAGE_CODE, e).unwrap()).unwrap();

        let mut i = vec![0u8; 36];
        i[4] = 31;

        Capabilities {
            identity: Identity::parse(&i).unwrap(),
            address,
            features,
            set_window,
            ccd: None,
            frames: None,
        }
    }

    const PIXELS: u32 = 4;
    const LINES: u32 = 2;

    /// Built through the real constructor so the set has to be legal
    fn layout(windows: &[Window]) -> Layout {
        Layout::new(&caps(), windows, 4000).unwrap()
    }

    fn windows(ids: &[u8], exposure: u32) -> Vec<Window> {
        let visible = ids.iter().filter(|&&id| id != IR).count();
        ids.iter()
            .map(|&id| {
                let mut w = Window::try_from(&[0u8; LENGTH][..]).unwrap();
                w.id = id;
                w.exposure = exposure;
                w.resolution = (4000, 4000);
                w.size = (PIXELS, LINES);
                w.bpp = 16;
                w.color_interleaving = ColorInterleaving::LINE_WITHOUT_DISTANCE;
                w.composition = match visible {
                    1 => Composition::MultilevelBW,
                    _ => Composition::MultilevelRGB,
                };
                w
            })
            .collect()
    }

    /// One line is every channel's row end to end
    fn raw(levels: &[u16]) -> Vec<u8> {
        let mut out = Vec::new();
        for _ in 0..LINES {
            for &level in levels {
                for _ in 0..PIXELS {
                    out.extend_from_slice(&level.to_be_bytes());
                }
            }
        }
        out
    }

    /// Linear in integration time, so the step is just the ratio
    #[test]
    fn each_channel_lands_on_the_target() {
        let m = Metering {
            target: 1.0,
            ..Default::default()
        };
        let w = windows(&[1, 2, 3], 1000);
        // A third, a half and a quarter of full scale
        let got = m
            .apply(&caps(), &layout(&w), &raw(&[21845, 32767, 16383]), &w)
            .unwrap();
        assert_eq!(got, vec![3000, 2000, 4000]);
    }

    /// Locked, they all move by the smallest factor any of them asked for
    #[test]
    fn locking_scales_the_set_by_its_most_constrained_channel() {
        let m = Metering {
            target: 1.0,
            lock_white_balance: true,
            ..Default::default()
        };
        let w = windows(&[1, 2, 3], 1000);
        let got = m
            .apply(&caps(), &layout(&w), &raw(&[21845, 32767, 16383]), &w)
            .unwrap();
        // Green asked for 2x and wants it least, so nothing overshoots
        assert_eq!(got, vec![2000, 2000, 2000]);
    }

    /// A channel at full scale could be anywhere above it, so it halves
    #[test]
    fn a_clipped_channel_comes_down_instead_of_scaling() {
        let m = Metering {
            target: 1.0,
            ..Default::default()
        };
        let w = windows(&[1, 2, 3], 1000);
        let got = m
            .apply(&caps(), &layout(&w), &raw(&[65535, 32767, 32767]), &w)
            .unwrap();
        assert_eq!(got, vec![500, 2000, 2000]);
    }

    /// Infrared measures what is in the way, not color, so a lock leaves it out
    #[test]
    fn infrared_meters_on_its_own_even_when_locked() {
        let m = Metering {
            target: 1.0,
            lock_white_balance: true,
            ..Default::default()
        };
        let w = windows(&[1, 2, 3, IR], 1000);
        let got = m
            .apply(
                &caps(),
                &layout(&w),
                &raw(&[21845, 32767, 32767, 16383]),
                &w,
            )
            .unwrap();
        // The visible three lock to green's 2x; infrared takes its own 4x
        assert_eq!(got, vec![2000, 2000, 2000, 4000]);
    }

    /// A dark channel gives us nothing to scale, so it keeps what it had
    #[test]
    fn a_dark_channel_keeps_what_it_had() {
        let m = Metering::default();
        let w = windows(&[1, 2, 3], 1000);
        let got = m
            .apply(&caps(), &layout(&w), &raw(&[0, 32767, 32767]), &w)
            .unwrap();
        assert_eq!(got[0], 1000);
    }
}
