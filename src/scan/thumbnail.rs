//! Scanning the whole holder at once
//!
//! A thumbnail is how a strip's frames get found. `C1h` byte 16 says whether the
//! unit publishes rectangles at all, and `C8h` says whether it knows where they
//! end: a masked holder does, a strip holder reports a length of zero until
//! something measures it.
//!
//! `E1h` puts thumbnail in the host cooperation bits on both families, so the
//! unit hands us the pass and expects us to make sense of it.

use crate::{
    error::Error,
    protocol::{
        caps::{
            address::CoordinateBase,
            other::HostCooperation,
            set_window::{ColorInterleaving, ScanKind, ScanMode},
        },
        data::CooperativeAction,
        image::Layout,
        window::{Composition, Window},
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

/// Whether the unit already knows where every frame ends
///
/// When it does there is nothing to measure and no reason to take a pass
pub fn frames_known(session: &Session) -> bool {
    let caps = session.capabilities();
    match caps.frames.as_ref() {
        Some(frames) => frames.measured(),
        // Without rectangles there is nothing to complete, and framing comes
        // from perforation counting instead
        None => !caps
            .address
            .coordinate_base
            .contains(CoordinateBase::FRAME_RECTS),
    }
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

    let window = window(session);
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
fn window(session: &Session) -> Window {
    let caps = session.capabilities();
    let dpi = caps.address.thumbnail_resolution.start;
    let (x, y) = (&caps.address.x_axis, &caps.address.y_axis);

    let mut w = Window::try_from(&[0u8; crate::protocol::window::LENGTH][..])
        .expect("a zeroed descriptor is the right length");
    w.id = crate::protocol::window::DEFAULT;
    w.resolution = (dpi, dpi);
    w.origin = (x.address_range.start, y.address_range.start);
    w.size = (x.boundary, y.address_range.last);
    w.bpp = 16;
    w.composition = Composition::MultilevelBW;
    w.scanning_kind = ScanKind::THUMBNAIL;
    w.scanning_mode = ScanMode::NORMAL_QUALITY;
    // Every thumbnail in the captures is line ordering, never the three-line mode
    w.color_interleaving = ColorInterleaving::LINE_WITHOUT_DISTANCE;
    w.ae_value = u8::MAX;
    w
}

/// Whether the host owes the unit a thumbnail it has to build itself
///
/// `E1h` byte 4 bit 0. Set on both families, so this is the normal case rather
/// than an exception
pub fn host_builds(session: &Session) -> bool {
    session
        .capabilities()
        .features
        .cooperation
        .contains(HostCooperation::THUMBNAIL)
}
