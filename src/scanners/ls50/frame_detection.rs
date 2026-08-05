//! Frame detection from thumbnail scans.
//!
//! Nikon Scan appears to derive frame locations on the host and then generates
//! DTC 0x8F (Boundary Information Type 2). The scanner itself only receives
//! the resulting table.
//!
//! The thumbnail is only 97dpi and very narrow, so a simple 1D profile works
//! surprisingly well. We collapse each scanline into a brightness value,
//! smooth the result, then look for strong transitions corresponding to frame
//! boundaries.

use crate::decode::Image;

/// Extract frame boundary candidates from a thumbnail image.
#[derive(Debug, Clone)]
pub struct FrameDetector {
    /// Half-width of the moving average window.
    pub smoothing_radius: usize,
    /// Minimum derivative magnitude required to consider a transition.
    pub edge_threshold: f32,
    /// Minimum spacing between reported boundaries.
    pub min_distance: u32,
    /// Film type for boundary detection. No influence on output
    pub film_type: FilmType,
}

#[derive(Debug, Clone, Copy)]
struct Edge {
    y: u32,
    strength: f32,
    direction: f32
}

#[derive(Debug, Clone, Copy)]
pub enum FilmType {
    Negative,
    Positive
}

impl Default for FrameDetector {
    fn default() -> Self {
        Self {
            smoothing_radius: 20,
            edge_threshold: 120.0,
            min_distance: 145, // frame pitch in pixels
            film_type: FilmType::Negative
        }
    }
}

impl FrameDetector {
    /// Return Y coordinates that appear to be frame boundaries.
    pub fn detect_frame_boundaries(&self, image: &Image) -> Vec<u32> {
        let profile = self.line_profile(image);
        let profile = self.smooth(&profile);
        let derivative = self.derivative(&profile);

        // First pass: collect all strong edges
        let mut candidates = Vec::new();

        let mut magnitudes: Vec<f32> =
            derivative.iter().map(|x| x.abs()).collect();

        magnitudes.sort_by(|a,b| a.partial_cmp(b).unwrap());

        let edge_threshold =
            magnitudes[(magnitudes.len() as f32 * 0.98) as usize];

        let max = *magnitudes.last().unwrap();

        let epsilon = 1.0; // tune this

        for y in 1..derivative.len() - 1 {
            let value = derivative[y];
            let strength = value.abs();

            if strength < edge_threshold || strength < epsilon {
                continue;
            }

            let correct_polarity = match self.film_type {
                FilmType::Negative => value < 0.0, // bright -> dark
                FilmType::Positive => value > 0.0, // dark -> bright
            };

            if !correct_polarity {
                continue;
            }

            if strength >= derivative[y - 1].abs()
                && strength >= derivative[y + 1].abs()
            {
                candidates.push(Edge {
                    y: y as u32,
                    strength,
                    direction: derivative[y]
                });
            }
        }

        dbg!(max);

        dbg!(&candidates);

        if candidates.len() < 2 {
            return candidates.into_iter().map(|e| e.y).collect();
        }

        // Merge adjacent detections into one edge, keeping the strongest point
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

        let pitch = self.estimate_pitch(&peaks).unwrap();

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

    fn estimate_pitch(&self, peaks: &[Edge]) -> Option<usize> {
        (140..160)
            .max_by_key(|pitch| self.score_pitch(peaks, *pitch))
    }

    fn score_pitch(&self, peaks: &[Edge], pitch: usize) -> usize {
        let tolerance = 10;
        let mut score = 0;

        for start in peaks {
            let mut expected = start.y;

            loop {
                let found = peaks.iter().any(|p| {
                    p.y.abs_diff(expected) <= tolerance
                });

                if found {
                    score += 1;
                }

                expected += pitch as u32;

                if expected > 6103 {
                    break;
                }
            }
        }

        score
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

    /// Collapse each scanline into a single brightness value.
    fn line_profile(&self, image: &Image) -> Vec<f32> {
        (0..image.rgb.height())
            .map(|y| {
                let mut sum = 0f32;

                for x in 0..image.rgb.width() {
                    let pixel = image.rgb.get_pixel(x, y);

                    sum += pixel[0] as f32 * 0.2126;
                    sum += pixel[1] as f32 * 0.7152;
                    sum += pixel[2] as f32 * 0.0722;
                }
                sum / image.rgb.width() as f32
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
            let start = i.saturating_sub(self.smoothing_radius);
            let end = (i + self.smoothing_radius + 1).min(input.len());

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