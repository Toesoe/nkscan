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
use std::{
    collections::VecDeque,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};
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
    /// Image rows and columns: the sensor (the layout's pixels) and the feed
    /// (its lines)
    pub rows: usize,
    pub cols: usize,
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

/// How many raw chunks the reader and decoder have in flight between them
const POOL: usize = 3;

/// A chunk handed from the reader thread to the decoder, or how the stream came to an end
enum Chunk {
    /// A whole chunk of the stream, to be decoded and then handed back
    Data(Vec<u8>),
    /// The stream ran out
    End,
    /// The stream faulted part-way
    Failed(Error),
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
    let layout = started.layout.clone();
    let mut decoder = decoder(&layout, curves)?;
    samples.clear();
    samples.resize(decoder.samples(), 0);

    thread::scope(|scope| {
        // One thread pulls whole chunks off the unit, this one unscrambles
        // them. A filled buffer goes down `full`, and the decoder hands the
        // empty one back down `empty`, so the same `POOL` buffers circulate
        // for the whole pass without a copy.
        let (full_tx, full_rx) = mpsc::channel::<Chunk>();
        let (empty_tx, empty_rx) = mpsc::channel::<Vec<u8>>();
        scope.spawn(move || read_chunks(session, &layout, &full_tx, &empty_rx));

        let mut out = Ok(());
        for msg in full_rx {
            let chunk = match msg {
                Chunk::Data(buf) => buf,
                Chunk::End => break,
                Chunk::Failed(e) => {
                    out = Err(e);
                    break;
                }
            };
            let decoded = decoder.push(&chunk, samples);
            // The buffer is spent either way, so back to the pool it goes
            let _ = empty_tx.send(chunk);
            if let Err(e) = decoded {
                out = Err(e);
                break;
            }
        }
        out
    })?;

    debug!(
        blocks = decoder.decoded(),
        complete = decoder.complete(),
        "pass"
    );

    let (rows, cols) = decoder.shape();
    Ok(Pass {
        layout: started.layout,
        cooperation: started.cooperation,
        complete: decoder.complete(),
        rows,
        cols,
    })
}

/// Read the whole stream off the unit a chunk at a time, forwarding each chunk
/// down `full` and drawing the buffer to fill from the pool `empty` keeps up
fn read_chunks(
    session: &mut Session,
    layout: &Layout,
    full: &Sender<Chunk>,
    empty: &Receiver<Vec<u8>>,
) {
    let mut chunks = match session.image_chunks(layout) {
        Ok(chunks) => chunks,
        Err(e) => {
            let _ = full.send(Chunk::Failed(e));
            let _ = full.send(Chunk::End);
            return;
        }
    };

    // The pool: `POOL` buffers handed back and forth by ownership, never copied
    let mut pool: VecDeque<Vec<u8>> = (0..POOL).map(|_| vec![0u8; chunks.capacity()]).collect();

    loop {
        // Reuse a buffer the decoder handed back, or a fresh one off the pool
        let mut buf = match pool.pop_front() {
            Some(buf) => buf,
            None => match empty.recv() {
                Ok(buf) => buf,
                // The decoder is gone, so the pass is over
                Err(_) => return,
            },
        };
        match chunks.fill(&mut buf) {
            Some(Ok(_)) => {}
            Some(Err(e)) => {
                let _ = full.send(Chunk::Failed(e));
                let _ = full.send(Chunk::End);
                return;
            }
            None => {
                let _ = full.send(Chunk::End);
                return;
            }
        }
        if full.send(Chunk::Data(buf)).is_err() {
            return;
        }
    }
}
