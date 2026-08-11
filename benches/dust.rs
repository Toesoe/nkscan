use divan::counter::ItemsCount;
use nkscan::{
    dust,
    protocol::decode::{Image, Samples},
};
use std::sync::LazyLock;
use tiff::decoder::{Decoder, DecodingResult, Limits};

fn main() {
    divan::main();
}

/// A plane's worth of linear samples, the shape `to_density` actually sees
fn plane(n: usize) -> Vec<u16> {
    (0..n).map(|i| (i % 65536) as u16).collect()
}

/// The biggest image, a 6x9 frame at 4000 DPI
#[divan::bench(args = [9440 * 14160])]
fn to_density(bencher: divan::Bencher, n: usize) {
    bencher
        .counter(ItemsCount::new(n))
        .with_inputs(|| plane(n))
        .bench_values(|samples| dust::to_density(&samples));
}

fn read_u16(path: &str) -> Option<Vec<u16>> {
    let file = std::fs::File::open(path)
        .inspect_err(|e| eprintln!("{path}: {e}"))
        .ok()?;
    let mut decoder = Decoder::new(std::io::BufReader::new(file))
        .inspect_err(|e| eprintln!("{path}: {e}"))
        .ok()?
        .with_limits(Limits::unlimited());
    match decoder.read_image() {
        Ok(DecodingResult::U16(v)) => Some(v),
        Ok(other) => {
            eprintln!("{path}: not 16-bit samples ({other:?})");
            None
        }
        Err(e) => {
            eprintln!("{path}: {e}");
            None
        }
    }
}

/// The TIFF itself is still chunky RGB; `Samples` wants planes apart
fn deinterleave3(chunky: &[u16]) -> Vec<Vec<u16>> {
    let mut planes: Vec<Vec<u16>> = (0..3)
        .map(|_| Vec::with_capacity(chunky.len() / 3))
        .collect();
    for pixel in chunky.chunks_exact(3) {
        for (plane, &v) in planes.iter_mut().zip(pixel) {
            plane.push(v);
        }
    }
    planes
}

static SCAN: LazyLock<Option<Samples>> = LazyLock::new(|| {
    Some(Samples {
        colors: deinterleave3(&read_u16("scan_1.tiff")?),
        ir: read_u16("scan_1_IR.tiff"),
    })
});

#[divan::bench]
fn clean(bencher: divan::Bencher) {
    let Some(samples) = SCAN.as_ref() else {
        eprintln!("no scan_1.tiff/scan_1_IR.tiff at the repo root, skipping");
        return;
    };
    let counted = samples.colors.iter().map(Vec::len).sum::<usize>()
        + samples.ir.as_ref().map_or(0, Vec::len);
    bencher
        .counter(ItemsCount::new(counted))
        .with_inputs(|| Image {
            colors: samples.colors.iter().map(Vec::as_slice).collect(),
            ir: samples.ir.as_deref().unwrap_or(&[]),
            rows: 8964,
            cols: 8820,
            bits: 16,
        })
        .bench_values(dust::clean);
}
