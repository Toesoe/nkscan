//! The Nikon LS-5000 ED (Super Coolscan 5000 ED), a 35 mm USB film scanner
//!
//! 16-bit samples, hardware multi-sampling, and a motorised feeder that senses frames along a
//! whole roll and reports where they are.
//!
//! Nothing here has been run against a real unit. See `docs/LS5000_HARDWARE_CHECKLIST.md`.

use crate::{
    decode::{Image, StreamDecoder},
    scanners::{
        FilmHolder, Flow, Focus, Scanner,
        nikon::{
            Channel, ChannelExposures,
            cdbs::{Subcode, VendorTrigger, VendorWrite},
            limits::DeviceLimits,
        },
    },
    scsi::{
        self as scsi, Command, Transport, TransportExt,
        cdbs::*,
        mode_pages::{BasicUnit, MeasurementUnits},
    },
};
use cdbs::vendor_read_write::{VendorPayload, VendorRead};
use decode::{DecodeError, frame_decoder};
use dtc::Dtc;
use geometry::ScanSettings;
use holder::Holder;
use status::Status;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use tracing::*;
use window::WindowParams;

pub mod boundaries;
pub mod calibration;
pub mod capabilities;
pub mod cdbs;
pub mod decode;
pub mod dtc;
pub mod geometry;
pub mod holder;
pub mod status;
pub mod window;

/// For [`UsbTransport::open`](crate::scsi::usb::UsbTransport::open)
pub const VENDOR_ID: u16 = 0x04B0;
/// The LS-50 ED and the LS-5000 ED share 0x04B0 and are told apart here. The LS-50 is 0x4001.
pub const PRODUCT_ID: u16 = 0x4002;

/// 40 standardized bytes plus 10 vendor
const WINDOW_DESCRIPTOR_LEN: u32 = 50;
/// SCSI-2 leaves control bits 7-6 vendor-specific. Every command here sets bit 7.
const VENDOR_CONTROL: u8 = 0x80;

/// SCAN is refused until DTC 0x87 has been read, and the refusal advances across attempts
const MAX_SCAN_ATTEMPTS: usize = 8;
/// Long enough for the lamp to make progress between tries
const SCAN_RETRY_PAUSE: Duration = Duration::from_millis(500);
/// How often the driver asks a busy scanner whether it has settled
pub const POLL_INTERVAL: Duration = Duration::from_millis(250);
/// How long [`wait_until_ready`](Ls5000::wait_until_ready) keeps asking before giving up
///
/// Generous on purpose: a pass reports NotReady throughout, and repositioning a roll is slow.
/// Firing early leaves the next command reaching a moving transport.
const READY_TIMEOUT: Duration = Duration::from_secs(300);
/// Mid-pass not-ready means the next chunk isn't out of the carriage yet, not end of data
const IMAGE_IDLE_TIMEOUT: Duration = Duration::from_secs(120);
/// Shorter than the ready poll: a chunk arrives in tens of milliseconds once the carriage moves
const IMAGE_IDLE_PAUSE: Duration = Duration::from_millis(200);
/// Image reads are 512-aligned, which is also what a line is padded to
const READ_ALIGNMENT: u32 = 512;
/// What the driver asks for per image read when the transport will carry it
///
/// Bulk transfers have no protocol ceiling, so this is a chunk size rather than a limit, and
/// it is clamped to the transport own maximum.
const IMAGE_CHUNK: u32 = 128 * 1024;

/// The channels one pass captures, infrared first
///
/// Whether the firmware cares about the order is unverified.
fn channels(settings: &ScanSettings) -> &'static [Channel] {
    const RGB: [Channel; 3] = [Channel::Red, Channel::Green, Channel::Blue];
    const IRGB: [Channel; 4] = [Channel::Ir, Channel::Red, Channel::Green, Channel::Blue];
    if settings.ir { &IRGB } else { &RGB }
}

/// Mid-pass "the chunk isn't ready yet", as opposed to end of data
fn is_not_ready(sense: &scsi::SenseData) -> bool {
    matches!(sense.sense_key(), scsi::SenseKey::NotReady)
        || (matches!(sense.sense_key(), scsi::SenseKey::IllegalRequest) && sense.asc == 0x2C)
}

