//! REMOVE LATER: decode a raw pass into something you can look at
//!
//! The correlation checks in `protocol::decode` prove the planes are put back
//! together consistently. They cannot tell a rotated image from an upright one,
//! or red from blue. That needs eyes.
//!
//! ```text
//! cargo run --release --example decode -- scan.raw 1494 1494 3
//! cargo run --release --example decode -- scan.raw 1494 1494 3 ccd=3 gap=2
//! cargo run --release --example decode -- scan.raw 1494 1494 3 ccd=3 gap=2 samples=2
//! cargo run --release --example decode -- scan.raw 1494 1494 3 linear
//! ```
//!
//! Writes a 16-bit binary PPM (or PGM for one channel) next to the input.
//!
//! The scanner's data is linear, and Nikon Scan applies gamma host-side, so a
//! linear dump looks almost black on a display that expects sRGB. This gamma
//! corrects for viewing only and is nothing the library should ever do.

use std::{fs, io::Write, path::PathBuf};

use nkscan::protocol::{
    caps::set_window::ColorInterleaving, decode::Decoder, image::Layout, window::Channel,
};

/// What a display expects, near enough for judging an image by eye
const DISPLAY_GAMMA: f32 = 2.2;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = PathBuf::from(args.first().map_or("scan.raw", String::as_str));
    let num = |n: usize, or: u32| args.get(n).and_then(|a| a.parse().ok()).unwrap_or(or);
    let (pixels, lines) = (num(1, 1494), num(2, 1494));
    let linear = args.iter().any(|a| a == "linear");
    // `ids=9,1,2,3` names the channels in SCAN order, which is the order the
    // stream interleaves them. A bare count assumes the color channels alone
    let ids: Vec<u8> = args
        .iter()
        .find_map(|a| a.strip_prefix("ids="))
        .map(|list| list.split(',').filter_map(|i| i.parse().ok()).collect())
        .unwrap_or_else(|| (1..=num(3, 3) as u8).collect());

    let raw = fs::read(&path)?;
    // `ccd=N gap=N` decodes a pass the CCD's rows were all read into. Both come
    // from the unit: `Address` bytes 86 and 85 over the feed pitch
    let ccd = args
        .iter()
        .find_map(|a| a.strip_prefix("ccd=")?.parse().ok());
    let layout = Layout {
        readings_per_line: args
            .iter()
            .find_map(|a| a.strip_prefix("samples=")?.parse().ok())
            .unwrap_or(1),
        ccd_lines: ccd.unwrap_or(1),
        registration_gap: args
            .iter()
            .find_map(|a| a.strip_prefix("gap=")?.parse().ok())
            .unwrap_or(1),
        interleaving: match ccd {
            Some(_) => ColorInterleaving::MULTILINE_SIMULTANEOUS,
            None => ColorInterleaving::LINE_WITHOUT_DISTANCE,
        },
        ..Layout::single_line(pixels, lines, ids.clone())
    };
    let mut decoder = Decoder::new(&layout)?;
    // Multi-sampling makes the wire longer than the image: the readings are
    // averaged away rather than kept
    println!(
        "{} bytes on the wire, expecting {}, for {pixels} x {lines} channels {ids:?}",
        raw.len(),
        layout.total_bytes()
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

    // Netpbm binary, maxval 65535, samples big-endian. P6 carries three
    // channels and P5 one, so the color planes go together and anything else,
    // infrared being the one that turns up, gets a file of its own
    let color: Vec<usize> = (0..ids.len())
        .filter(|&c| Channel::from(ids[c]).is_color())
        .collect();
    write_pnm(
        &path.with_extension("ppm"),
        &out,
        &color,
        ids.len(),
        cols,
        rows,
    )?;
    for (c, id) in ids.iter().enumerate() {
        if Channel::from(*id).is_color() {
            continue;
        }
        let name = format!(
            "{}.{}.pgm",
            path.file_stem().unwrap_or_default().to_string_lossy(),
            format!("{:?}", Channel::from(*id)).to_lowercase()
        );
        write_pnm(
            &path.with_file_name(name),
            &out,
            &[c],
            ids.len(),
            cols,
            rows,
        )?;
    }
    Ok(())
}

/// One Netpbm file holding the named channels, in the order given
fn write_pnm(
    dest: &std::path::Path,
    samples: &[u16],
    channels: &[usize],
    stride: usize,
    cols: usize,
    rows: usize,
) -> anyhow::Result<()> {
    let magic = match channels.len() {
        1 => "P5",
        3 => "P6",
        n => anyhow::bail!("{n} channels has no Netpbm form"),
    };
    let mut file = fs::File::create(dest)?;
    write!(file, "{magic}\n{cols} {rows}\n65535\n")?;
    let mut bytes = Vec::with_capacity(rows * cols * channels.len() * 2);
    for pixel in 0..rows * cols {
        for &c in channels {
            bytes.extend_from_slice(&samples[pixel * stride + c].to_be_bytes());
        }
    }
    file.write_all(&bytes)?;
    println!("wrote {}", dest.display());
    Ok(())
}
