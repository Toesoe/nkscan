//! Scanning all the available film at once to generate a thumbnail
//!
//! `Address` byte 16 says whether the unit publishes frames at all, and
//! `Frames` says whether it knows where they end. A fixed-format mount does;
//! loose film reports a length of zero until something measures it.
//!
//! `Features` puts thumbnail in the host cooperation bits on both families, so
//! the unit hands us the pass and expects us to make sense of it.

use super::{
    boundaries::{self, Polarity},
    framing,
    pass::Pass,
    window,
};
use crate::{
    error::Error,
    protocol::{
        caps::{
            Capabilities,
            set_window::{ColorInterleaving, ScanKind, ScanMode},
        },
        data::{Boundary, Rect, BoundaryType2, FramePosition, PerfInformation},
        decode::{Image, Samples},
        window::{Flags, Window},
    },
};
use tracing::*;

/// Whether this unit and adapter will thumbnail at all
///
/// Support follows the adapter rather than the model, so this is re-decided
/// whenever the adapter changes
pub fn available(caps: &Capabilities) -> bool {
    caps.set_window.kind.contains(ScanKind::THUMBNAIL)
        && caps.address.thumbnail_resolution.start > 0
}

/// The frame table a thumbnail measures, 2-11-6
///
/// `length` is the frame's extent along the feed, the film format, which
/// nothing advertises. Every rectangle comes out that long: the captures'
/// measured tables move the tops about and leave the heights at the format.
///
/// `polarity` of `None` is worked out from the strip.
pub fn frames(
    caps: &Capabilities,
    pass: &Pass,
    samples: &Samples,
    length: u32,
    polarity: Option<Polarity>,
) -> Result<Boundary, Error> {
    // The window that scans a frame has to be whole readout blocks and has to
    // sit inside the frame the table gives, so the table carries the rounding
    let length = window::whole_blocks(caps, length);
    framing::reachable(caps, length)?;
    let image = Image::new(&pass.layout, samples)?;

    // A thumbnail column is one line pitch of film, and the pass starts where
    // the Y axis does, so a column is an address
    let pitch = pass.layout.line_pitch.max(1);
    let origin = caps.address.y_axis.address_range.start;
    let end = caps.address.y_axis.address_range.last;
    let (left, width) = opening(caps);

    let found = boundaries::detect(&image, (length / pitch) as usize, polarity);
    let frames: Vec<Rect> = found
        .frames
        .iter()
        .map(|frame| origin + frame.col as u32 * pitch)
        .filter(|top| top + length <= end)
        .map(|top| Rect {
            top,
            left,
            bottom: top + length,
            right: left + width,
        })
        .collect();

    info!(
        frames = frames.len(),
        polarity = ?found.polarity,
        pitch = found.pitch as u32 * pitch,
        "measured the loaded strip"
    );
    Ok(Boundary { frames })
}

pub fn frames_type2(
    caps: &Capabilities,
    pass: &Pass,
    samples: &[u16],
    perf_info: &PerfInformation,
    length: u32,
    polarity: Option<Polarity>,
) -> Result<BoundaryType2, Error> {
    // The window that scans a frame has to be whole readout blocks.
    let length = window::whole_blocks(caps, length);
    framing::reachable(caps, length)?;

    let image = Image::new(&pass.layout, samples)?;

    // A thumbnail column is one line pitch of film, and the pass starts
    // where the Y axis does, so a column is an address.
    let pitch = pass.layout.line_pitch.max(1);
    let origin = caps.address.y_axis.address_range.start;
    let end = caps.address.y_axis.address_range.last;

    let found =
        boundaries::detect(&image, (length / pitch) as usize, polarity);

    let frames: Vec<FramePosition> = found
        .frames
        .iter()
        .filter_map(|frame| {
            let top = origin + frame.col as u32 * pitch;

            if top + length <= end {
                FramePosition::new(
                    caps.address.x_axis.optical_dpi,
                    caps.address.thumbnail_resolution.start,
                    frame.col as u32,
                    &perf_info,
                )
            } else {
                None
            }
        })
        .collect();

    info!(
        frames = frames.len(),
        polarity = ?found.polarity,
        pitch = found.pitch as u32 * pitch,
        "measured the loaded strip"
    );

    Ok(BoundaryType2 { frames })
}

/// Perforation calculations for 8Fh BoundaryInformation Type2. Inferred from full roll previews
/// Start offset always seems to be 28 internal units for the first perf
fn perforation_position(y: u32) -> (u16, u8) {
    const PERF_ORIGIN: f64 = 28.0;
    const PERF_PITCH: f64 = 4000.0 * 4.8 / 25.4;

    let position = (y as f64 - PERF_ORIGIN) / PERF_PITCH;
    let number = position.floor() as u16;
    let decimal = ((position - number as f64) * 5.0).floor() as u8;

    (number, decimal)
}

/// Where the adapter's opening sits on the sensor, and how wide it is
///
/// The first published image is the opening: a frame narrower than that is a
/// crop, and cropping is not what a pass over the whole strip is for
fn opening(caps: &Capabilities) -> (u32, u32) {
    let x = &caps.address.x_axis;
    match caps.frames.as_ref().and_then(|f| f.images.first()) {
        Some(opening) => (opening.left, opening.width),
        None => (x.address_range.start, x.boundary),
    }
}

/// Windows over everything the adapter can reach, one per channel
pub(crate) fn windows(caps: &Capabilities) -> Result<Vec<Window>, Error> {
    let y = &caps.address.y_axis;
    let unsupported = |reason: String| Error::Unsupported {
        op: "thumbnail window",
        reason,
    };

    // Line ordering owes the host nothing, where the three-line mode owes it
    // registration. Take it when offered rather than assuming it is
    let offered = caps.set_window.interleaving;
    if !offered.contains(ColorInterleaving::LINE_WITHOUT_DISTANCE) {
        return Err(unsupported(format!(
            "a thumbnail needs line ordering and this unit offers {offered:?}"
        )));
    }

    let (thumb_size, flags) = if caps.identity.model().unwrap().name().starts_with("LS-4") ||
                            caps.identity.model().unwrap().name().starts_with("LS-5") {
                                (250_278, Flags::POSITIVE | Flags::AVERAGING)
                            }
                            else {
                                (y.address_range.last, Flags::empty())
                            };

    let (left, width) = opening(caps);
    let mut windows = window::blank(caps, &window::color_channels(caps))?;
    for w in &mut windows {
        w.resolution = (
            caps.address.thumbnail_resolution.start,
            caps.address.thumbnail_resolution.start,
        );
        // Y starts at the axis rather than the first frame, so the leading
        // edge of the film is in the pass and can be found
        w.origin = (left, y.address_range.start);
        w.size = (width, thumb_size);
        w.scanning_kind = ScanKind::THUMBNAIL;
        w.scanning_mode = ScanMode::NORMAL_QUALITY;
        w.flags = flags;
        w.color_interleaving = ColorInterleaving::LINE_WITHOUT_DISTANCE;
    }
    Ok(windows)
}
