use crate::{
    scanners::{FilmHolder, Focus, Scanner},
    scsi::{
        self as scsi, SenseKey, Transport, TransportExt,
        cdbs::*,
        mode_pages::{BasicUnit, MeasurementUnits},
    },
};
use cdbs::{Subcode, VendorPayload, VendorRead, VendorTrigger, VendorWrite};
use dtc::Dtc;
use holder::Holder;
use status::Status;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use tracing::*;

pub mod boundaries;
pub mod calibration;
pub mod cdbs;
pub mod decode;
pub mod dtc;
pub mod geometry;
pub mod holder;
pub mod status;
pub mod window;

pub use calibration::ChannelExposures;
pub use geometry::{CcdMode, Dpi, Multisample, ScanArea, ScanSettings};
pub use window::{BaseQuality, WindowKind, WindowParams};

/// This scanner always works in u16 pixels
pub const BITS_PER_PIXEL: u8 = 16;
/// This scanner's window descriptors are 50 bytes: 40 standardized bytes plus 10 vendor-specific
const WINDOW_DESCRIPTOR_LEN: u32 = 50;
/// This scanner always defines exactly 5 windows: 0 = all/composite, 1/2/3 = R/G/B, 9 = IR
const WINDOW_COUNT: u32 = 5;
/// SCSI-2 leaves control bits 7-6 vendor-specific, and Nikon Scan sets bit 7 on every
/// command with a data phase. GET WINDOW reads back zeroed geometry without it.
const VENDOR_CONTROL: u8 = 0x80;
/// Unit attentions queue up, but not without bound. Past this something is wrong.
const MAX_QUEUED_UNIT_ATTENTIONS: usize = 8;
/// The most tries a SCAN gets before we call it a refusal. Nikon Scan needs up to four.
const MAX_SCAN_ATTEMPTS: usize = 8;
/// How often [`wait_until_ready`](Ls9000ed::wait_until_ready) asks. Nikon Scan polls at
/// roughly this rate through an autofocus.
pub const POLL_INTERVAL: Duration = Duration::from_millis(200);
/// How long [`wait_until_ready`](Ls9000ed::wait_until_ready) keeps asking before giving up.
/// Long enough for a calibration, which Nikon Scan's own UI says can take two minutes.
const READY_TIMEOUT: Duration = Duration::from_secs(300);

/// Dots the stage advances per step it reports, 1/333 in, the same 12 dots the three CCD
/// lines are spaced by
const STAGE_STEP_DOTS: u32 = 12;
/// The step the scanner reports when the stage is at y=0. Autofocus on two frames 20160 dots
/// apart moved it by exactly 1680 steps, and both solve to this origin with no residual.
const STAGE_HOME_STEP: u32 = 171;

/// Stage steps as [`ScanArea`] dots
fn stage_dots(steps: u16) -> u32 {
    u32::from(steps).saturating_sub(STAGE_HOME_STEP) * STAGE_STEP_DOTS
}

/// The Nikon LS-9000 ED (Super Coolscan 9000)
pub struct Ls9000ed<T> {
    pub(crate) transport: T,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Color channels for the scanner's lamp
pub enum Channel {
    All,
    Red,
    Green,
    Blue,
    Ir,
}

impl Channel {
    /// The three visible channels, in the order Nikon Scan stages them
    pub const RGB: [Channel; 3] = [Channel::Red, Channel::Green, Channel::Blue];
    /// The visible channels plus infrared, as a dust-removal pass needs
    pub const RGBI: [Channel; 4] = [Channel::Red, Channel::Green, Channel::Blue, Channel::Ir];

    pub(crate) fn to_id(self) -> u8 {
        match self {
            Channel::All => 0,
            Channel::Red => 1,
            Channel::Green => 2,
            Channel::Blue => 3,
            Channel::Ir => 9,
        }
    }