/// The Nikon LS-5000 ED
///
/// Generic over the transport, but this model has no SCSI bus, so in practice
/// [`UsbTransport`](crate::scsi::usb::UsbTransport).
pub struct Ls5000<T> {
    pub(crate) transport: T,
    capabilities: DeviceLimits,
}

impl<T> Ls5000<T>
where
    T: Transport,
{
    /// Open a handle, without moving anything
    ///
    /// Film motion waits for [`warm_up`](Self::warm_up), so read-only callers never spin the
    /// transport.
    pub fn new(mut transport: T) -> Result<Self, scsi::Error> {
        // A scanner still coming up from power-on refuses everything, INQUIRY included
        let coming_up: Status =
            crate::scanners::wait_while_initializing(&mut transport, READY_TIMEOUT, POLL_INTERVAL)?;
        trace!(?coming_up, "Scanner state before the capability read");

        // A cold start queues several unit attentions and everything below would choke on one
        let initial_status: Status = crate::scanners::drain_unit_attentions(&mut transport)?;
        debug!(?initial_status, "Scanner state at open");

        // Everything this will accept comes from the device
        let capabilities = capabilities::read(&mut transport)?;
        debug!(?capabilities, "Scanner capabilities");

        let mut scanner = Ls5000 {
            transport,
            capabilities,
        };
        scanner.tolerate_busy(&ReserveUnit::default())?;
        scanner.set_global_units()?;
        Ok(scanner)
    }

    /// What the adapter reported about itself when this handle was opened
    pub fn capabilities(&self) -> DeviceLimits {
        self.capabilities
    }

    /// Slots sensed on the loaded roll, 0 for none
    ///
    /// Read fresh every call, since loading or ejecting changes it.
    pub fn sensed_frames(&mut self) -> u32 {
        capabilities::read_sensed_frames(&mut self.transport)
    }

    /// Read an uncharacterized vendor register
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

    /// Read an uncharacterized framed vendor structure
    pub fn probe_dtc(&mut self, code: u8, qualifier: u8) -> Result<Vec<u8>, scsi::Error> {
        self.read_framed_dtc(Dtc::Other { code, qualifier }, None)
    }

    /// Pin the working units to 1/4000 in, the sensor's own pitch
    ///
    /// Without this the scan mode stays unset and SET WINDOW is rejected.
    fn set_global_units(&mut self) -> Result<(), scsi::Error> {
        let units = MeasurementUnits {
            basic_unit: BasicUnit::Inches,
            divisor: geometry::DOTS_PER_INCH as u16,
        };
        let descriptor = BlockDescriptor {
            density_code: 0x00,
            number_of_blocks: 0x00,
            block_length: 0x01,
        };
        match self.transport.set_mode_page(&units, Some(descriptor)) {
            // Answered while still applying it, which is not a refusal
            Err(scsi::Error::Status { status, sense }) => {
                trace!(status, ?sense, "MODE SELECT reported busy");
                Ok(())
            }
            other => other,
        }
    }

    /// `Some(channel)` fetches that window's descriptor, `None` whatever the scanner leads with
    pub fn get_window(
        &mut self,
        channel: Option<Channel>,
    ) -> Result<Vec<WindowDescriptor>, scsi::Error> {
        let (single, window_identifier) = match channel {
            Some(channel) => (true, channel.to_id()),
            None => (false, 0),
        };
        self.transport.send(&GetWindow::new(
            0,
            single,
            window_identifier,
            8 + WINDOW_DESCRIPTOR_LEN,
            VENDOR_CONTROL,
        ))
    }

    /// A RECOVERED ERROR means the window was taken with a value snapped to the scanner's grid,
    /// so it counts as success
    pub fn set_window(
        &mut self,
        channel: Channel,
        mut descriptor: WindowDescriptor,
    ) -> Result<(), scsi::Error> {
        // Inclusive: a window exactly `boundary_y` long is legal here
        if descriptor.length > self.capabilities.boundary_y {
            warn!(
                length = descriptor.length,
                boundary = self.capabilities.boundary_y,
                "Refusing a window past the adapter's reported scan area"
            );
            return Err(scsi::Error::Unsupported(
                "window longer than the adapter's reported scan area",
            ));
        }
        self.capabilities
            .allows_resolution(descriptor.x_resolution, descriptor.y_resolution)?;

        descriptor.id = channel.to_id();
        match self
            .transport
            .send(&SetWindow::new(0, &[descriptor], VENDOR_CONTROL))
        {
            Err(scsi::Error::Status {
                sense: Some(sense), ..
            }) if sense.sense_key() == scsi::SenseKey::RecoveredError => {
                debug!(?sense, "SET WINDOW accepted with a recovered error");
                Ok(())
            }
            other => other,
        }
    }

    /// Send a control command that answers CHECK CONDITION while it runs. A real transport
    /// failure still propagates.
    pub(crate) fn tolerate_busy<C: Command>(&mut self, command: &C) -> Result<(), scsi::Error> {
        match self.transport.send(command) {
            Ok(_) => Ok(()),
            Err(scsi::Error::Status { status, sense }) => {
                trace!(status, ?sense, "Control command reported busy");
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}

impl<T> Scanner for Ls5000<T>
where
    T: Transport,
{
    type Status = Status;
    type Transport = T;

    fn transport(&mut self) -> &mut T {
        &mut self.transport
    }

    /// A bulk slice of the pending pass, gated on the carriage having produced it
    ///
    /// An empty return means the pass ended early, which
    /// [`read_into`](Scanner::read_into) reports as a short stream.
    fn read_chunk(&mut self, want: u32) -> Result<Vec<u8>, scsi::Error> {
        self.read_image(want)
    }
}

impl<T> FilmHolder for Ls5000<T>
where
    T: Transport,
{
    type Holder = Holder;

    fn holder(&mut self) -> Result<Holder, scsi::Error> {
        self.transport.vpd()
    }
}

impl<T> Focus for Ls5000<T>
where
    T: Transport,
{
    /// The staged setpoint, which the motor may still be traveling towards
    fn focus(&mut self) -> Result<u16, scsi::Error> {
        match self.transport.send(&VendorRead::focus())? {
            VendorPayload::Focus(focus) => {
                u16::try_from(focus).map_err(|_| scsi::Error::InvalidResponse("focus beyond a u16"))
            }
            // A VendorRead built with Subcode::Focus always decodes to Focus
            _ => unreachable!(),
        }
    }

    /// Staged, then committed via TRIGGER. 0 parks the motor.
    fn set_focus(&mut self, focus: u16) -> Result<(), scsi::Error> {
        self.tolerate_busy(&VendorWrite::new(VendorPayload::Focus(focus.into())))?;
        self.tolerate_busy(&VendorTrigger)
    }
}

/// Either half of a pass can fail: the transport, or decoding what came back
pub type ScanError = crate::scanners::ReadError<DecodeError>;

/// The scan drive: warm-up, arming, and draining the image
impl<T> Ls5000<T>
where
    T: Transport,
{
    /// Bring the scanner from cold to scannable
    ///
    /// The self-test and lamp both move the carriage, so this stops short without film and
    /// skips the motion on a feeder, which would spit its roll back out.
    ///
    /// Unverified on this model.
    pub fn warm_up(&mut self) -> Result<(), scsi::Error> {
        if self.wait_settled()? == Status::NoFilm {
            return Err(scsi::Error::InvalidResponse("no film loaded"));
        }
        let feeder = self.holder().map(Holder::is_roll).unwrap_or(false);
        self.tolerate_busy(&ReserveUnit::default())?;
        self.probe_adapter_pages();
        if !feeder {
            self.tolerate_busy(&SendDiagnostic::self_test())?;
            self.tolerate_busy(&VendorWrite::new(VendorPayload::Lamp))?;
            self.tolerate_busy(&VendorTrigger)?;
        }
        self.wait_until_ready()
    }

    /// Eject the loaded film
    pub fn eject(&mut self) -> Result<(), scsi::Error> {
        self.tolerate_busy(&ReserveUnit::default())?;
        self.tolerate_busy(&VendorWrite::new(VendorPayload::Eject))?;
        self.tolerate_busy(&VendorTrigger)?;
        // The motor runs for several seconds, reporting Ejecting the whole time. Let it settle
        // before handing the scanner back, or the release lands mid-motion.
        //
        // Whether it settled is the one part worth reporting: if it never did, the film is
        // still somewhere inside. A failed release only leaves a stale reservation behind.
        let settled = self.wait_settled();
        if let Err(e) = self.release() {
            debug!(%e, "Could not release the scanner after ejecting");
        }
        settled.map(|_| ())
    }

    /// Focus on one point of the film, in 1/4000-in dots
    ///
    /// Aim it at the [`center`](ScanSettings::center) of the frame about to be scanned. Blocks
    /// for the ten or so seconds it takes, and reports where the setpoint landed.
    pub fn autofocus(&mut self, (x, y): (u32, u32)) -> Result<u16, scsi::Error> {
        let before = self.focus().unwrap_or(0);
        self.tolerate_busy(&VendorWrite::new(VendorPayload::AutoFocus { x, y }))?;
        self.tolerate_busy(&VendorTrigger)?;
        self.wait_until_ready()?;
        let after = self.focus().unwrap_or(0);
        debug!(x, y, before, after, "Autofocus done");
        Ok(after)
    }

    /// Stage the channels this pass needs, scan, and decode what comes back
    pub fn scan_image(
        &mut self,
        settings: &ScanSettings,
        gain: ChannelExposures,
    ) -> Result<Image, ScanError> {
        self.scan_image_with(settings, gain, |_, _| Flow::Continue)
    }

    /// [`scan_image`](Self::scan_image), reporting (received, expected) bytes as it reads
    pub fn scan_image_with<F: FnMut(u64, u64) -> Flow>(
        &mut self,
        settings: &ScanSettings,
        gain: ChannelExposures,
        progress: F,
    ) -> Result<Image, ScanError> {
        // Checked before anything is armed. A window narrower than one output pixel gives a
        // zero-length line, and the read loop below would then never run: the SCAN would be
        // left pending with nothing draining it, and an empty image handed back as a success.
        if settings.bytes_per_line() == 0 {
            return Err(scsi::Error::Unsupported(
                "window is narrower than one output pixel at this resolution",
            )
            .into());
        }

        self.arm(settings, gain)?;
        self.scan(channels(settings))?;

        let mut decoder = frame_decoder(settings);
        let chunk = self.image_chunk(settings);
        self.read_into_with(&mut decoder, chunk, progress)?;
        let frame = decoder.finish().map_err(ScanError::Decode)?.to_owned();
        debug!(
            width = frame.rgb.width(),
            height = frame.rgb.height(),
            "Image drained"
        );
        Ok(frame)
    }

    /// How much to ask for per image read
    ///
    /// The scanner does not stream in line units: reads are 512-aligned bulk chunks that
    /// straddle lines, and the block decoder reassembles. Never below one line, so a pass
    /// always makes progress.
    fn image_chunk(&mut self, settings: &ScanSettings) -> u32 {
        let ceiling = IMAGE_CHUNK.min(self.transport.max_transfer());
        let aligned = (ceiling / READ_ALIGNMENT) * READ_ALIGNMENT;
        aligned.max(settings.bytes_per_line() as u32)
    }

    /// Arm one pass: windows, then the window read-back. The caller issues the SCAN.
    ///
    /// No gamma table is uploaded: there is no hardware LUT on these scanners.
    fn arm(&mut self, settings: &ScanSettings, gain: ChannelExposures) -> Result<(), scsi::Error> {
        // Refused rather than put on the wire. A multi-sampled pass does not stream a planar
        // image: it sends every sample for the host to average, in a record shape this driver
        // does not decode. Arming one would read the declared length off a longer stream and
        // fail somewhere in the middle of it.
        if settings.samples.is_multi() {
            return Err(scsi::Error::Unsupported(
                "multi-sampled readout is not implemented",
            ));
        }

        let channels = channels(settings);

        for &channel in channels {
            let params = WindowParams {
                samples: settings.samples,
                exposure: gain.get(channel),
            };
            self.set_window(channel, params.descriptor(settings, channel))?;
            debug!(?channel, "Arm: window set");
        }

        // Load-bearing: without the read-back the pass never reaches read-ready
        for _ in channels {
            let _ = self.get_window(None);
        }
        // Arming moves nothing, so this returns at once on an idle scanner. Ignored: the
        // device only has to be ready for the SCAN that follows.
        let _ = self.wait_until_ready();
        debug!("Arm: windows read back");
        Ok(())
    }

    /// Start a pass over the given channels' previously-configured windows
    ///
    /// SCAN is refused until the scan parameters have been read, and the refusal advances
    /// across attempts rather than repeating, so the read is part of the retry rather than a
    /// preamble to it.
    pub fn scan(&mut self, channels: &[Channel]) -> Result<(), scsi::Error> {
        let window_ids: Vec<_> = channels.iter().map(|c| c.to_id()).collect();
        for attempt in 0..MAX_SCAN_ATTEMPTS {
            match self
                .transport
                .send(&Scan::new(0, window_ids.clone(), VENDOR_CONTROL))
            {
                Ok(()) => return Ok(()),
                Err(scsi::Error::Status { sense, .. }) => {
                    debug!(attempt, ?sense, "SCAN refused, reading scan parameters");
                    // Reading this is what clears the refusal; what it reports is
                    // uncharacterized, so the payload is logged rather than acted on
                    match self.read_framed_dtc(Dtc::ScanParameters, None) {
                        Ok(parameters) => trace!(?parameters, "Scan parameters"),
                        Err(e) => debug!(%e, "Could not read the scan parameters"),
                    }
                    sleep(SCAN_RETRY_PAUSE);
                }
                Err(err) => return Err(err),
            }
        }
        Err(scsi::Error::InvalidResponse("scanner kept rejecting SCAN"))
    }

    /// Walk the adapter-configuration VPD pages before arming
    ///
    /// Read-only, every result discarded.
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
                .transport
                .send(&VpdInquiry::new(page, allocation_length));
        }
    }

    /// Read one bulk slice of the image, waiting for the carriage to produce it
    ///
    /// Reading mid-positioning aborts the scan, hence the status gate.
    fn read_image(&mut self, want: u32) -> Result<Vec<u8>, scsi::Error> {
        let deadline = Instant::now() + IMAGE_IDLE_TIMEOUT;
        while Instant::now() < deadline {
            if !matches!(self.status(), Ok(Status::Ready)) {
                sleep(IMAGE_IDLE_PAUSE);
                continue;
            }
            match self.read_dtc(Dtc::Image, None, want) {
                Ok(bytes) => return Ok(bytes),
                // Flipped back to not-ready between the poll and the read: same chunk again
                Err(scsi::Error::Status {
                    sense: Some(sense), ..
                }) if is_not_ready(&sense) => sleep(IMAGE_IDLE_PAUSE),
                // Anything else ends the transfer
                Err(scsi::Error::Status { sense, .. }) => {
                    debug!(?sense, "Image: end of data");
                    return Ok(Vec::new());
                }
                Err(err) => return Err(err),
            }
        }
        Err(scsi::Error::InvalidResponse(
            "scanner stopped producing image data",
        ))
    }

    /// Poll until the state is one that waiting can't change
    fn wait_settled(&mut self) -> Result<Status, scsi::Error> {
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

    /// Poll until ready, draining the transient states on the way
    pub fn wait_until_ready(&mut self) -> Result<(), scsi::Error> {
        match self.wait_settled()? {
            Status::Ready => Ok(()),
            status => {
                debug!(?status, "Scanner settled short of ready");
                Err(scsi::Error::InvalidResponse(
                    "scanner will not become ready without help",
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests;
