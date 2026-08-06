//! Frame detection from thumbnail scans.
//!
//! Nikon Scan appears to derive frame locations on the host and then generates
//! DTC 0x8F (Boundary Information Type 2). The scanner itself only receives
//! the resulting table. No framing information is received from the scanner, this is all done host-side.
//!
//! The thumbnail is only 97dpi and very narrow, so a simple 1D profile works
//! surprisingly well. We collapse each scanline into a brightness value,
//! smooth the result, then look for strong transitions corresponding to frame
//! boundaries.

use crate::decode::Image;

use std::fs::File;
use std::io::Write;

/// how many pixels to discard on each X edge of the frame
const SLICE_DISCARD_PIXELS_FROM_FRAME_SIDES: u32 = 5;

/// halfwidth for moving average
const SMOOTHING_RADIUS: usize = 20;

const EDGE_DETECTION_EPSILON: f32 = 1.0;

// at 97dpi
const PIXELS_PER_MM: f32 = 3.8189;

/// Extract frame boundary candidates from a thumbnail image.
#[derive(Debug, Clone)]
pub struct FrameDetector {
    /// Film type for boundary detection. No influence on output
    pub film_type: FilmType,
    pub frame_length_mm: usize,
}

#[derive(Debug, Clone, Copy)]
enum EdgeDirection{
    LowToHigh,
    HighToLow,
}

#[derive(Debug, Clone, Copy)]
struct Edge {
    y: u32,
    strength: f32,
    direction: EdgeDirection
}

#[derive(Debug, Clone, Copy)]
pub enum FilmType {
    Negative,
    Positive,
}

impl Default for FrameDetector {
    fn default() -> Self {
        Self {
            film_type: FilmType::Negative,
            frame_length_mm: 36
        }
    }
}

impl FrameDetector {
    /// Return Y coordinates that appear to be frame boundaries.
    pub fn detect_frame_boundaries(&self, image: &Image) -> Vec<u32> {
        let profile = self.line_profile(image);
        let derivative = self.derivative(&profile);

        let pitch = self.estimate_pitch(&self.derivative(&self.smooth(&profile))).unwrap();

        let mut candidates = Vec::new();
        let mut magnitudes: Vec<f32> = derivative.iter().map(|x| x.abs()).collect();

        magnitudes.sort_by(|a,b| a.partial_cmp(b).unwrap());

        // compute a sensible edge threshold. use the top 2% peaks here
        let edge_threshold = magnitudes[(magnitudes.len() as f32 * 0.98) as usize];
        let max = *magnitudes.last().unwrap();
        dbg!(max);

        for y in 1..derivative.len() - 1 {
            let value = derivative[y];
            let strength = value.abs();

            if strength < edge_threshold || strength < EDGE_DETECTION_EPSILON {
                continue;
            }

            // skip edges with wrong polarity
            let correct_polarity = match self.film_type {
                FilmType::Negative => value < 0.0, // bright -> dark = base to image
                FilmType::Positive => value > 0.0, // dark -> bright = base to image
            };

            if !correct_polarity {
                continue;
            }

            // ensure the current edge is actually different from the previous one
            if strength >= derivative[y - 1].abs()
                && strength >= derivative[y + 1].abs()
            {
                candidates.push(Edge {
                    y: y as u32,
                    strength,
                    direction: if derivative[y] > 0.0 { EdgeDirection::LowToHigh } else { EdgeDirection::HighToLow }
                });
            }
        }

        dbg!(&candidates);

        if candidates.len() < 2 {
            return candidates.into_iter().map(|e| e.y).collect();
        }

        // merge adjacent detections into one edge, keeping the strongest point which should be the actual transition
        let mut peaks = Vec::new();
        let mut current = candidates[0];

        for edge in candidates.into_iter().skip(1) {
            if edge.y - current.y <= 5 {
                if edge.strength > current.strength {
                    current = edge;
                }
            } else {
                peaks.push(current);
                current = edge;
            }
        }

        peaks.push(current);

        if peaks.len() < 2 {
            return peaks.into_iter().map(|e| e.y).collect();
        }

        let mut boundaries = self.fit_boundaries(&peaks, pitch);

        boundaries = boundaries
            .windows(2)
            .filter_map(|w| {
                if self.has_frame_content(&profile, w[0], w[1] - w[0]) {
                    Some(w[0])
                } else {
                    None
                }
            })
            .collect();

        dbg!(&peaks);
        dbg!(pitch);
        dbg!(&boundaries);

        boundaries
    }

