//! Frame boundary rectangles, DTC 0x88
//!
//! This is the strip's frame table, where the frames sit on the loaded film.
//!
//! Nikon Scan writes it twice: nominal, evenly-spaced rectangles during calibration,
//! then the real per-frame positions once the overview scan has actually located the frames.

use super::{
    Ls9000ed, ScanArea,
    dtc::{self, Dtc},
};
use crate::scsi::{self, Transport};
use image::{ImageBuffer, Rgb};
use std::ops::Deref;

/// One frame's extent, in the same 1/4000-in dots as [`ScanArea`](super::ScanArea)
///
/// Y is along stage travel (which frame), X is along the sensor bar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRect {
    pub y_top: u32,
    pub x_left: u32,
    pub y_bottom: u32,
    pub x_right: u32,
}

impl FrameRect {
    /// A frame `length` dots along the strip at `y_top`, spanning the full 56 mm film width
    ///
    /// Only correct for holders that lay film out in a single row, which is all we have captures for
    /// A holder carrying two rows of 35 mm side by side would put each row in its own X band
    pub fn full_width(y_top: u32, length: u32) -> Self {
        Self {
            y_top,
            x_left: Self::X_LEFT,
            y_bottom: y_top + length,
            x_right: Self::X_RIGHT,
        }
    }

    /// The same centered 8964-dot span [`ScanArea::centered`](super::ScanArea::centered) produces,
    /// derived rather than restated so the two cannot drift
    const X_LEFT: u32 = (ScanArea::SENSOR_DOTS - ScanArea::FILM_WIDTH_DOTS) / 2;
    const X_RIGHT: u32 = Self::X_LEFT + ScanArea::FILM_WIDTH_DOTS;

    fn to_bytes(self) -> [u8; 16] {
        let mut buf = [0u8; 16];
        buf[0..4].copy_from_slice(&self.y_top.to_be_bytes());
        buf[4..8].copy_from_slice(&self.x_left.to_be_bytes());
        buf[8..12].copy_from_slice(&self.y_bottom.to_be_bytes());
        buf[12..16].copy_from_slice(&self.x_right.to_be_bytes());
        buf
    }
}

/// The DTC 0x88 parameter list: a short header plus one rectangle per frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameBoundaries(pub Vec<FrameRect>);

impl FrameBoundaries {
    /// The nominal boundaries Nikon Scan writes during calibration, before any
    /// frame has actually been located: four 6696-dot (6x4.5) frames butted
    /// together from y=2236, for some reason
    pub fn nominal() -> Self {
        Self::evenly_spaced(2236, 6696, 4)
    }

    /// `count` frames of `length` dots each, butted together from `y_top`.
    /// Single-row holders only, see [`FrameRect::full_width`]
    pub fn evenly_spaced(y_top: u32, length: u32, count: u32) -> Self {
        Self(
            (0..count)
                .map(|i| FrameRect::full_width(y_top + i * length, length))
                .collect(),
        )
    }