    /// The window identifier as it comes back off the scanner
    pub(crate) fn from_id(id: u8) -> Option<Self> {
        Some(match id {
            0 => Channel::All,
            1 => Channel::Red,
            2 => Channel::Green,
            3 => Channel::Blue,
            9 => Channel::Ir,
            _ => return None,
        })
    }
}

/// The coolscan 9000 is SCSI-only, so we can gate here on scsi backends
impl<T> Ls9000ed<T>
where
    T: Transport,
{
    pub fn new(transport: T) -> Result<Self, scsi::Error> {
        let mut scanner = Ls9000ed { transport };

        // Everything below would choke on a queued unit attention, and there can be several:
        // ejecting a holder raises both a holder change and a reset.
        let initial_status = scanner.drain_unit_attentions()?;
        debug!(?initial_status, "Scanner state at open");

        // We always want exclusive access for the lifetime of this handle
        scanner.reserve()?;

        // On startup, make sure we set the working units to 4000 DPI
        // We will always assume these are the units everywhere (like NikonScan)
        // Without this, SET_WINDOW will fail because we haven't set a unit
        debug!("Setting global units to 4000 DPI");
        scanner.set_global_units()?;
        Ok(scanner)
    }

    /// Report the scanner's state, clearing any queued unit attentions first
    ///
    /// The device reports one unit attention per command and clears it as it goes, so a
    /// single [`status`](Scanner::status) only ever sees the oldest. Anything that can't
    /// tolerate a stray CHECK CONDITION needs this instead.
    pub fn drain_unit_attentions(&mut self) -> Result<Status, scsi::Error> {
        for _ in 0..MAX_QUEUED_UNIT_ATTENTIONS {
            let status = self.status()?;
            if !status.is_unit_attention() {
                return Ok(status);
            }
            debug!(?status, "Cleared a unit attention");
        }
        Err(scsi::Error::InvalidResponse(
            "scanner kept reporting unit attentions",
        ))
    }

    /// Block until the scanner finishes whatever it's doing
    ///
    /// Mechanical passes report NotReady for their whole duration, so this is how you wait
    /// one out: an autofocus, a scan, a calibration. Losing the holder mid-pass is an error
    /// rather than something to keep waiting on.
    pub fn wait_until_ready(&mut self) -> Result<(), scsi::Error> {
        let deadline = Instant::now() + READY_TIMEOUT;
        loop {
            match self.status()? {
                Status::Ready => return Ok(()),
                Status::NoFilmHolder => {
                    return Err(scsi::Error::InvalidResponse("the film holder was removed"));
                }
                state => trace!(?state, "Waiting"),
            }
            if Instant::now() >= deadline {
                return Err(scsi::Error::InvalidResponse("scanner never became ready"));
            }
            sleep(POLL_INTERVAL);
        }
    }

    /// Set our working units to "points" in 4000DPI increments
    ///
    /// NOTE: We will hard-code a set to 4000dpi as then we don't have to do math later
    /// The block descriptor is what Nikon Scan sends; without it SET WINDOW stays unarmed
    fn set_global_units(&mut self) -> Result<(), scsi::Error> {
        self.transport.set_mode_page(
            &MeasurementUnits {
                basic_unit: BasicUnit::Inches,
                divisor: 4000,
            },
            Some(BlockDescriptor {
                density_code: 0x00,
                number_of_blocks: 0x00,
                block_length: 0x01,
            }),
        )
    }

    /// `Some(channel)` fetches just that window's descriptor; `None` fetches every window this scanner has defined.
    pub fn get_window(
        &mut self,
        channel: Option<Channel>,
    ) -> Result<Vec<WindowDescriptor>, scsi::Error> {
        let (single, window_identifier, count) = match channel {
            Some(channel) => (true, channel.to_id(), 1),
            None => (false, 0, WINDOW_COUNT),
        };
        let transfer_length = 8 + count * WINDOW_DESCRIPTOR_LEN;
        self.transport.send(&GetWindow::new(
            0,
            single,
            window_identifier,
            transfer_length,
            VENDOR_CONTROL,
        ))
    }

    /// Configure `channel`'s window
    pub fn set_window(
        &mut self,
        channel: Channel,
        mut descriptor: WindowDescriptor,
    ) -> Result<(), scsi::Error> {
        descriptor.id = channel.to_id();
        self.transport
            .send(&SetWindow::new(0, &[descriptor], VENDOR_CONTROL))
    }

    /// Focus on one point of the film, in 1/4000-in dots, and report where it landed
    ///
    /// Blocks for the ten or so seconds the mechanism takes. Nikon Scan runs this once per
    /// frame before scanning it, aimed at [`FrameRect::center`](boundaries::FrameRect::center),
    /// and needs the frame table written first.
    pub fn autofocus(&mut self, (x, y): (u32, u32)) -> Result<u16, scsi::Error> {
        debug!(x, y, "Autofocusing");
        self.transport
            .send(&VendorWrite::new(VendorPayload::AutoFocus { x, y }))?;
        self.transport.send(&VendorTrigger)?;
        self.wait_until_ready()?;

        let focus = self.focus()?;
        debug!(focus, "Autofocus finished");
        Ok(focus)
    }

    /// Where the stage currently sits, in the same 1/4000-in dots as [`ScanArea`]
    ///
    /// Read back off vendor subcode 0x42, which reports the position in stage steps. Only
    /// legible at rest: the scanner refuses every vendor read while a pass is pending.
    pub fn stage_position(&mut self) -> Result<u32, scsi::Error> {
        let bytes = self.probe_vendor(0x42, 11)?;
        let steps = bytes
            .get(3..5)
            .map(|b| u16::from_be_bytes([b[0], b[1]]))
            .ok_or(scsi::Error::InvalidResponse("0x42 read too short"))?;
        Ok(stage_dots(steps))
    }

    /// Read an uncharacterized vendor register, for working out what the firmware exposes
    ///
    /// Reads only. Nothing here commands the mechanism.
    pub fn probe_vendor(&mut self, subcode: u8, length: u32) -> Result<Vec<u8>, scsi::Error> {
        match self
            .transport
            .send(&VendorRead::new(Subcode::Other(subcode), length))?
        {
            VendorPayload::Raw(bytes) => Ok(bytes),
            // Subcode::Other always decodes to Raw, see VendorRead::parse_response
            _ => unreachable!(),
        }
    }

    /// Read an uncharacterized vendor data structure, the DTC counterpart to
    /// [`probe_vendor`](Self::probe_vendor)
    pub fn probe_dtc(
        &mut self,
        code: u8,
        qualifier: u8,
        channel: Option<Channel>,
        length: u32,
    ) -> Result<Vec<u8>, scsi::Error> {
        self.read_dtc(Dtc::Other { code, qualifier }, channel, length)
    }

    /// Scan parameters
    ///
    /// Only readable while a scan is pending: once the pass finishes this returns
    /// `CommandSequenceError`, so [`scan`](Self::scan) reads it as part of its retry rather
    /// than exposing it as a post-scan query.
    fn scan_parameters(&mut self) -> Result<Vec<u8>, scsi::Error> {
        self.read_framed_dtc(Dtc::ScanParameters, None, dtc::HEADER_LEN)
    }

    /// Trigger a scan using the given channels' previously-configured windows
    ///
    /// The scanner works up to accepting a SCAN over several tries, rejecting each with a
    /// vendor sense whose ASCQ advances as it goes (0x80/0x01, then 0x04, then 0x07). Nikon
    /// Scan just keeps issuing it, reading the scan parameters in between: an 83-DPI overview
    /// takes two tries, a 666x333 prescan up to four. The parameter payload is sometimes all
    /// zeros, so it's the read itself that moves things along rather than anything in it.
    pub fn scan(&mut self, channels: &[Channel]) -> Result<(), scsi::Error> {
        let window_ids: Vec<_> = channels.iter().map(|c| c.to_id()).collect();

        for attempt in 0..MAX_SCAN_ATTEMPTS {
            match self.transport.send(&Scan::new(0, window_ids.clone(), 0x00)) {
                Err(scsi::Error::Status {
                    sense: Some(sense), ..
                }) if sense.sense_key() == SenseKey::VendorSpecific => {
                    debug!(attempt, ?sense, "SCAN rejected, retrying");
                    // Only readable in some of these states, and a refusal is not itself a
                    // reason to give up on the scan
                    match self.scan_parameters() {
                        Ok(parameters) => trace!(?parameters, "Scan parameters"),
                        Err(err) => trace!(?err, "Scan parameters unreadable"),
                    }
                }
                other => return other,
            }
        }
        Err(scsi::Error::InvalidResponse("scanner kept rejecting SCAN"))
    }
}

impl<T> Scanner for Ls9000ed<T>
where
    T: Transport,
{
    type Status = Status;

    fn identify(&mut self) -> Result<InquiryResponse, scsi::Error> {
        self.transport.send(&Inquiry::new())
    }

    fn status(&mut self) -> Result<Status, scsi::Error> {
        // Unfold "Errors" from the state buffer into normal ok states
        match self.transport.send(&TestUnitReady::new()) {
            Ok(()) => Ok(Status::Ready),
            Err(err) => {
                if let scsi::Error::Status {
                    sense: Some(sense), ..
                } = &err
                    && let Some(state) = Status::from_sense(sense)
                {
                    return Ok(state);
                }
                Err(err)
            }
        }
    }

    fn reserve(&mut self) -> Result<(), scsi::Error> {
        self.transport.send(&ReserveUnit::default())
    }

    fn release(&mut self) -> Result<(), scsi::Error> {
        self.transport.send(&ReleaseUnit::default())
    }

    fn read_chunk(&mut self, want: u32) -> Result<Vec<u8>, scsi::Error> {
        self.transport.send(&Read::new(
            0,
            DataTypeCode::Image,
            0x0000,
            want,
            VENDOR_CONTROL,
        ))
    }
}

impl<T> FilmHolder for Ls9000ed<T>
where
    T: Transport,
{
    type Holder = Holder;

    fn holder(&mut self) -> Result<Holder, scsi::Error> {
        self.transport.vpd()
    }
}

impl<T> Focus for Ls9000ed<T>
where
    T: Transport,
{
    /// May be a setpoint rather than the motor's actual physical position
    /// See [`VendorPayload::Focus`]
    fn focus(&mut self) -> Result<u16, scsi::Error> {
        match self.transport.send(&VendorRead::new(Subcode::Focus, 9))? {
            VendorPayload::Focus(focus) => Ok(focus),
            // A VendorRead built with Subcode::Focus always decodes to
            // VendorPayload::Focus - see VendorRead::parse_response.
            _ => unreachable!(),
        }
    }

    /// Staged, then committed via TRIGGER
    fn set_focus(&mut self, focus: u16) -> Result<(), scsi::Error> {
        self.transport
            .send(&VendorWrite::new(VendorPayload::Focus(focus)))?;
        self.transport.send(&VendorTrigger)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both autofocus points land exactly on their target, which is what fixes the mapping
    #[test]
    fn stage_steps_match_the_probed_positions() {
        assert_eq!(stage_dots(738), 6804);
        assert_eq!(stage_dots(2418), 26964);
        // The first probe run caught it sitting at the origin
        assert_eq!(stage_dots(171), 0);
        // A thumbnail leaves it one step short of full travel
        assert_eq!(stage_dots(3055), ScanArea::STRIP_DOTS - 36);
    }

    /// A reading below the origin would otherwise wrap into a huge position
    #[test]
    fn steps_below_home_clamp_to_zero() {
        assert_eq!(stage_dots(0), 0);
        assert_eq!(stage_dots(STAGE_HOME_STEP as u16 - 1), 0);
    }
}
