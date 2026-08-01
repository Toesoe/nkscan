//! Scanners supported by this library

use crate::{
    decode::StreamDecoder,
    scsi::{self, TransportExt},
};
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

pub mod ls40;
pub mod ls4000;
pub mod ls50;
pub mod ls5000;
pub mod ls8000;
pub mod ls9000;
pub mod nikon;

/// A window into the scanner's field of view
///
/// In the measurement units the driver set at open, so the pitch is per scanner.
/// Constructors live with each driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanArea {
    /// Offset along the sensor bar
    pub x_pos: u32,
    /// Offset along the feed, which selects the frame
    pub y_pos: u32,
    /// Extent along the sensor bar
    pub x_size: u32,
    /// Extent along the feed. Some scanners only take whole interleave blocks here.
    pub y_size: u32,
}

/// Either half of a streamed read can fail: the transport, or the decoder consuming it
#[derive(Debug, thiserror::Error)]
pub enum ReadError<E> {
    #[error(transparent)]
    Scsi(#[from] scsi::Error),
    #[error(transparent)]
    Decode(E),
    /// A progress report asked for the pass to stop
    #[error("the pass was cancelled")]
    Cancelled,
}

/// What a progress report asks the pass to do next
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Continue,
    /// Stop reading. Whoever knows how to throw the rest of the pass away is the one that does
    /// it, since the command to do so is per model.
    Cancel,
}

/// Reports (bytes read, bytes expected) of a pass and says whether to keep going
///
/// Boxed rather than generic so a caller reached through a trait object can still be given one.
pub type ProgressFn<'a> = dyn FnMut(u64, u64) -> Flow + 'a;

/// Unit attentions queue up, but not without bound. Past this something is wrong.
const MAX_QUEUED_UNIT_ATTENTIONS: usize = 8;

/// How a scanner reports readiness
///
/// Which sense codes mean what is per model, so only the shape is shared. The states that
/// clear by being read have to be distinguishable from the ones that do not, or a driver
/// cannot tell a queue of stale attentions from a scanner that is genuinely stuck.
pub trait ScannerStatus: Sized {
    /// What a scanner with nothing to report is in
    fn ready() -> Self;

    /// The state this sense describes, or `None` if it is a real error rather than a state
    fn from_sense(sense: &scsi::SenseData) -> Option<Self>;

    /// Whether reading this state is what clears it
    fn is_unit_attention(&self) -> bool;

    /// Whether the unit is still bringing itself up, so waiting is what clears it
    fn is_initializing(&self) -> bool;
}

/// Readiness out of TEST UNIT READY, with transient not-ready folded in as an ok state
///
/// Free-standing so opening a handle can ask before there is a scanner to ask.
pub fn status_of<S, T>(transport: &mut T) -> Result<S, scsi::Error>
where
    S: ScannerStatus,
    T: scsi::Transport + ?Sized,
{
    match transport.send(&scsi::cdbs::TestUnitReady::new()) {
        Ok(()) => Ok(S::ready()),
        Err(err) => {
            if let scsi::Error::Status {
                sense: Some(sense), ..
            } = &err
                && let Some(state) = S::from_sense(sense)
            {
                return Ok(state);
            }
            Err(err)
        }
    }
}

/// [`status_of`], clearing any queued unit attentions first
///
/// A device reports one per command and clears it as it goes, so a single status only ever
/// sees the oldest. Anything that cannot tolerate a stray CHECK CONDITION needs this instead.
pub fn drain_unit_attentions<S, T>(transport: &mut T) -> Result<S, scsi::Error>
where
    S: ScannerStatus + std::fmt::Debug,
    T: scsi::Transport + ?Sized,
{
    for _ in 0..MAX_QUEUED_UNIT_ATTENTIONS {
        let status = status_of::<S, T>(transport)?;
        if !status.is_unit_attention() {
            return Ok(status);
        }
        tracing::debug!(?status, "Cleared a unit attention");
    }
    Err(scsi::Error::InvalidResponse(
        "scanner kept reporting unit attentions",
    ))
}

/// Spin until the unit has configured itself, which a cold power-on has not
///
/// Everything is refused until then, INQUIRY included, so this runs before a handle exists and
/// before anything reads geometry. Not a unit attention: reporting it does not clear it, only
/// waiting does.
pub fn wait_while_initializing<S, T>(
    transport: &mut T,
    timeout: Duration,
    poll: Duration,
) -> Result<S, scsi::Error>
where
    S: ScannerStatus + std::fmt::Debug,
    T: scsi::Transport + ?Sized,
{
    let deadline = Instant::now() + timeout;
    loop {
        let status = status_of::<S, T>(transport)?;
        if !status.is_initializing() {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            return Err(scsi::Error::InvalidResponse(
                "scanner never finished configuring itself",
            ));
        }
        tracing::debug!(?status, "Waiting for the scanner to come up");
        sleep(poll);
    }
}

