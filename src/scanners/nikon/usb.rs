//! What the two USB bodies do identically
//!
//! The LS-50 and the LS-5000 drive the mechanism the same way: same measurement units, same
//! readiness polling, same vendor register writes for focus and eject. Only the scan path
//! differs, and that stays in each driver.
//!
//! Everything here was written twice and agreed both times. Where the two disagreed it is not
//! here — see `docs/OPEN_QUESTIONS.md` sections 13 to 18 for the six places, and the focus read
//! length below for the one that reaches this far.

use super::cdbs::{VendorTrigger, VendorWrite};
use super::status_usb::UsbStatus;
use super::vendor_read_write::{VendorPayload, VendorRead};
use crate::scanners::{Focus, Scanner};
use crate::scsi::{
    self, Command, TransportExt,
    cdbs::{BlockDescriptor, ReleaseUnit, ReserveUnit, VpdInquiry},
    mode_pages::{BasicUnit, MeasurementUnits},
};
use std::thread::sleep;
use std::time::{Duration, Instant};
use tracing::{debug, trace, warn};

/// The measurement units these drivers pin at open, so every length is in sensor dots
pub const DOTS_PER_INCH: u32 = 4000;

/// How often the readiness poll asks again
pub const POLL_INTERVAL: Duration = Duration::from_millis(250);

/// How long it keeps asking. Long enough for a full-resolution pass.
pub const READY_TIMEOUT: Duration = Duration::from_secs(300);

/// A NOT READY, including the one the firmware dresses up as an illegal request
pub fn is_not_ready(sense: &scsi::SenseData) -> bool {
    matches!(sense.sense_key(), scsi::SenseKey::NotReady)
        || (matches!(sense.sense_key(), scsi::SenseKey::IllegalRequest) && sense.asc == 0x2C)
}

/// The mechanism control the USB bodies share
pub(crate) trait UsbCoolscan: Scanner<Status = UsbStatus> + Focus {
    /// Pin the measurement units, without which SET WINDOW is refused
    fn set_global_units(&mut self) -> Result<(), scsi::Error> {
        let units = MeasurementUnits {
            basic_unit: BasicUnit::Inches,
            divisor: DOTS_PER_INCH as u16,
        };
        let descriptor = BlockDescriptor {
            density_code: 0x00,
            number_of_blocks: 0x00,
            block_length: 0x01,
        };
        match self.transport().set_mode_page(&units, Some(descriptor)) {
            // Answered while still applying it, which is not a refusal
            Err(scsi::Error::Status { status, sense }) => {
                trace!(status, ?sense, "MODE SELECT reported busy");
                Ok(())
            }
            other => other,
        }
    }

    /// Issue a control command, treating a busy answer as success
    fn tolerate_busy<C: Command>(&mut self, command: &C) -> Result<(), scsi::Error> {
        match self.transport().send(command) {
            Ok(_) => Ok(()),
            Err(scsi::Error::Status { status, sense }) => {
                trace!(status, ?sense, "Control command reported busy");
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    /// Wait out every state that clears itself, and report whatever is left
    fn wait_settled(&mut self) -> Result<UsbStatus, scsi::Error> {
        let deadline = Instant::now() + READY_TIMEOUT;
        loop {
            let status = self.status()?;
            if !status.is_transient() {
                return Ok(status);
            }
            if Instant::now() >= deadline {
                warn!(?status, "Scanner never settled");
                return Err(scsi::Error::InvalidResponse("scanner never became ready"));
            }
            sleep(POLL_INTERVAL);
        }
    }

    /// [`wait_settled`](Self::wait_settled), refusing anything that settled short of ready
    fn wait_until_ready(&mut self) -> Result<(), scsi::Error> {
        match self.wait_settled()? {
            UsbStatus::Ready => Ok(()),
            status => {
                debug!(?status, "Scanner settled short of ready");
                Err(scsi::Error::InvalidResponse(
                    "scanner will not become ready without help",
                ))
            }
        }
    }

    /// Read the pages Nikon Scan reads on its way up
    ///
    /// Nothing consumes the answers. Kept because arming has never been verified without it.
    fn probe_adapter_pages(&mut self) {
        for (page, allocation_length) in [
            (0x00u8, 23u8),
            (0xD1, 28),
            (0xC1, 87),
            (0xE1, 39),
            (0xF0, 53),
            (0xF8, 17),
        ] {
            let _ = self
                .transport()
                .send(&VpdInquiry::new(page, allocation_length));
        }
    }

    /// The staged focus position
    ///
    /// `length` is the one thing the two bodies disagree about; each passes its own constant.
    /// See `docs/OPEN_QUESTIONS.md` section 18.
    fn read_focus(&mut self, length: u32) -> Result<u16, scsi::Error> {
        match self.transport().send(&VendorRead::focus(length))? {
            VendorPayload::Focus(focus) => {
                u16::try_from(focus).map_err(|_| scsi::Error::InvalidResponse("focus beyond a u16"))
            }
            // A VendorRead built with Subcode::Focus always decodes to Focus, see
            // VendorRead::parse_response
            _ => unreachable!(),
        }
    }

    /// Stage a focus target and commit it
    fn write_focus(&mut self, focus: u16) -> Result<(), scsi::Error> {
        self.tolerate_busy(&VendorWrite::new(VendorPayload::Focus(focus.into())))?;
        self.tolerate_busy(&VendorTrigger)
    }

    /// Focus on a point, and report where the motor landed
    fn autofocus(&mut self, (x, y): (u32, u32)) -> Result<u16, scsi::Error> {
        let before = self.focus().unwrap_or(0);
        self.tolerate_busy(&VendorWrite::new(VendorPayload::AutoFocus { x, y }))?;
        self.tolerate_busy(&VendorTrigger)?;
        self.wait_until_ready()?;
        let after = self.focus().unwrap_or(0);
        debug!(x, y, before, after, "Autofocus done");
        Ok(after)
    }

    /// Send the film back out
    fn eject(&mut self) -> Result<(), scsi::Error> {
        self.tolerate_busy(&ReserveUnit::default())?;
        self.tolerate_busy(&VendorWrite::new(VendorPayload::Eject))?;
        self.tolerate_busy(&VendorTrigger)?;
        // The motor runs for several seconds, reporting Ejecting the whole time. Let it settle
        // before handing the scanner back, or the release lands mid-motion.
        //
        // Whether it settled is the one part worth reporting: if it never did, the film is still
        // somewhere inside. A failed release only leaves a stale reservation behind.
        let settled = self.wait_settled();
        if let Err(e) = self.transport().send(&ReleaseUnit::default()) {
            debug!(%e, "Could not release the scanner after ejecting");
        }
        settled.map(|_| ())
    }
}
