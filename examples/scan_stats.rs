//! Per-channel statistics of a scanned TIFF, for telling exposure problems from encoding ones
//!
//! Percentiles come off a full 65536-bin histogram rather than a sort, which is exact for
//! 16-bit samples and does not need a second copy of the image.

use anyhow::{Result, bail};
use image::ImageReader;
use std::{env, path::PathBuf};

/// One channel's histogram, indexed by sample value
struct Histogram(Vec<u64>);

impl Histogram {
    fn new() -> Self {
        Self(vec![0; 65536])
    }

    fn total(&self) -> u64 {
        self.0.iter().sum()
    }

    /// The sample value at `fraction` through the distribution
    fn percentile(&self, fraction: f64) -> u16 {
        let target = (self.total() as f64 * fraction) as u64;
        let mut seen = 0u64;
        for (value, count) in self.0.iter().enumerate() {
            seen += count;
            if seen >= target.max(1) {
                return value as u16;
            }
        }
        u16::MAX
    }

    fn mean(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            return 0.0;
        }
        let sum: f64 = self
            .0
            .iter()
            .enumerate()
            .map(|(value, count)| value as f64 * *count as f64)
            .sum();
        sum / total as f64
    }

    /// What fraction of samples sit at the very top or bottom of the range
    fn clipped(&self) -> (f64, f64) {
        let total = self.total().max(1) as f64;
        (self.0[0] as f64 / total, self.0[65535] as f64 / total)
    }
}

fn main() -> Result<()> {
    let Some(path) = env::args().nth(1).map(PathBuf::from) else {
        bail!("usage: scan_stats <scan.tiff>");
    };

    // A 4000-DPI medium format frame is comfortably past the decoder's default ceiling
    let mut reader = ImageReader::open(&path)?;
    reader.no_limits();
    let image = reader.decode()?.to_rgb16();
    let (width, height) = image.dimensions();

    let mut histograms = [Histogram::new(), Histogram::new(), Histogram::new()];
    for pixel in image.pixels() {
        for (channel, histogram) in histograms.iter_mut().enumerate() {
            histogram.0[usize::from(pixel.0[channel])] += 1;
        }
    }

    println!("{}", path.display());
    println!(
        "{width}x{height}, {} Mpx\n",
        width as u64 * height as u64 / 1_000_000
    );

    println!(
        "{:<8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "channel", "min", "p1", "p25", "median", "p75", "p99", "p99.9", "mean"
    );
    for (channel, histogram) in ["red", "green", "blue"].iter().zip(&histograms) {
        println!(
            "{:<8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8.0}",
            channel,
            histogram.percentile(0.0),
            histogram.percentile(0.01),
            histogram.percentile(0.25),
            histogram.percentile(0.5),
            histogram.percentile(0.75),
            histogram.percentile(0.99),
            histogram.percentile(0.999),
            histogram.mean(),
        );
    }

    println!("\n{:<8} {:>12} {:>12}", "channel", "at 0", "at 65535");
    for (channel, histogram) in ["red", "green", "blue"].iter().zip(&histograms) {
        let (black, white) = histogram.clipped();
        println!(
            "{channel:<8} {:>11.4}% {:>11.4}%",
            black * 100.0,
            white * 100.0
        );
    }

    // The discriminator. Scanner data is linear, so a viewer that assumes sRGB renders it far
    // darker than it is. If the median lands near mid-gray once encoded, the exposure was
    // never the problem.
    println!(
        "\n{:<8} {:>10} {:>12} {:>12}",
        "channel", "median", "of full", "gamma 2.2"
    );
    for (channel, histogram) in ["red", "green", "blue"].iter().zip(&histograms) {
        let median = f64::from(histogram.percentile(0.5)) / 65535.0;
        println!(
            "{channel:<8} {:>10} {:>11.1}% {:>11.1}%",
            histogram.percentile(0.5),
            median * 100.0,
            median.powf(1.0 / 2.2) * 100.0,
        );
    }

    // Two thumbnails of the same data, which is the whole diagnosis: if the linear one looks
    // dark and the encoded one looks right, the exposure was never the problem.
    if let Some(out) = env::args().nth(2).map(PathBuf::from) {
        let thumb = image::imageops::thumbnail(&image, 700, 700 * height / width);

        let linear = image::ImageBuffer::from_fn(thumb.width(), thumb.height(), |x, y| {
            let p = thumb.get_pixel(x, y).0;
            image::Rgb([(p[0] >> 8) as u8, (p[1] >> 8) as u8, (p[2] >> 8) as u8])
        });
        linear.save(out.with_extension("linear.png"))?;

        let encoded = image::ImageBuffer::from_fn(thumb.width(), thumb.height(), |x, y| {
            let p = thumb.get_pixel(x, y).0;
            let gamma = |v: u16| {
                ((f64::from(v) / 65535.0).powf(1.0 / 2.2) * 255.0)
                    .round()
                    .clamp(0.0, 255.0) as u8
            };
            image::Rgb([gamma(p[0]), gamma(p[1]), gamma(p[2])])
        });
        encoded.save(out.with_extension("gamma.png"))?;

        println!("\nwrote {} .linear.png and .gamma.png", out.display());
    }

    Ok(())
}
