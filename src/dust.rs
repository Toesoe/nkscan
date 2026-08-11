//! An implementation of IR-based dust removal following @a6o's [openICE](https://github.com/a6o/openICE)
//! As to not break any GPL rules, we didn't look at the code at all and followed the pipeline document.
//!
//! Some design notes: gotta go fast at all costs.
//! I don't care about matchin Nikon bit-for-bit. It need to look correct and not waste my time.
//!
//! `to_density` and `from_density` are the only places u16 shows up: raw
//! samples in, a corrected image out. Every step between them stays in f32,
//! so one step's output never has to widen back out of a narrow the step
//! before it just did.

use crate::protocol::decode::Image;
use rayon::prelude::*;
use wide::f32x4;

// How many samples at a time to pass to chunk for rayon
const CHUNK: usize = 1 << 16;

/// The maximum value of a 16-bit sample
const M: u16 = 65535;

/// Widen four linear samples into a float lane
fn widen(nu: [u16; 4]) -> f32x4 {
    f32x4::new(nu.map(|v| v as f32))
}

/// Narrow a lane back to four samples, rounding to nearest and clamping to u16
fn narrow(lane: f32x4) -> [u16; 4] {
    lane.round_int()
        .to_array()
        .map(|v| v.clamp(0, i32::from(u16::MAX)) as u16)
}

/// Map a plane through a per-lane transform, four elements and one thread's chunk at a time.
fn par_map4<S, D>(
    src: &[S],
    pack: impl Fn([S; 4]) -> f32x4 + Sync,
    f: impl Fn(f32x4) -> f32x4 + Sync,
    unpack: impl Fn(f32x4) -> [D; 4] + Sync,
) -> Vec<D>
where
    S: Copy + Default + Send + Sync,
    D: Copy + Default + Send + Sync,
{
    let mut out = vec![D::default(); src.len()];
    src.par_chunks(CHUNK)
        .zip(out.par_chunks_mut(CHUNK))
        .for_each(|(src, dst)| {
            let mut chunks = src.chunks_exact(4).zip(dst.chunks_exact_mut(4));
            for (s, d) in &mut chunks {
                d.copy_from_slice(&unpack(f(pack(s.try_into().unwrap()))));
            }
            let rem = src.len() / 4 * 4;
            let (rest, dst_rest) = (&src[rem..], &mut dst[rem..]);
            if !rest.is_empty() {
                let mut buf = [S::default(); 4];
                buf[..rest.len()].copy_from_slice(rest);
                dst_rest.copy_from_slice(&unpack(f(pack(buf)))[..rest.len()]);
            }
        });
    out
}

/// Fused u16 -> f32x4 -> D(nu) kernel: `D(nu) = M/16 * log2(nu + 1)`
pub fn to_density(samples: &[u16]) -> Vec<f32> {
    let scale = f32x4::splat(f32::from(M) / 16.0);
    par_map4(
        samples,
        widen,
        |nu| (nu + f32x4::ONE).log2() * scale,
        f32x4::to_array,
    )
}

/// Fused D(nu) -> f32x4 -> u16 kernel: `nu = 2^(16D(nu)/M) - 1`
pub fn from_density(values: &[f32]) -> Vec<u16> {
    let scale = f32x4::splat(16.0 / f32::from(M));
    par_map4(
        values,
        f32x4::new,
        |d| (d * scale).exp2() - f32x4::ONE,
        narrow,
    )
}

#[derive(Debug, Clone)]
/// The IR calibration terms from a prescan
pub struct Calibration {
    /// R->IR leakage term.
    pub c: f32,
    pub ir_ref: f32,
}

/// Compute the IR calibration constants from a prescan image
pub fn calibrate(prescan: &Image) -> Calibration {
    // A magic constant that is the "threshold" value for determining if the film is clear in IR
    const TAU: f32 = 8847.23;

    // Create log-densities of the red and IR channel
    let d_r = to_density(prescan.colors[0]);
    let d_ir = to_density(prescan.ir);

    // Find all the indices of "clear" film in the IR
    let clear_idxs: Vec<_> = prescan
        .ir
        .par_iter()
        .enumerate()
        .filter_map(|(idx, &nu)| (nu as f32 > TAU).then_some(idx))
        .collect();

    todo!()
}

/// Remove dust from an image like magic
pub fn clean(image: Image) -> Vec<u16> {
    // 1. Convert all values to their log-density
    let d_r = to_density(image.colors[0]);
    let d_g = to_density(image.colors[1]);
    let d_b = to_density(image.colors[2]);
    let d_ir = to_density(image.ir);

    // The rest of the owl

    // Convert result back from log density into a Vec<u16> and return
    from_density(&d_r)
}