    /// Find where the frames actually sit, from a decoded overview pass
    ///
    /// Film does not land in the holder at a fixed offset, which is what the Strip Film
    /// Offset control in Nikon Scan exists to correct, so the nominal table is only ever a
    /// starting point.
    ///
    /// `None` if the strip carries too little detail to place anything, which is the honest
    /// answer for blank or unexposed film.
    ///
    /// Only `frames` is needed. Frame sizes are open-ended (6x8, 6x7, 6x12, custom holders),
    /// so the length is solved for rather than looked up.
    pub fn detect<C>(overview: &ImageBuffer<Rgb<u16>, C>, frames: usize) -> Option<Self>
    where
        C: Deref<Target = [u16]>,
    {
        let score = frame_score(overview)?;
        let fit = fit_frames(&score, frames)?;
        let extents = even_up(&frame_extents(&score, &fit, frames), score.len());

        Some(Self(
            extents
                .iter()
                .map(|&(start, end)| {
                    FrameRect::full_width(
                        start as u32 * ScanArea::OVERVIEW_DIVISOR,
                        (end - start) as u32 * ScanArea::OVERVIEW_DIVISOR,
                    )
                })
                .collect(),
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4 + 16 * self.0.len());
        // Total length includes the 4-byte header itself
        buf.extend_from_slice(&(4 + 16 * self.0.len() as u16).to_be_bytes());
        buf.push(self.0.len() as u8);
        buf.push(0x00); // reserved
        for rect in &self.0 {
            buf.extend_from_slice(&rect.to_bytes());
        }
        buf
    }
}

/// A frame layout: `frames` windows of `length` rows, the first at `offset`, `pitch` apart
#[derive(Debug)]
struct Fit {
    offset: usize,
    pitch: usize,
    length: usize,
    score: f32,
}

/// Per-row detail and brightness, the two ways a row can show film rather than holder
///
/// Detail is a high-pass along x, so a smooth illumination falloff across the sensor bar
/// contributes nothing, and the strongest channel wins so nothing assumes which one carries
/// the structure.
///
/// Brightness catches what detail cannot: a frame of clear sky or a flat color has no local
/// variation at all, but still transmits far more light than the opaque holder between the
/// apertures. That comparison is film against holder rather than dark against light, so it
/// holds for negatives and positives alike.
fn row_profiles<C>(overview: &ImageBuffer<Rgb<u16>, C>) -> (Vec<f32>, Vec<f32>)
where
    C: Deref<Target = [u16]>,
{
    let (width, height) = overview.dimensions();
    (0..height)
        .map(|y| {
            let detail = (0..3)
                .map(|channel| {
                    let total: u64 = (1..width)
                        .map(|x| {
                            let here = overview.get_pixel(x, y).0[channel];
                            let left = overview.get_pixel(x - 1, y).0[channel];
                            u64::from(here.abs_diff(left))
                        })
                        .sum();
                    total as f32 / (width - 1) as f32
                })
                .fold(0.0, f32::max);

            let level: u64 = (0..width)
                .map(|x| {
                    let p = overview.get_pixel(x, y).0;
                    (u64::from(p[0]) + u64::from(p[1]) + u64::from(p[2])) / 3
                })
                .sum();

            (detail, level as f32 / width as f32)
        })
        .unzip()
}

/// How far a row's detail exceeds the noise the same row's brightness would produce anyway
///
/// Raw brightness cannot be used on its own: the film between frames is unexposed, which is
/// clear base on a negative and D-max on a slide, so it sits at opposite ends of the range
/// depending on the film. Raw detail cannot either, because photon noise grows as the square
/// root of the signal, so bright empty base looks as textured as a dim frame.
///
/// Dividing one by the other measures structure beyond what noise explains. Opaque holder,
/// dark D-max and bright clear base then all land on the same floor, whatever the film.
fn detail_over_noise<C>(overview: &ImageBuffer<Rgb<u16>, C>) -> Vec<f32>
where
    C: Deref<Target = [u16]>,
{
    let (detail, level) = row_profiles(overview);
    detail
        .iter()
        .zip(&level)
        .map(|(d, l)| d / l.max(1.0).sqrt())
        .collect()
}

/// How far above the noise floor a row has to sit before it counts as a frame outright
const SATURATION: f32 = 1.0;
/// Without at least this much above the floor somewhere there is nothing to lock onto
const MIN_EXCESS: f32 = 0.3;

/// Rescale so empty film reads 0 and anything clearly exposed reads 1
///
/// The floor is measured rather than assumed, since it depends on the scanner's gain. The
/// answer wanted here is nearly binary, so this saturates rather than scaling linearly: a
/// thin frame and a dense one should both read as "frame".
fn frame_score<C>(overview: &ImageBuffer<Rgb<u16>, C>) -> Option<Vec<f32>>
where
    C: Deref<Target = [u16]>,
{
    let excess = detail_over_noise(overview);

    let mut sorted = excess.clone();
    sorted.sort_by(f32::total_cmp);
    let floor = sorted[sorted.len() / 10].max(f32::MIN_POSITIVE);
    let peak = sorted[sorted.len() * 95 / 100];
    if peak / floor - 1.0 < MIN_EXCESS {
        return None;
    }

    Some(
        excess
            .iter()
            .map(|v| ((v / floor - 1.0) / SATURATION).clamp(0.0, 1.0))
            .collect(),
    )
}
/// A layout has to beat this to count as found. Windows covering the whole strip score
/// around 0.5 whatever the film, so anything at or below that has found nothing.
const MIN_FIT: f32 = 0.6;

/// Solve for the layout that best explains the profile
///
/// Holder apertures are evenly spaced, so the whole thing is three numbers. Fitting them
/// together is far more robust than thresholding row by row: a frame with no content, a
/// blank sky or an unexposed shot, just contributes nothing instead of deleting a boundary.
fn fit_frames(score: &[f32], frames: usize) -> Option<Fit> {
    let rows = score.len();
    if frames == 0 || rows < frames * 2 {
        return None;
    }

    let mut prefix = vec![0.0f32; rows + 1];
    for (i, s) in score.iter().enumerate() {
        prefix[i + 1] = prefix[i] + s;
    }
    let total = prefix[rows];
    let window = |start: usize, len: usize| prefix[start + len] - prefix[start];

    // The frames plus their gaps have to fit the strip, which bounds the search on its own
    let mut best: Option<Fit> = None;
    for length in (rows / (2 * frames)).max(1)..=(rows / frames) {
        let max_pitch = if frames > 1 {
            (rows - length) / (frames - 1)
        } else {
            length
        };
        for pitch in length..=max_pitch {
            let span = (frames - 1) * pitch + length;
            for offset in 0..=(rows - span) {
                let inside: f32 = (0..frames)
                    .map(|i| window(offset + i * pitch, length))
                    .sum();
                let outside_rows = (rows - frames * length) as f32;

                // How well the layout explains the profile, counting every row once: busy
                // rows want to be inside a window, quiet ones outside. Averaging inside and
                // outside separately instead lets a window swallow a gap almost for free,
                // because the penalty is diluted across every row it covers.
                let fit = (inside + (outside_rows - (total - inside))) / rows as f32;
                if best.as_ref().is_none_or(|b| fit > b.score) {
                    best = Some(Fit {
                        offset,
                        pitch,
                        length,
                        score: fit,
                    });
                }
            }
        }
    }

    let best = best.filter(|b| b.score >= MIN_FIT)?;
    Some(best)
}

/// A row this far above the empty-film floor is inside a frame
const EDGE_THRESHOLD: f32 = 0.5;

/// Where each frame's exposed area actually starts and ends
///
/// The fit lands on the busy part of each frame but is only as precise as the search grid,
/// so this measures each one directly: the first and last exposed rows anywhere in the half
/// pitch either side of the fitted center. Taking the outermost rather than walking outwards
/// matters because a frame can go quiet in the middle, a plain sky or a dark interior, and
/// walking would stop there.
fn frame_extents(score: &[f32], fit: &Fit, frames: usize) -> Vec<(usize, usize)> {
    let rows = score.len();

    (0..frames)
        .map(|i| {
            let start = fit.offset + i * fit.pitch;
            let center = start + fit.length / 2;
            let low = center.saturating_sub(fit.pitch / 2);
            let high = (center + fit.pitch / 2).min(rows - 1);

            // A run this long, so a single noisy row can't stand in for a frame edge
            const RUN: usize = 3;
            let exposed: Vec<usize> = (low..=high)
                .filter(|&y| {
                    (y.saturating_sub(RUN - 1)..=(y + RUN - 1).min(rows - 1))
                        .collect::<Vec<_>>()
                        .windows(RUN)
                        .any(|w| w.iter().all(|&r| score[r] >= EDGE_THRESHOLD))
                })
                .collect();

            match (exposed.first(), exposed.last()) {
                // A frame with nothing on it keeps the layout's own guess
                (Some(&first), Some(&last)) if last > first => (first, last + 1),
                _ => (start, start + fit.length),
            }
        })
        .collect()
}

/// Give every frame the same length, since they physically are the same size
///
/// Frames vary in how much of themselves they expose, so the median span is the best estimate
/// of the real one. Each frame then keeps its own measured center, which tracks the small
/// drift in film transport that a single pitch cannot.
fn even_up(extents: &[(usize, usize)], rows: usize) -> Vec<(usize, usize)> {
    let mut spans: Vec<usize> = extents.iter().map(|(a, b)| b - a).collect();
    spans.sort_unstable();
    let length = spans[spans.len() / 2];

    extents
        .iter()
        .map(|&(first, last)| {
            let center = (first + last) / 2;
            let start = center
                .saturating_sub(length / 2)
                .min(rows.saturating_sub(length));
            (start, start + length)
        })
        .collect()
}

impl<T> Ls9000ed<T>
where
    T: Transport,
{
    /// Where the frames sit on the loaded film, as the scanner currently has it
    pub fn frame_boundaries(&mut self) -> Result<Vec<u8>, scsi::Error> {
        self.read_framed_dtc(Dtc::FrameBoundaries, None, dtc::HEADER_LEN)
    }

    /// Tell the scanner where the frames sit on the loaded film
    pub fn set_frame_boundaries(
        &mut self,
        boundaries: &FrameBoundaries,
    ) -> Result<(), scsi::Error> {
        self.write_dtc(Dtc::FrameBoundaries, None, boundaries.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wire(hex: &[&str]) -> Vec<u8> {
        hex.join(" ")
            .split_whitespace()
            .map(|b| u8::from_str_radix(b, 16).unwrap())
            .collect()
    }

    /// The nominal write from full_session_cold_start
    #[test]
    fn nominal_matches_real_capture() {
        let expected = wire(&[
            "00 44 04 00",
            "00 00 08 BC 00 00 02 06 00 00 22 E4 00 00 25 0A",
            "00 00 22 E4 00 00 02 06 00 00 3D 0C 00 00 25 0A",
            "00 00 3D 0C 00 00 02 06 00 00 57 34 00 00 25 0A",
            "00 00 57 34 00 00 02 06 00 00 71 5C 00 00 25 0A",
        ]);

        assert_eq!(FrameBoundaries::nominal().to_bytes(), expected);
    }

    /// The 6x9 write from 16x_multisample
    #[test]
    fn six_by_nine_matches_real_capture() {
        let expected = wire(&[
            "00 24 02 00",
            "00 00 08 BC 00 00 02 06 00 00 3C 34 00 00 25 0A",
            "00 00 3C 34 00 00 02 06 00 00 6F AC 00 00 25 0A",
        ]);

        assert_eq!(
            FrameBoundaries::evenly_spaced(2236, 13176, 2).to_bytes(),
            expected
        );
    }

    #[test]
    fn header_length_covers_itself_and_every_rectangle() {
        for count in 1..=8u32 {
            let bytes = FrameBoundaries::evenly_spaced(0, 100, count).to_bytes();
            assert_eq!(bytes.len(), 4 + 16 * count as usize);
            assert_eq!(
                u16::from_be_bytes([bytes[0], bytes[1]]) as usize,
                bytes.len()
            );
            assert_eq!(bytes[2] as u32, count);
        }
    }

    #[test]
    fn frames_are_butted_together() {
        let FrameBoundaries(rects) = FrameBoundaries::evenly_spaced(2236, 6696, 4);
        for pair in rects.windows(2) {
            assert_eq!(pair[0].y_bottom, pair[1].y_top);
        }
        assert!(rects.iter().all(|r| r.x_right - r.x_left == 8964));
    }
}
