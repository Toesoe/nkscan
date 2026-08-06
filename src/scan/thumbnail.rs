//! Scanning all the available film at once to generate a thumbnail
//!
//! `Address` byte 16 says whether the unit publishes frames at all, and
//! `Frames` says whether it knows where they end. A fixed-format mount does;
//! loose film reports a length of zero until something measures it.
//!
//! `Features` puts thumbnail in the host cooperation bits on both families, so
//! the unit hands us the pass and expects us to make sense of it.

use super::pass::{self, Pass};
use crate::{
    error::Error,
    protocol::{
        caps::{
            Capabilities,
            other::HostCooperation,
            set_window::{ColorInterleaving, ScanKind, ScanMode},
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
pub fn scan(session: &mut Session) -> Result<Pass, Error> {
    if !available(session.capabilities()) {
        return Err(Error::Unsupported {
            op: "thumbnail",
            reason: "this unit and adapter do not offer thumbnail scanning".into(),
        });
    }

    let window = window(session.capabilities())?;
    pass::take(session, std::slice::from_ref(&window), THUMBNAIL_TIMEOUT)
}

/// One window over everything the adapter can reach
fn window(caps: &Capabilities) -> Result<Window, Error> {
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

    let mut w = Window::try_from(&[0u8; LENGTH][..]).expect("a zeroed descriptor is long enough");
    // 2-7 reads the default color on its own, which is the one plane 2-10-6's
    // black and white composition carries
    w.id = Channel::Default.id();
    w.composition = Composition::MultilevelBW;
    w.resolution = (
        caps.address.thumbnail_resolution.start,
        caps.address.thumbnail_resolution.start,
    );
    w.origin = (x.address_range.start, y.address_range.start);
    w.size = (x.boundary, y.address_range.last);
    w.bpp = bpp;
    w.scanning_kind = ScanKind::THUMBNAIL;
    w.scanning_mode = ScanMode::NORMAL_QUALITY;
    w.color_interleaving = ColorInterleaving::LINE_WITHOUT_DISTANCE;
    // 2-10 byte 45: the default, and what the unit reports back for a 0
    w.ae_value = 255;
    Ok(w)
}
