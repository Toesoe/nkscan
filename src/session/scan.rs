//! Taking scan passes: thumbnail, prescan, and full resolution
//!
//! Every kind of pass is the same sequence: stage the windows, start the scan,
//! read the stream a chunk at a time while a decoder unscrambles it.

use super::Session;
use crate::{
    error::Error,
    protocol::{image::Layout, window::Window},
    scan::pass::{self, Pass, Progress},
    session::window::Started,
};
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::{Duration, Instant},
};
use tracing::*;

/// How many raw chunks the reader and decoder have in flight between them
const POOL: usize = 3;

/// A chunk handed from the reader thread to the decoder, or how the stream ended
enum Chunk {
    Data(Vec<u8>),
    End,
    Failed(Error),
}

impl Session {
    /// Stage the windows and start a scan pass, returning once the data is ready
    ///
    /// `timeout` bounds the wait for the unit to report ready after SCAN, and
    /// nothing else. Each read of the data that follows carries its own
    /// [`MOVE_TIMEOUT`](super::MOVE_TIMEOUT), so a long pass is bounded a chunk
    /// at a time rather than as a whole.
    ///
    /// The caller owes the unit a read: a scan whose data is never read locks
    /// out every command that follows
    pub fn start_pass(&mut self, windows: &[Window], timeout: Duration) -> Result<Started, Error> {
        for w in windows {
            self.set_window(w)?;
        }
        let started = self.scan(windows)?;
        // Whether the unit reports ready as soon as it is streaming or only
        // once the whole pass is taken decides what this budget has to cover
        let waited = Instant::now();
        self.test_unit_ready(timeout)?;
        debug!(ready_in = ?waited.elapsed(), "scan ready");
        Ok(started)
    }

    /// Start a pass and unscramble it into `samples` as it arrives
    ///
    /// `samples` is cleared and resized; the caller owns it
    pub fn scan_pass(
        &mut self,
        windows: &[Window],
        timeout: Duration,
        samples: &mut Vec<u16>,
    ) -> Result<Pass, Error> {
        self.scan_pass_with(windows, timeout, samples, |_| {})
    }

