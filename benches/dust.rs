use divan::counter::ItemsCount;
use nkscan::{
    dust::{self, Calibration, Options, Params},
    protocol::decode::{Image, Samples},
};
use std::sync::LazyLock;
use tiff::decoder::{Decoder, DecodingResult, Limits};

fn main() {
    divan::main();
}

/// The real frame's shape: 6x9 at 4000 DPI
const ROWS: usize = 8964;
const COLS: usize = 8820;

/// A plane's worth of linear samples, the shape `to_density` actually sees
fn plane(n: usize) -> Vec<u16> {
    (0..n).map(|i| (i % 65536) as u16).collect()
}

/// The profile every kernel bench runs under
fn params() -> Params {
    Params::new(
        &Options::default(),
        &Calibration {
            c: 0.05,
            ir_ref: 40_000.0,
        },
    )
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

/// The real AE prescan shape for medium format: 666x333 DPI, 1494 sensor
/// pixels by 1098 stage positions, IR present. Same fixture as
/// `protocol::image::readouts::prescan` -- "as the captures deliver it"
const PRESCAN_ROWS: usize = 1494;
const PRESCAN_COLS: usize = 1098;

struct Prescan {
    colors: Vec<Vec<u16>>,
    ir: Vec<u16>,
}

static PRESCAN: LazyLock<Prescan> = LazyLock::new(|| {
    let n = PRESCAN_ROWS * PRESCAN_COLS;
    Prescan {
        colors: vec![plane(n), plane(n), plane(n)],
        ir: plane(n),
    }
});

/// A plane's worth of density values, the shape `confidence` sees. Value
/// doesn't matter -- there is no data-dependent branching -- only the count
fn density_plane(n: usize) -> Vec<f32> {
    plane(n).into_iter().map(f32::from).collect()
}

/// A full-res red and IR plane, the real size `clean` calls `gate` at.
/// `gate` takes raw samples now, fused with its own density transform
#[divan::bench(args = [ROWS * COLS])]
fn gate(bencher: divan::Bencher, n: usize) {
    let p = params();
    bencher
        .counter(ItemsCount::new(n))
        .with_inputs(|| (plane(n), plane(n)))
        .bench_values(|(red, ir)| dust::gate(&red, &ir, &p));
}

/// Same shape as `gate`'s output, the input `confidence` sees
#[divan::bench(args = [ROWS * COLS])]
fn confidence(bencher: divan::Bencher, n: usize) {
    let p = params();
    bencher
        .counter(ItemsCount::new(n))
        .with_inputs(|| density_plane(n))
        .bench_values(|g| dust::confidence(&g, COLS, &p));
}

/// Confidence values that stay under the `w >= 1` short-circuit, so every
/// pixel actually walks its probes -- a worst case, since real confidence
/// clips to 1 for a share of clean pixels that skip the probes entirely
fn sub_one_plane(n: usize) -> Vec<f32> {
    (0..n).map(|i| (i % 100) as f32 / 100.0).collect()
}

/// Full-res plane shape. `g` cycles through the full range so both sides of
/// the dust floor phi show up in the probes
#[divan::bench(args = [ROWS * COLS])]
fn decide(bencher: divan::Bencher, n: usize) {
    let p = params();
    bencher
        .counter(ItemsCount::new(n))
        .with_inputs(|| (density_plane(n), sub_one_plane(n)))
        .bench_values(|(g, w)| dust::decide(&g, &w, ROWS, COLS, &p));
}

/// A mask with roughly `pct`% of pixels flagged, evenly spread. Cost here
/// doesn't depend on clustering (each flagged pixel's cost is independent),
/// so spread vs clustered shouldn't matter, unlike a real scan's blobs
fn mask_plane(n: usize, pct: usize) -> Vec<bool> {
    (0..n).map(|i| i % 100 < pct).collect()
}

/// Full-res shape; flagged fraction matches what the pipeline actually sees
/// on a real scan (1.2%, see the mask dump)
#[divan::bench(args = [ROWS * COLS])]
fn reconstruct_core(bencher: divan::Bencher, n: usize) {
    let p = params();
    bencher
        .counter(ItemsCount::new(n))
        .with_inputs(|| {
            (
                density_plane(n),
                sub_one_plane(n),
                plane(n),
                plane(n),
                plane(n),
                mask_plane(n, 1),
            )
        })
        .bench_values(|(g, w, r, gr, b, mask)| {
            dust::reconstruct_core(&g, &w, [&r, &gr, &b], &mask, &p, ROWS, COLS)
        });
}

#[divan::bench]
fn calibrate(bencher: divan::Bencher) {
    bencher
        .counter(ItemsCount::new(PRESCAN_ROWS * PRESCAN_COLS))
        .with_inputs(prescan_image)
        .bench_values(|prescan| dust::calibrate(&prescan));
}

fn prescan_image() -> Image<'static> {
    Image {
        colors: PRESCAN.colors.iter().map(Vec::as_slice).collect(),
        ir: &PRESCAN.ir,
        rows: PRESCAN_ROWS,
        cols: PRESCAN_COLS,
        bits: 16,
    }
}

