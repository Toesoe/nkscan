//! Taking one pass over the film
//!
//! Every kind of pass, thumbnail, prescan and scan alike, is the same four
//! commands. Only the descriptors and the time budget differ.

use crate::{
    error::Error,
    protocol::{
        caps::set_window::ColorInterleaving, curves::Curves, data::CooperativeAction,
        decode::Decoder, image::Layout, window::Window,
    },
    session::{Session, window::Started},
};
use std::time::Duration;
use tracing::*;

/// A finished pass and the bytes it produced
#[derive(Debug)]
pub struct Pass {
    /// The stream's shape, as far as 2-10's formula describes it
    pub layout: Layout,
    /// What the unit asked the host to do with the data, if anything
    pub cooperation: Option<CooperativeAction>,
    /// The bytes, however many arrived
    pub data: Vec<u8>,
}

/// A decoder for a pass off this unit
///
/// Correcting the CCD's rows against each other wherever they were read at
/// once, since a three-line pass has the mismatch whether or not anyone asked
/// about it. `curves` comes from
/// [`ccd_curves`](Session::ccd_curves), and `None` there, from a unit with no
/// curves or a reply that does not match its page, decodes uncorrected.
///
/// Borrowed rather than owned so that a caller taking pass after pass builds
/// the tables once.
pub fn decoder<'a>(layout: &Layout, curves: Option<&'a Curves>) -> Result<Decoder<'a>, Error> {
    let decoder = Decoder::new(layout)?;
    match curves.filter(|_| {
        layout
            .interleaving
            .contains(ColorInterleaving::MULTILINE_SIMULTANEOUS)
    }) {
        Some(curves) => Ok(decoder.correcting(curves)),
        None => Ok(decoder),
    }
}

/// Stage the windows and start a pass, returning once the data is ready
///
/// The caller owes the unit a read: a scan whose data is never read locks out
/// every command that follows.
pub fn start(
    session: &mut Session,
    windows: &[Window],
    timeout: Duration,
) -> Result<Started, Error> {
    for w in windows {
        session.set_window(w)?;
    }
    let started = session.scan(windows)?;
    session.test_unit_ready(timeout)?;
    Ok(started)
}

/// [`start`] the pass and read it out
pub fn take(session: &mut Session, windows: &[Window], timeout: Duration) -> Result<Pass, Error> {
    let started = start(session, windows, timeout)?;

    let mut data = vec![0u8; started.layout.total_bytes() as usize];
    let got = session.read_image(&started.layout, &mut data)?;
    data.truncate(got);
    debug!(bytes = got, expected = started.layout.total_bytes(), "pass");

    Ok(Pass {
        layout: started.layout,
        cooperation: started.cooperation,
        data,
    })
}
