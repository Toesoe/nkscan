//! Turning a color pass monochrome

use anyhow::{Context, Result, bail};
use moxcms::{ColorProfile, ToneReprCurve};

/// Entries in the table that takes a device value to a linear one
const ENTRIES: usize = u16::MAX as usize + 1;

/// The luminance a monochrome profile says a pixel is
pub struct Luminance {
    /// Device value to linear, one table per channel
    curves: [Vec<f32>; 3],
    /// The Y row of the profile's colorant matrix, which is what makes it
    /// luminance rather than any other axis
    weights: [f32; 3],
}

impl Luminance {
    /// Read the transform out of a monochrome profile
    pub fn from_profile(icc: &[u8]) -> Result<Self> {
        let profile = ColorProfile::new_from_slice(icc)
            .map_err(|e| anyhow::anyhow!("{e:?}"))
            .context("reading the monochrome profile")?;

        let curve = |trc: &Option<ToneReprCurve>, name: &str| -> Result<Vec<f32>> {
            match trc {
                Some(trc) => table(trc),
                None => bail!("the monochrome profile has no {name} tone curve"),
            }
        };
        let curves = [
            curve(&profile.red_trc, "red")?,
            curve(&profile.green_trc, "green")?,
            curve(&profile.blue_trc, "blue")?,
        ];

        // Row 1 of the colorant matrix is Y, and the three sum to the white
        // point's luminance
        let m = profile.colorant_matrix();
        let weights = [m.v[1][0] as f32, m.v[1][1] as f32, m.v[1][2] as f32];
        if weights.iter().sum::<f32>() <= 0.0 {
            bail!("the monochrome profile's colorants carry no luminance");
        }

        Ok(Self { curves, weights })
    }

    /// What this pixel weighs, full scale
    pub fn of(&self, rgb: [u16; 3]) -> u16 {
        let y = self.weights[0] * self.curves[0][usize::from(rgb[0])]
            + self.weights[1] * self.curves[1][usize::from(rgb[1])]
            + self.weights[2] * self.curves[2][usize::from(rgb[2])];
        (y.clamp(0.0, 1.0) * f32::from(u16::MAX)).round() as u16
    }
}

/// A curve as a table over every device value
fn table(trc: &ToneReprCurve) -> Result<Vec<f32>> {
    match trc {
        ToneReprCurve::Lut(lut) if lut.len() >= 2 => {
            let last = lut.len() - 1;
            Ok((0..ENTRIES)
                .map(|v| {
                    let at = v as f32 / u16::MAX as f32 * last as f32;
                    let lo = at as usize;
                    let hi = (lo + 1).min(last);
                    let f = at - lo as f32;
                    let value = f32::from(lut[lo]) * (1.0 - f) + f32::from(lut[hi]) * f;
                    value / f32::from(u16::MAX)
                })
                .collect())
        }
        // A single entry is a gamma, which is how the shorter form encodes one
        ToneReprCurve::Lut(lut) => {
            let gamma = lut.first().map_or(1.0, |g| f32::from(*g) / 256.0);
            Ok(powered(gamma))
        }
        ToneReprCurve::Parametric(p) if p.len() == 1 => Ok(powered(p[0])),
        ToneReprCurve::Parametric(_) => {
            bail!("the monochrome profile's curves are parametric, which this does not evaluate")
        }
    }
}

/// A plain power curve as a table
fn powered(gamma: f32) -> Vec<f32> {
    (0..ENTRIES)
        .map(|v| (v as f32 / u16::MAX as f32).powf(gamma))
        .collect()
}

/// A linear gray space to tag the result with
///
/// Nikon writes theirs against a gray profile with a gamma in it, since their
/// pipeline has encoded by then. Ours has not.
pub fn gray_profile() -> Result<Vec<u8>> {
    ColorProfile::new_gray_with_gamma(1.0)
        .encode()
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .context("building a linear gray profile")
}
