//! REMOVE LATER: decode a raw pass into something you can look at
//!
//! The correlation checks in `protocol::decode` prove the planes are put back
//! together consistently. They cannot tell a rotated image from an upright one,
//! or red from blue. That needs eyes.
//!
//! ```text
//! cargo run --release --example decode -- scan.raw 1494 1494 3
//! cargo run --release --example decode -- scan.raw 1494 1494 3 ccd=3 gap=2
//! cargo run --release --example decode -- scan.raw 1494 1494 3 linear
//! ```
//!
//! Writes a 16-bit binary PPM (or PGM for one channel) next to the input.
//!
//! The scanner's data is linear, and Nikon Scan applies gamma host-side, so a
//! linear dump looks almost black on a display that expects sRGB. This gamma
//! corrects for viewing only and is nothing the library should ever do.

use std::{fs, io::Write, path::PathBuf};

use nkscan::protocol::{caps::set_window::ColorInterleaving, decode::Decoder, image::Layout};

/// What a display expects, near enough for judging an image by eye
const DISPLAY_GAMMA: f32 = 2.2;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = PathBuf::from(args.first().map_or("scan.raw", String::as_str));
    let num = |n: usize, or: u32| args.get(n).and_then(|a| a.parse().ok()).unwrap_or(or);
    let (pixels, lines, channels) = (num(1, 1494), num(2, 1494), num(3, 3) as usize);
    let linear = args.iter().any(|a| a == "linear");

    let raw = fs::read(&path)?;
    // `ccd=N gap=N` decodes a pass the CCD's rows were all read into. Both come
    // from the unit: `Address` bytes 86 and 85 over the feed pitch
    let ccd = args
        .iter()
        .find_map(|a| a.strip_prefix("ccd=")?.parse().ok());
    let layout = Layout {
        ccd_lines: ccd.unwrap_or(1),
        registration_gap: args
            .iter()
            .find_map(|a| a.strip_prefix("gap=")?.parse().ok())
            .unwrap_or(1),
        interleaving: match ccd {
            Some(_) => ColorInterleaving::MULTILINE_SIMULTANEOUS,
            None => ColorInterleaving::LINE_WITHOUT_DISTANCE,
        },
        ..Layout::single_line(pixels, lines, (1..=channels as u8).collect())
    };
    let mut decoder = Decoder::new(&layout)?;
    println!(
        "{} bytes, expecting {} for {pixels} x {lines} x {channels}",
        raw.len(),
        decoder.samples() * 2
    );

    let mut out = vec![0u16; decoder.samples()];
    // The same lengths the transport hands over, so this exercises the carry
    for piece in raw.chunks(262_144) {
        decoder.push(piece, &mut out)?;
    }
    let (rows, cols) = decoder.shape();
    println!(
        "{rows} x {cols}, {} blocks, complete={}",
        decoder.decoded(),
        decoder.complete()
    );

    if !linear {
        let g = 1.0 / DISPLAY_GAMMA;
        for v in &mut out {
            *v = ((f32::from(*v) / 65535.0).powf(g) * 65535.0) as u16;
        }
    }

    // Netpbm binary, maxval 65535, samples big-endian. P6 is three channels
    // and P5 is one; anything else has no Netpbm form, so dump the planes
    let (magic, ext) = match channels {
        1 => ("P5", "pgm"),
        3 => ("P6", "ppm"),
        n => anyhow::bail!("{n} channels has no Netpbm form"),
    };
    let dest = path.with_extension(ext);
    let mut file = fs::File::create(&dest)?;
    write!(file, "{magic}\n{cols} {rows}\n65535\n")?;
    let mut bytes = Vec::with_capacity(out.len() * 2);
    for v in &out {
        bytes.extend_from_slice(&v.to_be_bytes());
    }
    file.write_all(&bytes)?;
    println!("wrote {}", dest.display());
    Ok(())
}
