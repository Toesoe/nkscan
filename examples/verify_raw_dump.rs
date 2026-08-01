//! TEMPORARY: decode a raw dump through the real FrameDecoder and print a specific pixel, to
//! cross-check hand-transcribed scratch-tool formulas against the actual production decode.
//! Remove once the three-line interleave bug is fixed.

use nkscan::decode::StreamDecoder;
use nkscan::scanners::{
    ScanArea,
    ls9000::{
        decode::FrameDecoder,
        geometry::{CcdMode, Dpi, Multisample, ScanSettings},
        window::BaseQuality,
    },
};

fn main() {
    let path = std::env::args().nth(1).expect("raw dump path");
    let mode = std::env::args().nth(2).expect("singleline|threeline");
    let x: u32 = std::env::args().nth(3).expect("x").parse().unwrap();
    let y: u32 = std::env::args().nth(4).expect("y").parse().unwrap();
    let raw = std::fs::read(&path).expect("read raw dump");

    let settings = ScanSettings {
        ccd_mode: if mode == "singleline" {
            CcdMode::SingleLine
        } else {
            CcdMode::ThreeLine
        },
        ir: false,
        dpi: Dpi::_2000,
        quality: BaseQuality::Scan,
        multisample: Multisample::X1,
        window: ScanArea {
            x_pos: 0,
            y_pos: 0,
            x_size: ScanArea::FILM_WIDTH_DOTS,
            y_size: 8784, // matches the pinned --pitch 56 capture: 4392 * stage_divisor(2)
        },
    };

    let mut decoder = FrameDecoder::new(&settings).expect("settings should be valid");
    decoder
        .push(&raw)
        .expect("raw dump should match expected_bytes for these settings");
    let image = decoder.finish().expect("decode should complete").to_owned();

    eprintln!("dimensions {:?}", image.rgb.dimensions());
    let px = image.rgb.get_pixel(x, y);
    eprintln!("pixel at (x={x}, y={y}): {px:?}");
}
