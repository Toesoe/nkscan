//! Scanning all the available film at once to generate a thumbnail
//!
//!`Address` byte 16 says whether the unit publishes frames at all, and
//!`Frames` says whether it knows where they end.
//! A masked holder does, a strip holder reports a length
//! of zero until something measures it.
//!
//! `Features` puts thumbnail in the host cooperation bits on both families, so
//! the unit hands us the pass and expects us to make sense of it.

use crate::{
    error::Error,
    protocol::{
        caps::{
            other::HostCooperation,
            set_window::{ColorInterleaving, ScanKind, ScanMode},
        },
        data::CooperativeAction,
        image::Layout,
        window::{Channel, Composition, LENGTH, Window, deepest_depth},
    },
    session::Session,
};
use std::time::Duration;
use tracing::*;

/// The whole holder at the lowest resolution there is, so give it room
const THUMBNAIL_TIMEOUT: Duration = Duration::from_secs(600);

/// A thumbnail pass and what came back
#[derive(Debug)]
pub struct Thumbnail {
    /// The stream's shape, as far as 2-10's formula describes it
    pub layout: Layout,
    /// What the unit asked us to do with it
    pub cooperation: Option<CooperativeAction>,
    /// The bytes, however many arrived
    pub data: Vec<u8>,
}

/// Whether this unit and holder will thumbnail at all
///
/// Thumbnail support follows the adapter rather than the model, so this is
/// re-decided whenever the holder changes
pub fn available(session: &Session) -> bool {
    let caps = session.capabilities();
    caps.set_window.kind.contains(ScanKind::THUMBNAIL)
        && caps.address.thumbnail_resolution.start > 0
}

/// Scan the whole holder
///
/// Reads the pass out: a scan whose data is never read locks out every command
/// that follows.
pub fn scan(session: &mut Session) -> Result<Thumbnail, Error> {
    if !available(session) {
        return Err(Error::Unsupported {
            op: "thumbnail",
            reason: "this unit and holder do not offer thumbnail scanning".into(),
        });
    }

    let window = window(session)?;
    session.set_window(&window)?;

    let started = session.scan(std::slice::from_ref(&window))?;
    session.test_unit_ready(THUMBNAIL_TIMEOUT)?;

    let mut data = vec![0u8; started.layout.total_bytes() as usize];
    let got = session.read_image(&started.layout, &mut data)?;
    data.truncate(got);
    debug!(bytes = got, "thumbnail");

    Ok(Thumbnail {
        layout: started.layout,
        cooperation: started.cooperation,
        data,
    })
}

/// One window over everything the holder can reach
fn window(session: &Session) -> Result<Window, Error> {
    let caps = session.capabilities();
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
    let interleaving = match offered.contains(ColorInterleaving::LINE_WITHOUT_DISTANCE) {
        true => ColorInterleaving::LINE_WITHOUT_DISTANCE,
        false => {
            return Err(unsupported(format!(
                "a thumbnail needs line ordering and this unit offers {offered:?}"
            )));
        }
    };

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
    w.color_interleaving = interleaving;
    // 2-10 byte 45: the default, and what the unit reports back for a 0
    w.ae_value = 255;
    Ok(w)
}

/// Whether the host owes the unit a thumbnail it has to build itself
///
/// `Features` byte 4 bit 0. Set on both families, so this is the normal case rather
/// than an exception
pub fn host_builds(session: &Session) -> bool {
    session
        .capabilities()
        .features
        .cooperation
        .contains(HostCooperation::THUMBNAIL)
}