    fn estimate_pitch(&self, derivative: &Vec<f32>) -> Option<usize> {
        let expected = self.frame_length_mm as f32 * PIXELS_PER_MM;

        let min_lag = (expected * 0.8) as usize;
        let max_lag = (expected * 1.2) as usize;

        let mut best_lag = None;
        let mut best_score = 0.0;

        let mut csv = File::create("/tmp/autocorr.csv").unwrap();
        writeln!(csv, "lag,score").unwrap();

        for lag in min_lag..=max_lag {
            let mut score = 0.0;

            for i in 0..derivative.len() - lag {
                score += derivative[i] * derivative[i + lag];
            }

            let denom_a: f32 = derivative[..derivative.len()-lag]
                .iter()
                .map(|x| x*x)
                .sum();

            let denom_b: f32 = derivative[lag..]
                .iter()
                .map(|x| x*x)
                .sum();

            score /= (denom_a * denom_b).sqrt();

            writeln!(csv, "{},{}", lag, score).unwrap();

            if score > best_score {
                best_score = score;
                best_lag = Some(lag);
            }
        }

        best_lag
    }

    fn fit_boundaries(&self, peaks: &[Edge], pitch: usize) -> Vec<u32> {
        let tolerance = 15;

        let mut best = Vec::new();

        for start in peaks {
            let mut result = Vec::new();
            let mut expected = start.y;

            loop {
                let candidate = peaks
                    .iter()
                    .filter(|p| p.y.abs_diff(expected) <= tolerance)
                    .max_by(|a,b| {
                        a.strength.partial_cmp(&b.strength).unwrap()
                    });

                match candidate {
                    Some(edge) => {
                        result.push(edge.y);
                        expected += pitch as u32;
                    }
                    None => break,
                }

                if result.len() > 45 {
                    break;
                }
            }

            if result.len() > best.len() {
                best = result;
            }
        }

        best
    }


    /// collapse each scanline into a single brightness value
    fn line_profile(&self, image: &Image) -> Vec<f32> {
        (0..image.rgb.height())
            .map(|y| {
                let mut sum = 0f32;

                // perform analysis on frame region only
                for x in SLICE_DISCARD_PIXELS_FROM_FRAME_SIDES..image.rgb.width() - SLICE_DISCARD_PIXELS_FROM_FRAME_SIDES {
                    let pixel = image.rgb.get_pixel(x, y);

                    // use BT.709
                    sum += pixel[0] as f32 * 0.2126;
                    sum += pixel[1] as f32 * 0.7152;
                    sum += pixel[2] as f32 * 0.0722;
                }
                sum / (image.rgb.width() - (2 * SLICE_DISCARD_PIXELS_FROM_FRAME_SIDES)) as f32
            })
            .collect()
    }

    fn has_frame_content(&self, profile: &[f32], y: u32, pitch: u32) -> bool {
        let start = y as usize;
        let end = (start + pitch as usize).min(profile.len());

        if end <= start {
            return false;
        }

        let region = &profile[start..end];

        let min = region.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = region.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        // A real frame has density variation. Empty leader/tail does not.
        max - min > 100.0
    }

    /// Simple moving-average smoothing.
    fn smooth(&self, input: &[f32]) -> Vec<f32> {
        let mut output = vec![0.0; input.len()];

        for i in 0..input.len() {
            let start = i.saturating_sub(SMOOTHING_RADIUS);
            let end = (i + SMOOTHING_RADIUS + 1).min(input.len());

            let sum: f32 = input[start..end].iter().sum();

            output[i] = sum / (end - start) as f32;
        }

        output
    }

    /// First derivative of the signal.
    fn derivative(&self, input: &[f32]) -> Vec<f32> {
        input
            .windows(2)
            .map(|window| window[1] - window[0])
            .collect()
    }
}