    /// The same, telling `on` how far along the pass is after every chunk
    ///
    /// `on` runs on the decoding thread between chunks, so anything slow in it
    /// is time the unit spends waiting for the next read with its buffer filling
    pub fn scan_pass_with(
        &mut self,
        windows: &[Window],
        timeout: Duration,
        samples: &mut Vec<u16>,
        mut on: impl FnMut(Progress),
    ) -> Result<Pass, Error> {
        let started = self.start_pass(windows, timeout)?;
        let layout = started.layout.clone();
        let total = layout.total_bytes();
        let curves = self.curves();
        let mut decoder = pass::decoder(&layout, curves.as_deref())?;
        samples.clear();
        samples.resize(decoder.samples(), 0);

        let timing = Timing::default();
        let mut decoding = Duration::ZERO;
        let mut idle = Duration::ZERO;

        thread::scope(|scope| {
            let (full_tx, full_rx) = mpsc::channel::<Chunk>();
            let (empty_tx, empty_rx) = mpsc::channel::<Vec<u8>>();
            let timing = &timing;
            scope.spawn(move || read_chunks(self, &layout, &full_tx, &empty_rx, timing));

            let mut out = Ok(());
            let mut bytes = 0u64;
            loop {
                let waited = Instant::now();
                let msg = full_rx.recv();
                idle += waited.elapsed();

                let chunk = match msg {
                    Ok(Chunk::Data(buf)) => buf,
                    Ok(Chunk::End) | Err(_) => break,
                    Ok(Chunk::Failed(e)) => {
                        out = Err(e);
                        break;
                    }
                };

                let pushed = Instant::now();
                let decoded = decoder.push(&chunk, samples);
                decoding += pushed.elapsed();

                bytes += chunk.len() as u64;
                let _ = empty_tx.send(chunk);
                if let Err(e) = decoded {
                    out = Err(e);
                    break;
                }
                on(Progress {
                    bytes,
                    total,
                    blocks: decoder.decoded(),
                });
            }
            out
        })?;

        // `starved` is the only one of these the unit can feel: it is time we
        // spent not asking for data, with its buffer filling behind the stage
        debug!(
            blocks = decoder.decoded(),
            complete = decoder.complete(),
            chunks = Timing::get(&timing.chunks),
            bytes = Timing::get(&timing.bytes),
            read_ms = Timing::get(&timing.read) / 1_000_000,
            starved_ms = Timing::get(&timing.starved) / 1_000_000,
            decode_ms = decoding.as_millis(),
            idle_ms = idle.as_millis(),
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

    /// Scan everything loaded at the lowest resolution
    ///
    /// Builds its own windows from the capabilities (whole strip, lowest dpi,
    /// one channel per color), seeds white balance, and takes the pass
    pub fn scan_thumbnail(&mut self, samples: &mut Vec<u16>) -> Result<Pass, Error> {
        self.scan_thumbnail_with(samples, |_| {})
    }

    /// The same, telling `on` how far along the pass is after every chunk
    pub fn scan_thumbnail_with(
        &mut self,
        samples: &mut Vec<u16>,
        on: impl FnMut(Progress),
    ) -> Result<Pass, Error> {
        if !crate::scan::thumbnail::available(self.capabilities()) {
            return Err(Error::Unsupported {
                op: "thumbnail",
                reason: "this unit and adapter do not offer thumbnail scanning".into(),
            });
        }

        let windows = crate::scan::thumbnail::windows(self.capabilities())?;
        let windows = self.seed_white_balance(&windows)?;
        self.scan_pass_with(&windows, THUMBNAIL_TIMEOUT, samples, on)
    }
}

/// Long enough for a whole-strip pass at thumbnail resolution
const THUMBNAIL_TIMEOUT: Duration = Duration::from_secs(600);

/// Where a pass spent its time, so a unit that pauses can be told from a
/// decoder that will not keep up
///
/// The unit streams while the stage runs and has only its own buffer to hold
/// what we have not taken yet. Nothing is read while the reader waits for a
/// buffer to come back, so `starved` is time we spend not asking for data, and
/// it is the only one of these that stalls the mechanism. `idle` is the other
/// way round: the decoder had nothing to do because the unit had nothing to give
#[derive(Default)]
struct Timing {
    /// In READ, which is the unit's own pace
    read: AtomicU64,
    /// Waiting for the decoder to give a buffer back
    starved: AtomicU64,
    chunks: AtomicU64,
    bytes: AtomicU64,
}

impl Timing {
    fn add(counter: &AtomicU64, by: u64) {
        counter.fetch_add(by, Ordering::Relaxed);
    }

    fn get(counter: &AtomicU64) -> u64 {
        counter.load(Ordering::Relaxed)
    }
}

/// Read the whole stream off the unit a chunk at a time, forwarding each chunk
/// down `full` and drawing the buffer to fill from the pool `empty` keeps up
fn read_chunks(
    session: &mut Session,
    layout: &Layout,
    full: &Sender<Chunk>,
    empty: &Receiver<Vec<u8>>,
    timing: &Timing,
) {
    let mut chunks = match session.image_chunks(layout) {
        Ok(chunks) => chunks,
        Err(e) => {
            let _ = full.send(Chunk::Failed(e));
            let _ = full.send(Chunk::End);
            return;
        }
    };

    let mut pool: VecDeque<Vec<u8>> = (0..POOL).map(|_| vec![0u8; chunks.capacity()]).collect();

    loop {
        let mut buf = match pool.pop_front() {
            Some(buf) => buf,
            None => {
                let waited = Instant::now();
                let buf = match empty.recv() {
                    Ok(buf) => buf,
                    Err(_) => return,
                };
                Timing::add(&timing.starved, waited.elapsed().as_nanos() as u64);
                buf
            }
        };

        let reading = Instant::now();
        let filled = chunks.fill(&mut buf);
        Timing::add(&timing.read, reading.elapsed().as_nanos() as u64);

        match filled {
            Some(Ok(got)) => {
                Timing::add(&timing.chunks, 1);
                Timing::add(&timing.bytes, got as u64);
            }
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
