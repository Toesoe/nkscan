//! An open, exclusive hold of a scanner
//!
//! Ties [`transport`](crate::transport) to [`protocol`](crate::protocol): holds
//! the state that outlives a single command, wraps each CDB in the invariants
//! its section imposes, and absorbs the retry and polling semantics. Deciding
//! what a scan should do belongs above this.

mod data;
mod focus;
mod image;
mod probe;
mod window;

pub use image::Chunks;
pub use probe::{inquiry, probe, vpd};

use crate::{
    error::Error,
    protocol::{
        caps::Capabilities,
        cdbs::{
            Abort, ModeSelect, ModeSense, PageControl, ReleaseUnit, ReserveUnit, TestUnitReady,
        },
        mode,
        sense::{Activity, Change, Coop, Fault, Outcome, Refusal, interpret},
    },
    transport::{self, Completion, Data, Transport},
};
use std::{
    io,
    thread::sleep,
    time::{Duration, Instant},
};
use tracing::*;

pub struct Session {
    caps: Capabilities,
    transport: Box<dyn Transport>,
    /// What a step of a window coordinate is currently worth
    divisor: u16,
    /// Whether we hold the unit, so [`Drop`] only releases what it took
    reserved: bool,
}

pub(crate) const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// How long to wait before asking a busy unit again
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Long enough for a full-length stage move
pub(crate) const MOVE_TIMEOUT: Duration = Duration::from_secs(180);

/// Long enough for a cold unit to warm its lamp and initialize
///
/// Nothing advertised bounds this: `Address` bytes 80,81 are the lamp warm-up
/// maximum, and both specs give them as 0
const READY_TIMEOUT: Duration = Duration::from_secs(180);

/// A device raising unit attentions forever would spin on refresh, so cap those.
/// Polling needs no cap: it sleeps, and the deadline already bounds it
const MAX_CHANGES: usize = 16;

/// A reply we could not make sense of
pub(crate) fn malformed(what: String) -> Error {
    Error::Transport(io::Error::new(io::ErrorKind::InvalidData, what).into())
}

impl Session {
    /// Start a new scanning session
    ///
    /// Pins the measurement unit divisor to the unit's maximum resolution, so
    /// every window coordinate is one pixel and agrees with the addresses and
    /// boundaries `Address` reports. A hard reset or a power cycle puts it back to
    /// 1200, so a session that outlives one has to open again
    pub fn open(mut transport: Box<dyn Transport>) -> Result<Self, Error> {
        let caps = probe(transport.as_mut())?;
        let divisor = caps.address.x_axis.dpi_range.last;
        let mut session = Self {
            transport,
            caps,
            divisor,
            reserved: false,
        };
        // INQUIRY answers while the unit is still initializing, so probing says
        // nothing about readiness. Everything below is a real command, and a
        // cold unit would spend the whole of the first one's budget not ready
        session.test_unit_ready(READY_TIMEOUT)?;
        // Hold the unit before touching anything that changes its state
        session.reserved = session.reserve()?;

        // A scan left over from whoever had it last refuses every non-basic
        // command with 05h-2Ch, so ask with one and stop it only if it has
        if let Err(Error::Device(fault)) = session.windows() {
            if matches!(*fault, Fault::Rejected(Refusal::OutOfSequence, _)) {
                debug!("a scan was still valid from earlier, stopping it");
                session.abort()?;
            } else {
                return Err(Error::Device(fault));
            }
        }

        session.set_units(divisor)?;
        Ok(session)
    }