/// Nearest-neighbor decimate, so the prescan is the shape AE really hands us
fn decimate(src: &[u16], step: usize) -> Vec<u16> {
    let (rows, cols) = (ROWS / step, COLS / step);
    let mut out = Vec::with_capacity(rows * cols);
    for y in 0..rows {
        for x in 0..cols {
            out.push(src[y * step * COLS + x * step]);
        }
    }
    out
}

/// A prescan decimated off the real scan. The synthetic PRESCAN fixture makes
/// color and IR identical, which gives every crosstalk slope exactly 1.0 --
/// always outside the +-0.2 filter, so c falls back to 0 and the fit never
/// runs. Feeding clean() the full-res frame as its own prescan works too, but
/// then calibrate() costs ~125ms of a number that is supposed to be about
/// everything else
struct Prescan6 {
    colors: Vec<Vec<u16>>,
    ir: Vec<u16>,
}

static SMALL_PRESCAN: LazyLock<Option<Prescan6>> = LazyLock::new(|| {
    let samples = SCAN.as_ref()?;
    Some(Prescan6 {
        colors: samples.colors.iter().map(|p| decimate(p, 6)).collect(),
        ir: decimate(samples.ir.as_ref()?, 6),
    })
});

#[divan::bench]
fn clean(bencher: divan::Bencher) {
    let (Some(samples), Some(pre)) = (SCAN.as_ref(), SMALL_PRESCAN.as_ref()) else {
        eprintln!("no scan_1.tiff/scan_1_IR.tiff at the repo root, skipping");
        return;
    };
    let counted = samples.colors.iter().map(Vec::len).sum::<usize>()
        + samples.ir.as_ref().map_or(0, Vec::len);
    bencher
        .counter(ItemsCount::new(counted))
        .with_inputs(|| {
            // clean() mutates in place, so with_inputs (untimed) gets a fresh
            // clone each sample -- the clone itself isn't what's being
            // measured, clean()'s own work on it is
            let [r, g, b]: [Vec<u16>; 3] =
                samples.colors.clone().try_into().expect("3 color planes");
            let ir = samples.ir.clone().unwrap_or_default();
            let prescan = Image {
                colors: pre.colors.iter().map(Vec::as_slice).collect(),
                ir: &pre.ir,
                rows: ROWS / 6,
                cols: COLS / 6,
                bits: 16,
            };
            (r, g, b, ir, prescan)
        })
        .bench_values(|(mut r, mut g, mut b, ir, prescan)| {
            dust::clean(
                [&mut r, &mut g, &mut b],
                &ir,
                &prescan,
                ROWS,
                COLS,
                &Options::default(),
            )
        });
}
