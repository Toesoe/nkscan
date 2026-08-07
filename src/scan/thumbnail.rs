//! Scanning all the available film at once to generate a thumbnail
//!
//! `Address` byte 16 says whether the unit publishes frames at all, and
//! `Frames` says whether it knows where they end. A fixed-format mount does;
//! loose film reports a length of zero until something measures it.
//!
//! `Features` puts thumbnail in the host cooperation bits on both families, so
//! the unit hands us the pass and expects us to make sense of it.

use super::{
    expose,
    pass::{self, Pass},
};
use crate::protocol::curves::Curves;
use crate::{
    error::Error,
    protocol::{
        caps::{
            Capabilities,
            other::HostCooperation,
            set_window::{ColorComponents, ColorInterleaving, ScanKind, ScanMode},
        },
        window::{Channel, Composition, LENGTH, Window, deepest_depth},
    },
    session::Session,
};
use std::time::Duration;

/// Everything loaded, at the lowest resolution there is, so give it room
const THUMBNAIL_TIMEOUT: Duration = Duration::from_secs(600);

/// Whether this unit and adapter will thumbnail at all
///
/// Support follows the adapter rather than the model, so this is re-decided
/// whenever the adapter changes
pub fn available(caps: &Capabilities) -> bool {
    caps.set_window.kind.contains(ScanKind::THUMBNAIL)
        && caps.address.thumbnail_resolution.start > 0
}

/// Whether the host has to build the thumbnail itself, `Features` byte 4 bit 0
///
/// Set on both families, so this is the normal case rather than an exception
pub fn host_builds(caps: &Capabilities) -> bool {
    caps.features
        .cooperation
        .contains(HostCooperation::THUMBNAIL)
}

/// Scan everything loaded
pub fn scan(
    session: &mut Session,
    curves: Option<&Curves>,
    samples: &mut Vec<u16>,
) -> Result<Pass, Error> {
    if !available(session.capabilities()) {
        return Err(Error::Unsupported {
            op: "thumbnail",
            reason: "this unit and adapter do not offer thumbnail scanning".into(),
        });
    }

    let windows = windows(session.capabilities())?;
    // A descriptor built from nothing carries no exposure, and equal exposures
    // are not neutral. Nikon Scan thumbnails at the unit's own white balance
    let windows = expose::seed_white_balance(session, &windows)?;
    pass::take(session, &windows, THUMBNAIL_TIMEOUT, curves, samples)
}

/// Windows over everything the adapter can reach, one per channel
///
/// In colour where the unit offers it. Nikon Scan thumbnails with windows 1, 2
/// and 3, the pre-rewrite driver did the same, and boundary finding wants all
/// three: a colour negative's mask leaves one plane a poor edge signal.
fn windows(caps: &Capabilities) -> Result<Vec<Window>, Error> {
    let (x, y) = (&caps.address.x_axis, &caps.address.y_axis);
    let unsupported = |reason: String| Error::Unsupported {
        op: "thumbnail window",
        reason,
    };

    let bpp = deepest_depth(caps.set_window.depth)
        .ok_or_else(|| unsupported("this unit advertises no pixel depth".into()))?;

    // Line ordering owes the host nothing, where the three-line mode owes it
    // registration. Take it when offered rather than assuming it is
    let offered = caps.set_window.interleaving;
    if !offered.contains(ColorInterleaving::LINE_WITHOUT_DISTANCE) {
        return Err(unsupported(format!(
            "a thumbnail needs line ordering and this unit offers {offered:?}"
        )));
    }

    // 2-10-6 has one code for a one-plane output and one for three
    let channels: &[Channel] = match caps.set_window.components.contains(ColorComponents::RGB) {
        true => &[Channel::Red, Channel::Green, Channel::Blue],
        false => &[Channel::Default],
    };
    let composition = match channels.len() {
        1 => Composition::MultilevelBW,
        _ => Composition::MultilevelRGB,
    };

    // The adapter publishes the opening it can actually see, which is inset from
    // the sensor: starting at the axis instead loses as much off the far edge as
    // it gains in holder on the near one
    let (left, width) = match caps.frames.as_ref().and_then(|f| f.images.first()) {
        Some(opening) => (opening.left, opening.width),
        None => (x.address_range.start, x.boundary),
    };

    Ok(channels
        .iter()
        .map(|channel| {
            let mut w =
                Window::try_from(&[0u8; LENGTH][..]).expect("a zeroed descriptor is long enough");
            w.id = channel.id();
            w.composition = composition;
            w.resolution = (
                caps.address.thumbnail_resolution.start,
                caps.address.thumbnail_resolution.start,
            );
            // Y starts at the axis rather than the first frame, so the leading
            // edge of the film is in the pass and can be found
            w.origin = (left, y.address_range.start);
            w.size = (width, y.address_range.last);
            w.bpp = bpp;
            w.scanning_kind = ScanKind::THUMBNAIL;
            w.scanning_mode = ScanMode::NORMAL_QUALITY;
            w.color_interleaving = ColorInterleaving::LINE_WITHOUT_DISTANCE;
            // 2-10 byte 45: the default, and what the unit reports back for a 0
            w.ae_value = 255;
            w
        })
        .collect())
}
