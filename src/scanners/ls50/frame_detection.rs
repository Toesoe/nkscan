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

use image::Rgb;

use crate::decode::Image;

use std::fs::File;
use std::io::Write;

/// how many pixels to discard on each X edge of the frame. the actual image is 90px wide so discarding 8px works well
const DISCARD_PIXELS_FROM_FRAME_SIDE: u32 = 4;

/// halfwidth for moving average
const SMOOTHING_RADIUS: usize = 20;
const EDGE_DETECTION_EPSILON: f32 = 1.0;

// at 97dpi
const PIXELS_PER_MM: f32 = 3.8189;

/// extract frame boundary candidates from a thumbnail image
#[derive(Debug, Clone)]
pub struct FrameDetector {
    /// Film type for boundary detection. No influence on output
    pub film_type: FilmType,
    pub frame_size: FrameSize,
}

#[derive(Debug, Clone, Copy)]
struct Boundary {
    y: u32,
    detected: bool,
}

#[derive(Debug, Clone, Copy)]
struct Edge {
    y: u32,
    strength: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub index: usize,
    pub start_line_y: u32, // pixel index in preview strip, not hardware address
    pub content_score: f32, // trust in this frame
    pub is_empty: bool,
    pub is_leader: bool, // override empty calculation
    pub is_interpolated: bool
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilmType {
    Negative, // start frame transition is HighToLow
    Positive, // start frame transition is LowToHigh
}

impl FilmType {
    fn edge_sign(self) -> f32 {
        match self {
            FilmType::Negative => -1.0,
            FilmType::Positive => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FrameSize {
    HalfFrame,
    FullFrame,
    XPan,
    TexPan6x6,
    TexPan6x7,
    TexPan6x8,
    TexPan6x9,
    TexPan6x12,
    TexPan6x17,
    TexPan4x5,
}

impl FrameSize {
    pub fn in_mm(self) -> usize {
        match self {
            FrameSize::HalfFrame => 18,
            FrameSize::FullFrame => 36,
            FrameSize::XPan => 65,
            FrameSize::TexPan6x6 => 60,
            FrameSize::TexPan6x7 => 70,
            FrameSize::TexPan6x8 => 80,
            FrameSize::TexPan6x9 => 90,
            FrameSize::TexPan6x12 => 120,
            FrameSize::TexPan6x17 => 170,
            FrameSize::TexPan4x5 => 127,
        }
    }
}

impl Default for FrameDetector {
    fn default() -> Self {
        Self {
            film_type: FilmType::Negative,
            frame_size: FrameSize::FullFrame,
        }
    }
}

impl FrameDetector {
    /// Return Y coordinates that appear to be frame boundaries.
    pub fn detect_frame_boundaries(&self, image: &Image) -> Vec<Frame> {
        let profile = self.line_profile(image);
        let pitch_profile = self.smooth(&profile);
        let derivative = self.derivative(&profile);

        let pitch = self
            .estimate_pitch(&self.derivative(&pitch_profile))
            .unwrap();

        let mut candidates = Vec::new();
        let mut magnitudes: Vec<f32> = derivative.iter().map(|x| x.abs()).collect();

        magnitudes.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // use the top 2% peaks for thresholding a transition
        let edge_threshold = magnitudes[(magnitudes.len() as f32 * 0.98) as usize];

        for y in 1..derivative.len() - 1 {
            let value = derivative[y];
            let strength = value.abs();

            if strength < edge_threshold || strength < EDGE_DETECTION_EPSILON {
                continue;
            }

            // skip edges with wrong transition, we only care about the start of a frame
            if value.signum() != self.film_type.edge_sign() {
                continue;
            }

            // ensure the current edge is actually different from the previous one
            if strength >= derivative[y - 1].abs() && strength >= derivative[y + 1].abs() {
                candidates.push(Edge {
                    y: y as u32,
                    strength
                });
            }
        }

        dbg!(&candidates);

        if candidates.len() < 2 {
            return self.edges_to_frames(candidates);
        }

        // merge adjacent detections into one edge, keep highest peak
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
            return self.edges_to_frames(peaks);
        }

        let mut frames = self.fit_boundaries(&peaks, &profile, pitch);
        self.classify_empty_frames(&mut frames);

        dbg!(&peaks);
        dbg!(&frames);

        frames
    }

    /// detect frame pitch in pixels from the preview strip
    fn estimate_pitch(&self, derivative: &[f32]) -> Option<f32> {
        let expected = self.frame_size.in_mm() as f32 * PIXELS_PER_MM;

        let min_lag = (expected * 0.8) as usize;
        let max_lag = (expected * 1.2) as usize;

        let mut best_lag = None;
        let mut best_score = 0.0;

        for lag in min_lag..=max_lag {
            let mut score = 0.0;

            for i in 0..derivative.len() - lag {
                score += derivative[i] * derivative[i + lag];
            }

            let denom_a: f32 = derivative[..derivative.len() - lag]
                .iter()
                .map(|x| x * x)
                .sum();

            let denom_b: f32 = derivative[lag..].iter().map(|x| x * x).sum();

            score /= (denom_a * denom_b).sqrt();

            if score > best_score {
                best_score = score;
                best_lag = Some(lag as f32);
            }
        }

        best_lag
    }

    fn fit_boundaries(&self, peaks: &[Edge], profile: &[f32], pitch: f32) -> Vec<Frame> {
        let tolerance = (pitch * 0.25) as u32;

        // ------------------------------------------------------------------
        // Pass 1: find the strongest chain using only detected boundaries
        // ------------------------------------------------------------------

        let mut best: Vec<Boundary> = Vec::new();
        let mut best_score = i32::MIN;

        for start in peaks {
            let mut result = Vec::new();
            let mut expected = start.y as f32;

            loop {
                // candidate chain selection. prioritize closer boundaries
                let candidate = peaks
                    .iter()
                    .filter(|p| p.y.abs_diff(expected as u32) <= tolerance)
                    .copied()
                    .max_by(|a, b| {
                        let edge_score = |p: &Edge| {
                            let distance = p.y.abs_diff(expected as u32) as f32 / pitch;
                            p.strength * (1.0 - distance)
                        };

                        edge_score(a)
                            .partial_cmp(&edge_score(b))
                            .unwrap()
                    });

                match candidate {
                    Some(edge) => {
                        result.push(Boundary {
                            y: edge.y,
                            detected: true,
                        });

                        expected += pitch;
                    }

                    None => break,
                }

                if expected > profile.len() as f32 {
                    break;
                }
            }

            let score =
                result.len() as i32 * 1000
                - result
                    .windows(2)
                    .map(|w| {
                        (w[1].y as i32 - w[0].y as i32 - pitch as i32).abs()
                    })
                    .sum::<i32>();

            if score > best_score {
                best_score = score;
                best = result;
            }
        }

        if best.is_empty() {
            return Vec::new();
        }
        let scores: Vec<f32> = best
            .iter()
            .map(|b| self.frame_content_score(profile, b.y, pitch))
            .collect();

        let content_threshold = self.frame_score_threshold(&scores);

        let mut recovered = Vec::new();

        let mut expected = best[0].y;
        let end = profile.len() as u32;

        while expected < end {
            let candidate = peaks
                .iter()
                .filter(|p| p.y.abs_diff(expected) <= tolerance)
                .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap());

            match candidate {
                Some(edge) => {
                    recovered.push(Boundary {
                        y: edge.y,
                        detected: true,
                    });

                    expected = edge.y + pitch as u32;
                }

                None => {
                    let score = self.frame_content_score(profile, expected, pitch);

                    if score > content_threshold {
                        recovered.push(Boundary {
                            y: expected,
                            detected: false,
                        });
                    }

                    expected += pitch as u32;
                }
            }
        }

        let mut best = recovered;

        let scores: Vec<f32> = best
            .iter()
            .map(|b| self.frame_content_score(profile, b.y, pitch))
            .collect();

        let content_threshold = self.frame_score_threshold(&scores);

        let mut added_leader = false;

        let first = best[0].y;
        let possible_first = first as i32 - pitch as i32;

        if possible_first >= 0 {
            let score = self.frame_content_score(profile, possible_first as u32, pitch);

            if score > content_threshold {
                best.insert(
                    0,
                    Boundary {
                        y: possible_first as u32,
                        detected: false,
                    },
                );

                added_leader = true;
            }
        }

        best.windows(2)
            .enumerate()
            .map(|(index, window)| {
                let boundary = window[0];

                Frame {
                    index,
                    start_line_y: boundary.y,
                    content_score: self.frame_content_score(profile, boundary.y, pitch),
                    is_empty: false,
                    is_leader: added_leader && index == 0,
                    is_interpolated: !boundary.detected,
                }
            })
            .collect()
    }

    /// collapse each scanline into a single brightness value
    fn line_profile(&self, image: &Image) -> Vec<f32> {
        (0..image.rgb.height())
            .map(|y| {
                let mut sum = 0f32;

                // perform analysis on frame region only
                for x in DISCARD_PIXELS_FROM_FRAME_SIDE
                    ..image.rgb.width() - DISCARD_PIXELS_FROM_FRAME_SIDE
                {
                    let pixel = image.rgb.get_pixel(x, y);

                    // use BT.709
                    sum += pixel[0] as f32 * 0.2126;
                    sum += pixel[1] as f32 * 0.7152;
                    sum += pixel[2] as f32 * 0.0722;
                }
                sum / (image.rgb.width() - (2 * DISCARD_PIXELS_FROM_FRAME_SIDE)) as f32
            })
            .collect()
    }

    fn frame_content_score(&self, profile: &[f32], start: u32, pitch: f32) -> f32 {
        let start = start as usize;
        let end = (start + pitch as usize).min(profile.len());

        if end <= start {
            return 0.0;
        }

        let region = &profile[start..end];

        let mean = region.iter().sum::<f32>() / region.len() as f32;

        let variance = region
            .iter()
            .map(|v| {
                let d = *v - mean;
                d * d
            })
            .sum::<f32>()
            / region.len() as f32;

        let detail_energy: f32 = region
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .sum::<f32>()
            / region.len() as f32;

        variance.sqrt() * 0.5 + detail_energy * 0.5
    }

    fn frame_score_threshold(&self, scores: &[f32]) -> f32 {
        if scores.is_empty() {
            return 0.0;
        }

        let mut sorted = scores.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median = sorted[sorted.len() / 2];

        let mut deviations: Vec<f32> = sorted
            .iter()
            .map(|s| (s - median).abs())
            .collect();

        deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mad = deviations[deviations.len() / 2];

        // accept frames that are not extreme low outliers
        (median - mad * 3.0).max(median * 0.1)
    }

    fn classify_empty_frames(&self, frames: &mut [Frame]) {
        if frames.is_empty() {
            return;
        }

        let scores: Vec<f32> = frames
            .iter()
            .map(|f| f.content_score)
            .collect();

        let threshold = self.frame_score_threshold(&scores);

        for frame in frames.iter_mut() {
            frame.is_empty = frame.content_score < threshold;
        }
    }

    fn edges_to_frames(&self, edges: Vec<Edge>) -> Vec<Frame> {
        edges
            .into_iter()
            .enumerate()
            .map(|(index, edge)| Frame {
                index,
                start_line_y: edge.y,
                content_score: 0.0,
                is_empty: true,
                is_leader: false,
                is_interpolated: true
            })
            .collect()
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

    pub fn dump_frame_overlay(
        image: &Image,
        frames: &[Frame],
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut output = image.rgb.clone();

        for frame in frames {
            let y = frame.start_line_y as u32;

            if y >= output.height() {
                continue;
            }

            for x in 0..20 {
                output.put_pixel(x, y, Rgb([65535, 0, 0]));
            }

            for x in output.width() - 20..output.width() {
                output.put_pixel(x, y, Rgb([65535, 0, 0]));
            }
        }

        output.save(path)?;

        Ok(())
    }
}