    /// Take the unit, so no other initiator can interleave with us
    ///
    /// Only SBP-2 has more than one initiator, but the 5000 documents the
    /// command too, in 2-4 and not in its own command list, so a unit that has
    /// never heard of it is not an error. Answers whether we got it
    fn reserve(&mut self) -> Result<bool, Error> {
        match self.run(&ReserveUnit.cdb(), Data::None, PROBE_TIMEOUT) {
            Ok(_) => Ok(true),
            Err(Error::Device(fault))
                if matches!(*fault, Fault::Rejected(Refusal::UnknownOpcode, _)) =>
            {
                debug!("this unit has no RESERVE UNIT");
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }

    /// Stop any scan in progress
    ///
    /// 2-13: the scan block stops where it is, and a scan has to be issued again
    /// to read anything. GOOD comes back even when nothing was running, so this
    /// is also how to get to a known state.
    ///
    /// Worth doing at open. A scan whose data was never read stays valid, and
    /// while it is, every non-basic command is refused with `05h-2Ch` -- so one
    /// program exiting early locks out the next one
    pub fn abort(&mut self) -> Result<(), Error> {
        match self.run(&Abort.cdb(), Data::None, PROBE_TIMEOUT) {
            Ok(_) => {}
            Err(Error::Device(fault))
                if matches!(*fault, Fault::Rejected(Refusal::UnknownOpcode, _)) =>
            {
                debug!("this unit has no ABORT");
                return Ok(());
            }
            Err(e) => return Err(e),
        }
        // An operation activation command, so it answers before it acts
        self.test_unit_ready(MOVE_TIMEOUT)
    }

    /// What the scanner says it can do
    pub fn capabilities(&self) -> &Capabilities {
        &self.caps
    }

    /// The measurement unit divisor in force, as [`set_units`](Self::set_units) left it
    pub fn divisor(&self) -> u16 {
        self.divisor
    }

    /// Re-read what the scanner says it can do
    ///
    /// Needed after anything that changes the adapter or holder, since several
    /// fields track those rather than the model
    pub fn refresh(&mut self) -> Result<(), Error> {
        self.caps = probe(self.transport.as_mut())?;
        Ok(())
    }

    /// Simple readiness check
    pub fn test_unit_ready(&mut self, timeout: Duration) -> Result<(), Error> {
        self.run(&TestUnitReady.cdb(), Data::None, timeout)?;
        Ok(())
    }

    /// One mode page, header and block descriptor included
    pub fn mode_sense(&mut self, page: u8, control: PageControl) -> Result<Vec<u8>, Error> {
        let cmd = ModeSense::new(page, control);
        let mut buf = vec![0u8; cmd.allocation_length()];
        let completion = self.run(&cmd.cdb(), Data::In(&mut buf), PROBE_TIMEOUT)?;
        buf.truncate(completion.transferred);
        Ok(buf)
    }

    /// Read the divisor back off the unit
    ///
    /// It outlives a session -- it holds until the next MODE SELECT, a reset or
    /// a power cycle -- so it is worth checking rather than assuming
    pub fn units(&mut self) -> Result<u16, Error> {
        let reply = self.mode_sense(mode::MEASUREMENT_UNITS, PageControl::Current)?;
        mode::divisor(&reply)
            .ok_or_else(|| malformed(format!("no measurement units page in {reply:02x?}")))
    }

    /// Count window coordinates in steps of an inch divided by `divisor`
    ///
    /// 2-3-4 note 5 takes only 1200 or the unit's own maximum resolution and
    /// answers anything else with common error 2
    pub fn set_units(&mut self, divisor: u16) -> Result<(), Error> {
        let max = self.caps.address.x_axis.dpi_range.last;
        if divisor != 1200 && divisor != max {
            return Err(Error::Unsupported {
                op: "measurement units",
                reason: format!("the divisor must be 1200 or {max}, not {divisor}"),
            });
        }

        let list = mode::set_divisor(divisor);
        let cmd = ModeSelect::new(list.len() as u8);
        self.run(&cmd.cdb(), Data::Out(&list), PROBE_TIMEOUT)?;
        self.divisor = divisor;
        Ok(())
    }

    /// Issue a command, absorbing everything that means "not done yet", and hand
    /// back the completion once it has actually terminated
    ///
    /// `timeout` budgets the whole command including re-issues, not one transfer
    pub fn run(
        &mut self,
        cdb: &[u8],
        data: Data<'_>,
        timeout: Duration,
    ) -> Result<Completion, Error> {
        let (completion, coop) = self.run_cooperative(cdb, data, timeout)?;
        match coop {
            None => Ok(completion),
            Some(coop) => Err(Error::Unsupported {
                op: "host cooperation",
                reason: format!("{coop:?} is not implemented yet"),
            }),
        }
    }

    /// As [`run`](Self::run), but hands a cooperative request back rather than
    /// refusing it
    ///
    /// Only SCAN and READ raise one. 2-7: read the parameter with
    /// `DataType::Cooperation`, do the
    /// work, and issue the command again
    pub fn run_cooperative(
        &mut self,
        cdb: &[u8],
        mut data: Data<'_>,
        timeout: Duration,
    ) -> Result<(Completion, Option<Coop>), Error> {
        let deadline = Instant::now() + timeout;
        let mut changes = 0usize;
        // Polling produces the same outcome over and over, so log transitions
        let mut reported: Option<Activity> = None;

        loop {
            // `Data::In` holds a `&mut [u8]` and so is not `Copy`. Reborrowing
            // it here is what lets the same command go out more than once
            let payload = match &mut data {
                Data::None => Data::None,
                Data::In(buf) => Data::In(buf),
                Data::Out(buf) => Data::Out(buf),
            };

            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err(Error::Transport(transport::Error::Timeout(timeout)));
            }

            let completion = self.transport.execute(cdb, payload, left)?;

            match interpret(&completion) {
                Outcome::Complete => return Ok((completion, None)),
                Outcome::CompleteWith(adjustment) => {
                    // Sense key 01h, so the command finished. Worth saying out
                    // loud: it means the unit did something other than what we
                    // asked, and GET WINDOW is what reports the result
                    info!(?adjustment, "the scanner had a note about that");
                    return Ok((completion, None));
                }

                // Not yet. Polling is re-issuing
                Outcome::Working(activity) => {
                    if reported != Some(activity) {
                        debug!(?activity, "waiting");
                        reported = Some(activity);
                    }
                    sleep(POLL_INTERVAL);
                }

                // Unit attention: the command did not run. Several can be
                // queued -- ejecting a holder raises a holder change and a
                // reset -- so this arm firing repeatedly is normal
                Outcome::StateChanged(change) => {
                    debug!(?change, "device state changed under us, re-issuing");
                    // Several fields track the adapter and holder rather than
                    // the model, so what we cached may no longer describe the
                    // unit we are about to re-issue against
                    self.refresh()?;
                    changes += 1;
                    if changes >= MAX_CHANGES {
                        return Err(unsettled(change, changes));
                    }
                }

                // The scanner wants post-processing before it will go on
                Outcome::NeedsHost(coop) => return Ok((completion, Some(coop))),

                terminal => return Err(Error::from_outcome(terminal, &completion)),
            }
        }
    }
}

/// A unit that keeps raising attentions is not faulted, it is unusable
fn unsettled(change: Change, changes: usize) -> Error {
    warn!(
        ?change,
        changes, "giving up on a device that will not settle"
    );
    Error::Unsupported {
        op: "command",
        reason: format!(
            "the unit raised {changes} unit attentions without running it, last {change:?}"
        ),
    }
}

/// A reservation only clears on RELEASE, a reset or a power cycle, so one we
/// drop on the floor locks the unit out of every other program until then
impl Drop for Session {
    fn drop(&mut self) {
        if !self.reserved {
            return;
        }
        if let Err(e) = self.run(&ReleaseUnit.cdb(), Data::None, PROBE_TIMEOUT) {
            warn!(%e, "could not release the scanner");
        }
    }
}
