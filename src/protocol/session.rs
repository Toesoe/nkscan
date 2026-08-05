//! Context for a scanner "session"
//!
//! A session here is an open, exclusive hold of a scanner that
//! needs to retain some state across commands. For example, we
//! will wrap up the sensed capabilities so operations are branched
//! or validated against what the scanner says it actually supports.
//! Additionally, the session handles retry loops and sensible blocking.

use crate::{
    error::Error,
    protocol::{
        caps::{
            Capabilities, Page, address::Address, address::Transfer, ccd::CcdMeasurement,
            identity::Identity, other::Features, set_window::SetWindowFunction,
        },
        cdbs::{
            Abort, Execute, GetWindow, Inquiry, ModeSelect, ModeSense, PageControl, Read,
            ReleaseUnit, ReserveUnit, Scan, SetParameter, SetWindow, TestUnitReady,
        },
        data, mode,
        sense::{Activity, Coop, Fault, Outcome, Refusal, interpret},
        window::{self, GetWindowHeader, SetWindowHeader, Window},
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
    /// Whether we hold the unit, so [`Drop`] only releases what it took
    reserved: bool,
}

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// How long to wait before asking a busy unit again
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Long enough for a full-length stage move
const MOVE_TIMEOUT: Duration = Duration::from_secs(180);

/// A reply we could not make sense of
fn malformed(what: String) -> Error {
    Error::Transport(io::Error::new(io::ErrorKind::InvalidData, what).into())
}

/// A device raising unit attentions forever would spin on refresh, so cap those.
/// Polling needs no cap: it sleeps, and the deadline already bounds it
const MAX_CHANGES: usize = 16;

/// Run one INQUIRY and hand back however many bytes actually arrived
pub fn inquiry(t: &mut dyn Transport, cmd: Inquiry) -> Result<Vec<u8>, Error> {
    let mut buf = vec![0u8; cmd.allocation_length()];
    let completion = t.execute(&cmd.cdb(), Data::In(&mut buf), PROBE_TIMEOUT)?;
    match interpret(&completion) {
        Outcome::Complete | Outcome::CompleteWith(_) => {}
        other => return Err(Error::from_outcome(other, &completion)),
    }
    buf.truncate(completion.transferred);
    Ok(buf)
}

/// Read a VPD page from the scanner
pub fn vpd(t: &mut dyn Transport, code: u8) -> Result<Page, Error> {
    Ok(Page::new(code, inquiry(t, Inquiry::vpd(code))?)?)
}

/// Ask the scanner what it can do
/// Safe to call with a unit attention outstanding
/// 2-2 note 5 says INQUIRY is performed regardless, and does not clear it
pub fn probe(t: &mut dyn Transport) -> Result<Capabilities, Error> {
    let identity = Identity::parse(&inquiry(t, Inquiry::standard())?)?;
    // Opening the wrong node is easy, and everything below assumes a scanner
    if !identity.is_scanner() {
        return Err(Error::NotFound);
    }

    Ok(Capabilities {
        identity,
        address: Address::try_from(&vpd(t, Address::PAGE_CODE)?)?,
        features: Features::try_from(&vpd(t, Features::PAGE_CODE)?)?,
        set_window: SetWindowFunction::try_from(&vpd(t, SetWindowFunction::PAGE_CODE)?)?,
        // Neither spec lists this one in page 00h, so a refusal means the unit
        // has not got it rather than that anything went wrong
        ccd: match vpd(t, CcdMeasurement::PAGE_CODE) {
            Ok(page) => Some(CcdMeasurement::try_from(&page)?),
            Err(Error::Device(fault)) if matches!(*fault, Fault::Rejected(..)) => {
                debug!("this unit has no CCD measurement page");
                None
            }
            Err(e) => return Err(e),
        },
    })
}

impl Session {
    /// Start a new scanning session
    ///
    /// The measurement unit divisor is pinned to the unit's maximum resolution,
    /// so from here on every window coordinate is one pixel and agrees with the
    /// addresses and boundaries `C1h` reports. A hard reset or a power cycle puts
    /// it back to 1200, so a session that outlives one has to open again
    pub fn open(mut transport: Box<dyn Transport>) -> Result<Self, Error> {
        // To start up a session, we need to query the scanner for its capabilites and optionally request exclsuive access (SBP-2)
        let caps = probe(transport.as_mut())?;
        let mut session = Self {
            transport,
            caps,
            reserved: false,
        };
        // Hold the unit before touching anything that changes its state
        session.reserved = session.reserve()?;
        // A scan left over from whoever had it last would refuse everything below
        session.abort()?;
        let max = session.caps.address.x_axis.dpi_range.last;
        session.set_units(max)?;
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

    /// Get the current capabilities of the scanner
    pub fn capabilities(&self) -> &Capabilities {
        &self.caps
    }

    /// Re-read what the scanner says it can do
    ///
    /// Needed after anything that changes the adapter or holder, since several fields track those rather than the model
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

    /// What a step of a window coordinate is currently worth
    ///
    /// This outlives a session -- it holds until the next MODE SELECT, a reset
    /// or a power cycle -- so it has to be read rather than assumed
    pub fn units(&mut self) -> Result<u16, Error> {
        let reply = self.mode_sense(mode::MEASUREMENT_UNITS, PageControl::Current)?;
        mode::divisor(&reply)
            .ok_or_else(|| malformed(format!("no measurement units page in {reply:02x?}")))
    }

    /// Count window coordinates in steps of an inch divided by `divisor`
    ///
    /// 2-3-4 note 5 takes only 1200 or the unit's own maximum resolution and
    /// answers anything else with common error 2, so the rest is refused here
    /// with a reason attached
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
        Ok(())
    }

    /// READ one data type, in two passes so the data header can size the second
    ///
    /// Only for the codes from 80h up, which are the ones the data header
    /// precedes. Image data carries none and is read against the scan geometry
    pub fn read_data(
        &mut self,
        kind: data::DataType,
        color: u8,
    ) -> Result<(data::Header, data::Values), Error> {
        let row = kind.row();
        let code = row.code;
        let refuse = |reason| {
            Err(Error::Unsupported {
                op: "read data type",
                reason,
            })
        };

        if !row.header {
            return refuse(format!(
                "{code:02X}h carries no data header to size a read by"
            ));
        }
        match row.read {
            Some(bit) if self.caps.features.data_types.contains(bit) => {}
            _ => return refuse(format!("this unit does not offer {code:02X}h")),
        }
        let Some(width) = row.width else {
            return refuse(format!("{code:02X}h takes a width 2-11-2 does not fix"));
        };
        let qualifier = data::width_code(width).expect("2-11-2 widths are all encodable");
        let color = if kind.per_color() { color } else { 0 };

        let mut fetch = |len: u32| -> Result<Vec<u8>, Error> {
            let cmd = Read::new(code, color, qualifier, len);
            let mut buf = vec![0u8; cmd.allocation_length()];
            let completion = self.run(&cmd.cdb(), Data::In(&mut buf), PROBE_TIMEOUT)?;
            buf.truncate(completion.transferred);
            Ok(buf)
        };

        // The header reports what the unit holds whatever we asked for, so one
        // short read is enough to size the real one
        let probe = fetch(data::HEADER as u32)?;
        let (probe, _) = data::Header::from_bytes(&probe)
            .ok_or_else(|| malformed(format!("{code:02X}h header was {} bytes", probe.len())))?;

        let raw = fetch(data::HEADER as u32 + probe.length)?;
        let (header, payload) = data::Header::from_bytes(&raw)
            .ok_or_else(|| malformed(format!("{code:02X}h header was {} bytes", raw.len())))?;

        // Analog gain reports 16 bytes against a documented 8, and the tail is
        // stale, so the table wins wherever it fixes a count
        let valid = match row.count {
            Some(n) => payload
                .get(..n as usize * width as usize)
                .unwrap_or(payload),
            None => payload,
        };
        debug!(?header, bytes = valid.len(), "read data");
        Ok((header, data::Values::decode(kind.scalar(), valid)))
    }

    /// Set the operation parameter, activate the operation, and confirm its
    /// termination
    ///
    /// 2-14: EXECUTE performs the operation *after* returning GOOD status, and no
    /// command other than a basic command may be issued before the operation
    /// termination is confirmed by TEST UNIT READY. So all three are one call
    pub fn execute(
        &mut self,
        operation: u8,
        params: data::Operation,
        timeout: Duration,
    ) -> Result<(), Error> {
        if !self.caps.features.execute.supports(operation) {
            return Err(Error::Unsupported {
                op: "execute operation",
                reason: format!("this unit does not offer {operation:02X}h"),
            });
        }

        let block = params.to_bytes();
        let cmd = SetParameter::new(operation, block.len() as u32);
        self.run(&cmd.cdb(), Data::Out(&block), PROBE_TIMEOUT)?;

        debug!(
            operation = format!("{operation:02X}h"),
            ?params,
            "executing"
        );
        self.run(&Execute.cdb(), Data::None, PROBE_TIMEOUT)?;
        self.test_unit_ready(timeout)
    }

    /// One GET WINDOW, exactly as asked: header plus descriptors, unparsed
    fn get_window_raw(&mut self, cmd: GetWindow) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0u8; cmd.allocation_length()];
        let completion = self.run(&cmd.cdb(), Data::In(&mut buf), PROBE_TIMEOUT)?;
        buf.truncate(completion.transferred);
        Ok(buf)
    }

    /// Read back every window descriptor the unit currently holds
    ///
    /// Two passes: a transfer longer than what is there gets refused, so the
    /// header has to say how much there is first
    pub fn windows(&mut self) -> Result<Vec<Window>, Error> {
        let probe = self.get_window_raw(GetWindow::all(window::HEADER as u32))?;
        let (probe, _) =
            GetWindowHeader::from_bytes(&probe).map_err(|e| malformed(e.to_string()))?;

        let data = self.get_window_raw(GetWindow::all(2 + u32::from(probe.data_length)))?;
        let (header, descriptors) =
            GetWindowHeader::from_bytes(&data).map_err(|e| malformed(e.to_string()))?;
        let stride = usize::from(header.descriptor_length);
        debug!(stride, bytes = descriptors.len(), "window descriptors");

        if stride < window::LENGTH {
            return Err(malformed(format!(
                "descriptor stride of {stride} is shorter than the {} bytes 2-10-3 defines",
                window::LENGTH
            )));
        }

        descriptors
            .chunks_exact(stride)
            .map(|d| Window::try_from(d).map_err(|e| malformed(e.to_string())))
            .collect()
    }

    /// Define one window
    pub fn set_window(&mut self, window: &Window) -> Result<(), Error> {
        window.validate(&self.caps)?;

        let header = SetWindowHeader {
            descriptor_length: window::LENGTH as u16,
        };
        let mut payload = Vec::with_capacity(window::HEADER + window::LENGTH);
        payload.extend_from_slice(&header.to_bytes());
        payload.extend_from_slice(&window.to_bytes());

        let cmd = SetWindow::new(payload.len() as u32);
        debug!(id = window.id, "setting window");
        self.run(&cmd.cdb(), Data::Out(&payload), MOVE_TIMEOUT)?;
        Ok(())
    }

    /// Start a scan of the windows named, and settle every cooperative request
    /// it raises
    ///
    /// 2-7: the unit answers, then scans, so this returns once it has started
    /// and TEST UNIT READY says when the data is there. A cooperative request
    /// means it will not start until the initiator has done a job for it, named
    /// by the `87h` record, after which the command goes out again
    pub fn scan(&mut self, windows: &[u8]) -> Result<(), Error> {
        let cmd = Scan::new(windows.len() as u8);
        let (_, coop) = self.run_cooperative(&cmd.cdb(), Data::Out(windows), MOVE_TIMEOUT)?;

        let Some(coop) = coop else {
            debug!(?windows, "scanning");
            return Ok(());
        };

        // Not an error, but we cannot honor it yet. Read the parameter anyway,
        // since it names the job precisely and the alternative is guessing from
        // a 4th sense byte the two specs disagree about. When the first job is
        // implemented this becomes a loop: do the work, issue SCAN again
        let record = self.cooperation()?;
        Err(Error::Unsupported {
            op: "host cooperation",
            reason: format!("{coop:?} is not implemented yet, and it wants {record:?}"),
        })
    }

    /// Read image data, continuing where the last read stopped
    ///
    /// Moves bytes and nothing else: type `00h` has no data header and no length
    /// of its own, and 2-11 has consecutive reads carry on rather than restart,
    /// so the caller sizes `buf` from the scan geometry. Unscrambling belongs to
    /// whatever consumes this.
    ///
    /// Answers how much arrived. Short of `buf` means the unit ran out, either
    /// by transferring less than asked or by answering `05h-2Ch` once the image
    /// is spent. `line` is the bytes in one scanned line, which `C1h` byte 4 can
    /// require every read to be a whole number of
    pub fn read_image(
        &mut self,
        buf: &mut [u8],
        line: usize,
        bytes_per_pixel: u8,
    ) -> Result<usize, Error> {
        let width = data::width_code(bytes_per_pixel).ok_or_else(|| Error::Unsupported {
            op: "image read",
            reason: format!("{bytes_per_pixel} bytes a pixel is not a width 2-11-4 encodes"),
        })?;

        let mut chunk = self.transport.max_transfer();
        if line > 0
            && self
                .caps
                .address
                .transfer
                .intersects(Transfer::READ_LINE | Transfer::READ_LINE_COLS)
        {
            chunk = (chunk / line).max(1) * line;
        }

        let code = data::DataType::Image.row().code;
        let mut done = 0;
        while done < buf.len() {
            let want = chunk.min(buf.len() - done);
            let cmd = Read::new(code, 0, width, want as u32);
            let slice = &mut buf[done..done + want];

            match self.run(&cmd.cdb(), Data::In(slice), MOVE_TIMEOUT) {
                Ok(completion) => {
                    done += completion.transferred;
                    if completion.transferred < want {
                        break;
                    }
                }
                // 2-11-5: reading past the end of the image is how it says the
                // image is spent, not a fault
                Err(Error::Device(fault))
                    if matches!(*fault, Fault::Rejected(Refusal::OutOfSequence, _)) =>
                {
                    break;
                }
                Err(e) => return Err(e),
            }
        }
        debug!(bytes = done, "read image");
        Ok(done)
    }

    /// Read the initiator cooperative action parameter a SCAN just asked for
    pub fn cooperation(&mut self) -> Result<data::CooperativeAction, Error> {
        let (_, values) = self.read_data(data::DataType::Cooperation, 0)?;
        let data::Values::Bytes(record) = values else {
            return Err(malformed("87h did not come back as bytes".into()));
        };
        data::CooperativeAction::from_bytes(&record)
            .ok_or_else(|| malformed(format!("87h was {} bytes", record.len())))
    }

    /// Issue a command, absorbing everything that means "not done yet", and hand back the completion once it has actually terminated
    ///
    /// `timeout` is a budget for the whole command including re-issues, not for one transfer.
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
    /// Only SCAN and READ raise one. 2-7 is explicit about what it means: read
    /// the parameter with `87h`, do the work, and issue the command again
    pub fn run_cooperative(
        &mut self,
        cdb: &[u8],
        mut data: Data<'_>,
        timeout: Duration,
    ) -> Result<(Completion, Option<Coop>), Error> {
        let deadline = Instant::now() + timeout;
        let mut changes = 0usize;
        // Polling produces the same outcome over and over, so log transitions
        // rather than every pass
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
                    // GET WINDOW is authoritative for what the unit actually
                    // used, so this is a note rather than a result
                    debug!(?adjustment, "the scanner moved a parameter");
                    return Ok((completion, None));
                }

                // Not yet. Polling is re-issuing.
                Outcome::Working(activity) => {
                    if reported != Some(activity) {
                        debug!(?activity, "waiting");
                        reported = Some(activity);
                    }
                    sleep(POLL_INTERVAL);
                }

                // Unit attention: the command did not run. Several can be
                // queued -- ejecting a holder raises a holder change and a
                // reset -- so this arm firing repeatedly is normal.
                Outcome::StateChanged(change) => {
                    debug!(?change, "device state changed under us, re-issuing");
                    // Several fields track the adapter and holder rather than
                    // the model, so what we cached may no longer describe the
                    // unit we are about to re-issue against
                    self.refresh()?;
                    changes += 1;
                    if changes >= MAX_CHANGES {
                        warn!(changes, "giving up on a device that will not settle");
                        return Err(Error::from_outcome(interpret(&completion), &completion));
                    }
                }

                // The scanner wants post-processing before it will go on
                Outcome::NeedsHost(coop) => return Ok((completion, Some(coop))),

                terminal => return Err(Error::from_outcome(terminal, &completion)),
            }
        }
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