/// What every scanner can do, whatever it is on the other end of the transport
pub trait Scanner {
    /// How this scanner reports readiness
    type Status: ScannerStatus + std::fmt::Debug;

    /// What it is on the other end
    type Transport: scsi::Transport;

    /// The transport this scanner drives
    ///
    /// The extension point the defaults below are built on. Handing out the transport is no
    /// more access than [`vpd_page`](Self::vpd_page) already gives.
    fn transport(&mut self) -> &mut Self::Transport;

    /// Vendor, product and revision
    fn identify(&mut self) -> Result<scsi::cdbs::InquiryResponse, scsi::Error> {
        self.transport().send(&scsi::cdbs::Inquiry::new())
    }

    /// Current readiness, with transient not-ready states folded in rather than raised
    fn status(&mut self) -> Result<Self::Status, scsi::Error> {
        status_of(self.transport())
    }

    /// See [`drain_unit_attentions`]
    fn drain_unit_attentions(&mut self) -> Result<Self::Status, scsi::Error> {
        drain_unit_attentions(self.transport())
    }

    /// Take exclusive access
    fn reserve(&mut self) -> Result<(), scsi::Error> {
        self.transport().send(&scsi::cdbs::ReserveUnit::default())
    }

    /// Give exclusive access back
    fn release(&mut self) -> Result<(), scsi::Error> {
        self.transport().send(&scsi::cdbs::ReleaseUnit::default())
    }

    /// Read a vital product data page. Page 0x00 lists the ones a device has.
    fn vpd_page(&mut self, page_code: u8) -> Result<Vec<u8>, scsi::Error> {
        Ok(self
            .transport()
            .send(&scsi::cdbs::VpdInquiry::new(page_code, 0xFF))?
            .data)
    }

    /// Pull the next slice of the pending pass. An empty return means the scanner stopped early.
    fn read_chunk(&mut self, want: u32) -> Result<Vec<u8>, scsi::Error>;

    /// Stream a pass into a decoder, `chunk` bytes at a time
    ///
    /// The decoder says how much to read, so the geometry lives in one place
    fn read_into<D>(&mut self, decoder: &mut D, chunk: u32) -> Result<(), ReadError<D::Error>>
    where
        D: StreamDecoder,
        Self: Sized,
    {
        self.read_into_with(decoder, chunk, |_, _| Flow::Continue)
    }

    /// [`read_into`](Self::read_into), calling `progress` with (received, expected) per chunk
    ///
    /// A [`Flow::Cancel`] stops the read and returns [`ReadError::Cancelled`] with the rest of
    /// the pass still pending on the device, which the caller has to clear.
    fn read_into_with<D, F>(
        &mut self,
        decoder: &mut D,
        chunk: u32,
        mut progress: F,
    ) -> Result<(), ReadError<D::Error>>
    where
        D: StreamDecoder,
        F: FnMut(u64, u64) -> Flow,
        Self: Sized,
    {
        let expected = decoder.expected_bytes();
        let mut received = 0u64;

        while received < expected {
            let want = u64::from(chunk).min(expected - received) as u32;
            let bytes = self.read_chunk(want)?;
            if bytes.is_empty() {
                return Err(scsi::Error::InvalidResponse(
                    "image read returned nothing before the expected length",
                )
                .into());
            }
            received += bytes.len() as u64;
            decoder.push(&bytes).map_err(ReadError::Decode)?;
            if progress(received, expected) == Flow::Cancel {
                return Err(ReadError::Cancelled);
            }
        }
        Ok(())
    }
}

/// A scanner with removable film holders
pub trait FilmHolder: Scanner {
    /// What holders this scanner recognizes
    type Holder: scsi::cdbs::VendorPage;

    /// Which holder, if any, is currently loaded
    ///
    /// A VPD inquiry, which SPC exempts from being blocked by a pending unit attention, so
    /// this keeps answering across the holder change that loading one raises.
    fn holder(&mut self) -> Result<Self::Holder, scsi::Error> {
        self.transport().vpd()
    }
}

/// A scanner with a movable focus mechanism
pub trait Focus {
    /// The focus value currently staged in firmware
    fn focus(&mut self) -> Result<u16, scsi::Error>;

    /// Stage a focus target and commit it
    fn set_focus(&mut self, focus: u16) -> Result<(), scsi::Error>;
}
