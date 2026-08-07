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

/// Build a decoder for a scan pass
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

/// Stage the windows and start a scan pass, returning once the data is ready
///
/// The caller owes the unit a read: a scan whose data is never read locks out every command that follows.
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
