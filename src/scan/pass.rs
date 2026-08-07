//! Taking one scan pass over the film
//!
//! Every kind of pass, thumbnail, prescan and scan alike, is the same four commands.

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

/// A finished scan pass
///
/// The samples are the caller's buffer, so this struct carries only what describes them
#[derive(Debug, Clone)]
pub struct Pass {
    /// The stream's shape, as far as 2-10's formula describes it
    pub layout: Layout,
    /// What the unit asked the host to do with the data, if anything
    pub cooperation: Option<CooperativeAction>,
    /// Whether every block the layout promised arrived
    pub complete: bool,
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

/// [`start`] the pass and unscramble it into `samples` as it arrives
///
/// Every pass goes through this, thumbnail and prescan and scan alike: what
/// they need is in the layout, so none of them is a special case. The stream is
/// decoded a chunk at a time and never held whole, which at full resolution is
/// half a gigabyte that never gets allocated.
///
/// `samples` is resized to what the layout describes and is the caller's, so it
/// can be the buffer an image ends up in rather than one more copy.
pub fn take(
    session: &mut Session,
    windows: &[Window],
    timeout: Duration,
    curves: Option<&Curves>,
    samples: &mut Vec<u16>,
) -> Result<Pass, Error> {
    let started = start(session, windows, timeout)?;
    let mut decoder = decoder(&started.layout, curves)?;
    samples.clear();
    samples.resize(decoder.samples(), 0);

    let mut chunks = session.image_chunks(&started.layout)?;
    while let Some(chunk) = chunks.next() {
        decoder.push(chunk?, samples)?;
    }
    debug!(
        blocks = decoder.decoded(),
        complete = decoder.complete(),
        "pass"
    );

    Ok(Pass {
        layout: started.layout,
        cooperation: started.cooperation,
        complete: decoder.complete(),
    })
}
