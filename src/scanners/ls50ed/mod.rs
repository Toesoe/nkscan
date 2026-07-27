use crate::{
    decode::StreamDecoder,
    scanners::{FilmHolder, Focus, Scanner},
    scsi::{
        self as scsi, Command, Transport, TransportExt,
        cdbs::*,
        mode_pages::{BasicUnit, MeasurementUnits},
    },
};
use cdbs::{VendorPayload, VendorTrigger, VendorWrite};
use decode::{DecodeError, FrameDecoder, Image};
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
pub mod capabilities;
pub mod cdbs;
pub mod decode;
pub mod dtc;
pub mod geometry;
pub mod holder;
pub mod status;
pub mod window;

pub use calibration::ChannelExposures;
pub use capabilities::Capabilities;
pub use geometry::{Dpi, ScanArea, ScanSettings, frame_offset, native_dots};
pub use window::{ScanMode, WindowParams};

/// For [`UsbTransport::open`](crate::scsi::usb::UsbTransport::open)
pub const VENDOR_ID: u16 = 0x04B0;
/// The LS-50 ED and the LS-5000 ED share 0x04B0 and are told apart here
pub const PRODUCT_ID: u16 = 0x4001;

/// 40 standardized bytes plus 10 vendor
const WINDOW_DESCRIPTOR_LEN: u32 = 50;
/// SCSI-2 leaves control bits 7-6 vendor-specific. Only SET WINDOW needs bit 7 here.
const VENDOR_CONTROL: u8 = 0x80;
/// Unit attentions queue up, but not without bound. Past this something is wrong.
const MAX_QUEUED_UNIT_ATTENTIONS: usize = 8;

/// SCAN answers CHECK CONDITION while the lamp and carriage warm up, so retry it
const MAX_SCAN_ATTEMPTS: usize = 30;
/// Long enough for the lamp to make progress between tries, so the budget covers ~15 s
const SCAN_RETRY_PAUSE: Duration = Duration::from_millis(500);
/// How often the driver asks a busy scanner whether it has settled
pub const POLL_INTERVAL: Duration = Duration::from_millis(250);
/// How long [`wait_until_ready`](Ls50ed::wait_until_ready) keeps asking before giving up
///
/// Generous on purpose: a pass reports NotReady throughout and an autoexposure measured 29 s.
/// Firing early leaves the next command reaching a moving scanner.
const READY_TIMEOUT: Duration = Duration::from_secs(300);
/// Mid-pass not-ready means the next line isn't out of the carriage yet, not end of data
const IMAGE_IDLE_TIMEOUT: Duration = Duration::from_secs(120);
/// Shorter than the ready poll: a line arrives in tens of milliseconds once the carriage moves
const IMAGE_IDLE_PAUSE: Duration = Duration::from_millis(200);

/// The readiness state out of TEST UNIT READY, transient not-ready folded in as an ok state
///
/// Free-standing so opening a handle can ask before there is a scanner to ask.
fn status_of<T: Transport + ?Sized>(transport: &mut T) -> Result<Status, scsi::Error> {
    match transport.send(&TestUnitReady::new()) {
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

/// See [`Ls50ed::drain_unit_attentions`]
fn drain_unit_attentions<T: Transport + ?Sized>(transport: &mut T) -> Result<Status, scsi::Error> {
    for _ in 0..MAX_QUEUED_UNIT_ATTENTIONS {
        let status = status_of(transport)?;
        if !status.is_unit_attention() {
            return Ok(status);
        }
        debug!(?status, "Cleared a unit attention");
    }
    Err(scsi::Error::InvalidResponse(
        "scanner kept reporting unit attentions",
    ))
}

/// The channels one pass captures
fn channels(settings: &ScanSettings) -> &'static [Channel] {
    if settings.ir {
        &Channel::RGBI
    } else {
        &Channel::RGB
    }
}

/// Mid-pass "the line isn't ready yet", as opposed to end of data
fn is_not_ready(sense: &scsi::SenseData) -> bool {
    matches!(sense.sense_key(), scsi::SenseKey::NotReady)
        || (matches!(sense.sense_key(), scsi::SenseKey::IllegalRequest) && sense.asc == 0x2C)
}

/// 16384 big-endian words, `table[i] = i`. Applied in hardware, so identity is what keeps
/// the output a linear raw.
fn identity_gamma_table() -> Vec<u8> {
    (0u16..16384).flat_map(|i| i.to_be_bytes()).collect()
}

/// The Nikon LS-50 ED (Super Coolscan V ED), a 35 mm USB film scanner
///
/// Generic over the transport, but this model has no SCSI bus, so in practice
/// [`UsbTransport`](crate::scsi::usb::UsbTransport).
pub struct Ls50ed<T> {
    pub(crate) transport: T,
    capabilities: Capabilities,
}

/// Color channels for the scanner's lamp
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Channel {
    Red,
    Green,
    Blue,
    Ir,
}

impl Channel {
    /// In the order the scanner stages them
    pub const RGB: [Channel; 3] = [Channel::Red, Channel::Green, Channel::Blue];
    pub const RGBI: [Channel; 4] = [Channel::Red, Channel::Green, Channel::Blue, Channel::Ir];

    pub(crate) fn to_id(self) -> u8 {
        match self {
            Channel::Red => 1,
            Channel::Green => 2,
            Channel::Blue => 3,
            Channel::Ir => 9,
        }
    }
}

impl<T> Ls50ed<T>
where
    T: Transport,
{
    /// Open a handle, without moving anything
    ///
    /// Film motion waits for [`warm_up`](Self::warm_up), so read-only callers never spin
    /// the motor.
    pub fn new(mut transport: T) -> Result<Self, scsi::Error> {
        // A cold start queues several unit attentions and everything below would choke on
        // one. Drained before the capability read, so a stray CHECK CONDITION cannot look
        // like a device with no geometry to report.
        let initial_status = drain_unit_attentions(&mut transport)?;
        debug!(?initial_status, "Scanner state at open");

        // Everything this will accept comes from the device
        let capabilities = capabilities::read(&mut transport)?;
        debug!(?capabilities, "Scanner capabilities");

        let mut scanner = Ls50ed {
            transport,
            capabilities,
        };
        scanner.tolerate_busy(&ReserveUnit::default())?;
        scanner.set_global_units()?;
        Ok(scanner)
    }

    /// Report the scanner's state, clearing any queued unit attentions first
    ///
    /// The device reports one per command, so a single [`status`](Scanner::status) sees
    /// only the oldest.
    pub fn drain_unit_attentions(&mut self) -> Result<Status, scsi::Error> {
        drain_unit_attentions(&mut self.transport)
    }

    /// What the adapter reported about itself when this handle was opened
    pub fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    /// Frames sensed on the loaded strip, 0 for none
    ///
    /// Read fresh every call, since a pass or an eject changes it. Six frames read 6 before
    /// scanning and 1 after any pass, with nothing to say which you are looking at.
    pub fn sensed_frames(&mut self) -> u32 {
        capabilities::read_sensed_frames(&mut self.transport)
    }

    /// Read a vital product data page. Page 0x00 lists the ones this device has.
    pub fn vpd_page(&mut self, page_code: u8) -> Result<Vec<u8>, scsi::Error> {
        Ok(self.transport.send(&VpdInquiry::new(page_code, 0xFF))?.data)
    }

    /// Read an uncharacterized vendor register
    pub fn probe_vendor(&mut self, subcode: u8, length: u32) -> Result<Vec<u8>, scsi::Error> {
        match self.transport.send(&cdbs::VendorRead::new(
            cdbs::Subcode::Other(subcode),
            length,
        ))? {
            cdbs::VendorPayload::Raw(bytes) => Ok(bytes),
            // Subcode::Other always decodes to Raw, see VendorRead::parse_response
            _ => unreachable!(),
        }
    }

    /// Pin the working units to 1/4000 in, the sensor's own pitch
    ///
    /// Without this the scan mode stays unset and SET WINDOW is rejected. The block descriptor
    /// is what Nikon Scan sends.
    fn set_global_units(&mut self) -> Result<(), scsi::Error> {
        let units = MeasurementUnits {
            basic_unit: BasicUnit::Inches,
            divisor: 4000,
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
            0x00,
        ))
    }

    /// A RECOVERED ERROR means the window was taken with a value snapped to the scanner's
    /// grid, so it counts as success
    pub fn set_window(
        &mut self,
        channel: Channel,
        mut descriptor: WindowDescriptor,
    ) -> Result<(), scsi::Error> {
        // The firmware enforces this itself with InvalidFieldInParameterList: against a
        // boundary of 5959, 5940 dots is taken and 5967 refused. Checked here to fail before
        // the mechanism rather than after.
        if descriptor.length >= self.capabilities.boundary_y {
            warn!(
                length = descriptor.length,
                boundary = self.capabilities.boundary_y,
                "Refusing a window past the adapter's reported scan area"
            );
            return Err(scsi::Error::Unsupported(
                "window longer than the adapter's reported scan area",
            ));
        }

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

impl<T> Scanner for Ls50ed<T>
where
    T: Transport,
{
    type Status = Status;

    fn identify(&mut self) -> Result<InquiryResponse, scsi::Error> {
        self.transport.send(&Inquiry::new())
    }

    fn status(&mut self) -> Result<Status, scsi::Error> {
        status_of(&mut self.transport)
    }

    fn reserve(&mut self) -> Result<(), scsi::Error> {
        self.transport.send(&ReserveUnit::default())
    }

    fn release(&mut self) -> Result<(), scsi::Error> {
        self.transport.send(&ReleaseUnit::default())
    }

    /// One line per read, gated on the carriage having produced it
    ///
    /// `want` is a whole padded line. An empty return means the pass ended early, which
    /// [`read_into`](Scanner::read_into) reports as a short stream.
    fn read_chunk(&mut self, want: u32) -> Result<Vec<u8>, scsi::Error> {
        self.read_line(want)
    }
}

impl<T> FilmHolder for Ls50ed<T>
where
    T: Transport,
{
    type Holder = Holder;

    fn holder(&mut self) -> Result<Holder, scsi::Error> {
        self.transport.vpd()
    }
}

impl<T> Focus for Ls50ed<T>
where
    T: Transport,
{
    /// The staged setpoint, which the motor may still be traveling towards
    fn focus(&mut self) -> Result<u16, scsi::Error> {
        match self.transport.send(&cdbs::VendorRead::focus())? {
            cdbs::VendorPayload::Focus(focus) => {
                u16::try_from(focus).map_err(|_| scsi::Error::InvalidResponse("focus beyond a u16"))
            }
            // A VendorRead built with Subcode::Focus always decodes to Focus, see
            // VendorRead::parse_response
            _ => unreachable!(),
        }
    }

    /// Staged, then committed via TRIGGER. 0 parks the motor.
    fn set_focus(&mut self, focus: u16) -> Result<(), scsi::Error> {
        self.tolerate_busy(&cdbs::VendorWrite::new(cdbs::VendorPayload::Focus(
            focus.into(),
        )))?;
        self.tolerate_busy(&cdbs::VendorTrigger)
    }
}

/// Either half of a pass can fail: the transport, or decoding what came back
pub type ScanError = crate::scanners::ReadError<DecodeError>;

/// The scan drive: warm-up, arming, and draining the image
impl<T> Ls50ed<T>
where
    T: Transport,
{
    /// Bring the scanner from cold to scannable: self-test, then lamp
    ///
    /// Both move the carriage, so this stops short without film and skips the motion on a
    /// feeder, which would spit its strip back out.
    pub fn warm_up(&mut self) -> Result<(), scsi::Error> {
        if self.wait_settled()? == Status::NoFilm {
            return Err(scsi::Error::InvalidResponse("no film loaded"));
        }
        let feeder = matches!(self.holder(), Ok(Holder::Feeder));
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
    ///
    /// The load counterpart, subcode 0xD1, is rejected on this unit.
    pub fn eject(&mut self) -> Result<(), scsi::Error> {
        self.tolerate_busy(&ReserveUnit::default())?;
        self.tolerate_busy(&VendorWrite::new(VendorPayload::Eject))?;
        self.tolerate_busy(&VendorTrigger)?;
        // The motor runs for several seconds, reporting Ejecting the whole time. Let it
        // settle before handing the scanner back, or the release lands mid-motion.
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
    /// Aim it at the [`center`](ScanSettings::center) of the frame about to be scanned. Other
    /// payloads for this subcode eject. Blocks for the ten or so seconds it takes, and reports
    /// where the setpoint landed.
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
    ///
    /// The caller sets [`set_frame_boundaries`](Self::set_frame_boundaries) then
    /// [`autofocus`](Self::autofocus) or [`set_focus`](Focus::set_focus) first.
    pub fn scan_image(
        &mut self,
        settings: &ScanSettings,
        gain: ChannelExposures,
    ) -> Result<Image, ScanError> {
        self.scan_image_with(settings, gain, |_, _| {})
    }

    /// [`scan_image`](Self::scan_image), reporting (received, expected) bytes as it reads
    pub fn scan_image_with<F: FnMut(u64, u64)>(
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

        self.arm(settings, gain, ScanMode::Normal)?;
        self.scan(channels(settings))?;

        let mut decoder = FrameDecoder::new(settings);
        // The scanner hands back exactly one padded line per read
        let chunk = settings.bytes_per_line() as u32;
        self.read_into_with(&mut decoder, chunk, progress)?;
        let frame = decoder.finish().map_err(ScanError::Decode)?.to_owned();
        debug!(
            width = frame.rgb.width(),
            height = frame.rgb.height(),
            "Image drained"
        );
        Ok(frame)
    }

    /// Arm one pass: gamma tables, windows, window read-back. The caller issues the SCAN.
    fn arm(
        &mut self,
        settings: &ScanSettings,
        gain: ChannelExposures,
        mode: ScanMode,
    ) -> Result<(), scsi::Error> {
        // Refused rather than put on the wire: a multi-sample pass arms and then never streams,
        // so the caller would sit through `read_line`'s idle timeout to be told nothing useful
        if settings.samples > 1 {
            return Err(scsi::Error::Unsupported(
                "multi-sampling arms a pass that never streams",
            ));
        }

        let channels = channels(settings);

        // Identity keeps the output linear. SANE skips the table for an AE pass.
        if mode == ScanMode::Normal {
            let table = identity_gamma_table();
            for &channel in channels {
                self.write_dtc(Dtc::Gamma, Some(channel), table.clone())?;
            }
            debug!("Arm: gamma tables uploaded");
        }

        let mode = match (mode, settings.samples) {
            (ScanMode::Normal, samples) if samples > 1 => ScanMode::Samples(samples),
            (mode, _) => mode,
        };
        for &channel in channels {
            let params = WindowParams {
                mode,
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
    /// Retried: SCAN is rejected with CHECK CONDITION until the lamp and carriage warm up.
    pub fn scan(&mut self, channels: &[Channel]) -> Result<(), scsi::Error> {
        let window_ids: Vec<_> = channels.iter().map(|c| c.to_id()).collect();
        for attempt in 0..MAX_SCAN_ATTEMPTS {
            match self.transport.send(&Scan::new(0, window_ids.clone(), 0x00)) {
                Ok(()) => return Ok(()),
                Err(scsi::Error::Status { sense, .. }) => {
                    debug!(attempt, ?sense, "SCAN busy, retrying");
                    sleep(SCAN_RETRY_PAUSE);
                }
                Err(err) => return Err(err),
            }
        }
        Err(scsi::Error::InvalidResponse("scanner kept rejecting SCAN"))
    }

    /// Walk the adapter-configuration VPD pages the way Nikon Scan does before arming
    ///
    /// Read-only, every result discarded. Kept because the arming sequence was only ever
    /// verified with it in place.
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

    /// Read one padded line, waiting for the carriage to produce it
    ///
    /// Reading mid-positioning aborts the scan, hence the status gate.
    fn read_line(&mut self, want: u32) -> Result<Vec<u8>, scsi::Error> {
        let deadline = Instant::now() + IMAGE_IDLE_TIMEOUT;
        while Instant::now() < deadline {
            if !matches!(self.status(), Ok(Status::Ready)) {
                sleep(IMAGE_IDLE_PAUSE);
                continue;
            }
            match self.read_dtc(Dtc::Image, None, want) {
                Ok(line) => return Ok(line),
                // Flipped back to not-ready between the poll and the read: same line again
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
            "scanner stopped producing image lines",
        ))
    }

    /// Poll until the state is one that waiting can't change
    ///
    /// Returns the state rather than folding it into an error: [`warm_up`](Self::warm_up)
    /// treats `NoFilm` differently from the rest.
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
    ///
    /// Settling elsewhere is not a timeout but a state waiting cannot clear: `NeedsInit` wants
    /// the self-test, `NoFilm` wants film. Debug rather than warn, since arming calls this
    /// speculatively and discards the result.
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
mod tests {
    use super::*;
    use crate::scsi::{DataDirection, Error, SenseData};
    use boundaries::FrameBoundaries;

    /// Samples as their big-endian wire bytes, the way the scanner sends them
    fn be_line(samples: &[u16]) -> Vec<u8> {
        samples.iter().flat_map(|s| s.to_be_bytes()).collect()
    }

    /// The captured page 0xC1 when that is what was asked for, zeroes otherwise
    fn serve_inquiry(cdb: &[u8], data: &mut [u8]) {
        data.fill(0);
        if cdb[1] & 1 == 1 && cdb[2] == 0xC1 {
            let raw = capabilities::fixture::raw_page();
            let n = raw.len().min(data.len());
            data[..n].copy_from_slice(&raw[..n]);
        }
    }

    /// A 1x2 frame, which is what the scripted mock serves: four native dots across and
    /// eight down, read out at a quarter of the optical resolution
    fn settings(ir: bool) -> ScanSettings {
        let capabilities = capabilities::fixture::capabilities();
        ScanSettings {
            dpi: Dpi::_1000,
            ir,
            samples: 1,
            window: TINY,
            capabilities,
        }
    }

    /// Small enough that a whole frame is two lines of one pixel
    const TINY: ScanArea = ScanArea {
        x_pos: 0,
        y_pos: 0,
        x_size: 4,
        y_size: 8,
    };

    /// Scripts a whole pass: SET WINDOW checks the mode byte and exposure, SCAN answers GOOD,
    /// READ hands back one padded line until the image runs out.
    struct ScanMock {
        image: Vec<u8>,
        cursor: usize,
        /// Real bytes in a line; the rest of each read is block padding
        line_len: usize,
        wrote_gamma: bool,
        windows_set: usize,
        /// Mode byte of the last window armed, which says what kind of pass follows
        last_mode: u8,
        /// What the real (non-AE) pass has to carry in SET WINDOW
        expect_exposure: [u32; 3],
        /// What a channel-selected GET WINDOW reports back as measured
        measured_exposure: Option<[u32; 3]>,
        /// The last autofocus payload seen
        autofocus_payload: Option<Vec<u8>>,
    }

    impl ScanMock {
        fn new(image: Vec<u8>, line_len: usize) -> Self {
            Self {
                image,
                cursor: 0,
                line_len,
                wrote_gamma: false,
                windows_set: 0,
                last_mode: 0,
                expect_exposure: {
                    let seed = ChannelExposures::default();
                    [seed.red, seed.green, seed.blue]
                },
                measured_exposure: None,
                autofocus_payload: None,
            }
        }
    }

    impl Transport for ScanMock {
        fn execute(
            &mut self,
            cdb: &[u8],
            _direction: DataDirection,
            data: &mut [u8],
            _sense: &mut [u8],
        ) -> Result<(), Error> {
            match cdb[0] {
                // TEST UNIT READY / RESERVE / RELEASE / SEND DIAGNOSTIC / MODE SELECT /
                // vendor read / vendor trigger
                0x00 | 0x16 | 0x17 | 0x1D | 0x15 | 0xE1 | 0xC1 => Ok(()),
                // Vendor write: keep the autofocus payload for inspection
                0xE0 => {
                    if cdb[2] == 0xA0 {
                        self.autofocus_payload = Some(data.to_vec());
                    }
                    Ok(())
                }
                // INQUIRY, standard and EVPD
                0x12 => {
                    serve_inquiry(cdb, data);
                    Ok(())
                }
                // SEND(10): the gamma table, and the frame boundaries
                0x2A => {
                    if cdb[2] == 0x03 {
                        self.wrote_gamma = true;
                    }
                    Ok(())
                }
                // The mode byte is either a real pass or an AE pass, and a real pass has to
                // carry the exposure under test
                0x24 => {
                    let descriptor = &data[8..];
                    assert!(
                        matches!(descriptor[42], 0x01 | 0x20),
                        "SET WINDOW #{} mode byte {:#04x}",
                        self.windows_set,
                        descriptor[42]
                    );
                    if descriptor[42] == 0x01 {
                        let exposure = u32::from_be_bytes(descriptor[46..50].try_into().unwrap());
                        let expected = self
                            .expect_exposure
                            .get(self.windows_set)
                            .copied()
                            .unwrap_or(0);
                        assert_eq!(exposure, expected, "SET WINDOW #{}", self.windows_set);
                    }
                    self.last_mode = descriptor[42];
                    self.windows_set += 1;
                    Ok(())
                }
                // A real pass must have its gamma table up first, an AE pass deliberately
                // has none. Each pass rewinds, so a strip drains one image per frame.
                0x1B => {
                    assert!(
                        self.wrote_gamma || self.last_mode == 0x20,
                        "SCAN issued before the gamma table"
                    );
                    self.cursor = 0;
                    self.windows_set = 0;
                    Ok(())
                }
                // An 8-byte header plus one 50-byte descriptor; a channel-selected read
                // serves the measured exposure
                0x25 => {
                    data.fill(0);
                    if data.len() >= 58 {
                        data[6..8].copy_from_slice(&50u16.to_be_bytes());
                        if cdb[1] & 1 == 1 {
                            // The descriptor names the window it describes, which is how a
                            // caller tells whether it got the channel it asked for
                            data[8] = cdb[5];
                            if let Some(measured) = self.measured_exposure {
                                let value = measured
                                    .get((cdb[5] as usize).wrapping_sub(1))
                                    .copied()
                                    .unwrap_or(0);
                                data[54..58].copy_from_slice(&value.to_be_bytes());
                            }
                        }
                    }
                    Ok(())
                }
                // READ(10) image data, one line a call
                0x28 => {
                    assert_eq!(cdb[2], 0x00, "unexpected data-type code");
                    if self.cursor >= self.image.len() {
                        return Err(Error::Status {
                            status: 0x02,
                            sense: Some(SenseData {
                                key: 0x0b,
                                asc: 0x3e,
                                ascq: 0x00,
                                ili: false,
                                deferred: false,
                            }),
                        });
                    }
                    let end = (self.cursor + self.line_len).min(self.image.len());
                    let n = end - self.cursor;
                    data[..n].copy_from_slice(&self.image[self.cursor..end]);
                    self.cursor = end;
                    Ok(())
                }
                other => panic!("unexpected opcode {other:#04x}"),
            }
        }
    }

    /// The 1x2 frame [`TINY`] asks for: two lines of one RGB pixel, each plane padded to two
    /// samples, so R at 0, G at 2, B at 4
    fn rgb_image() -> Vec<u8> {
        [be_line(&[1, 0, 1, 0, 1]), be_line(&[2, 0, 2, 0, 2])].concat()
    }

    /// What a caller does per frame
    fn scan_one(scanner: &mut Ls50ed<ScanMock>, settings: &ScanSettings) -> Image {
        scanner.warm_up().unwrap();
        scanner
            .scan_image(settings, ChannelExposures::default())
            .unwrap()
    }

    #[test]
    fn scan_decodes_an_rgb_frame() {
        let mut scanner = Ls50ed::new(ScanMock::new(rgb_image(), 10)).unwrap();
        let frame = scan_one(&mut scanner, &settings(false));
        assert_eq!(frame.rgb.dimensions(), (1, 2));
        assert!(frame.ir.is_none());
        assert_eq!(frame.rgb.get_pixel(0, 0).0, [1u16, 1, 1]);
        assert_eq!(frame.rgb.get_pixel(0, 1).0, [2u16, 2, 2]);
    }

    #[test]
    fn scan_decodes_an_rgbi_frame() {
        // Four planes a line: R G B I, each one sample padded to two
        let image = [
            be_line(&[1, 0, 1, 0, 1, 0, 90]),
            be_line(&[2, 0, 2, 0, 2, 0, 91]),
        ]
        .concat();
        let mut scanner = Ls50ed::new(ScanMock::new(image, 14)).unwrap();
        let frame = scan_one(&mut scanner, &settings(true));
        assert_eq!(frame.rgb.get_pixel(0, 0).0, [1u16, 1, 1]);
        let ir = frame.ir.expect("IR plane captured");
        assert_eq!(ir.dimensions(), (1, 2));
        assert_eq!(ir.get_pixel(0, 0).0, [90u16]);
        assert_eq!(ir.get_pixel(0, 1).0, [91u16]);
    }

    /// A zero-length line makes the read loop exit before it starts, so the SCAN would be left
    /// pending with nothing to drain it and an empty image would come back as a success
    #[test]
    fn a_window_narrower_than_a_pixel_is_refused_before_arming() {
        let mut scanner = Ls50ed::new(ScanMock::new(rgb_image(), 10)).unwrap();
        let settings = ScanSettings {
            window: ScanArea { x_size: 0, ..TINY },
            ..settings(false)
        };

        assert!(matches!(
            scanner.scan_image(&settings, ChannelExposures::default()),
            Err(ScanError::Scsi(scsi::Error::Unsupported(_)))
        ));
    }

    /// Documented as broken rather than enforced, which left the caller waiting out the idle
    /// timeout on a pass that was never going to stream
    #[test]
    fn multi_sampling_is_refused_rather_than_armed() {
        let mut scanner = Ls50ed::new(ScanMock::new(rgb_image(), 10)).unwrap();
        let settings = ScanSettings {
            samples: 2,
            ..settings(false)
        };

        assert!(matches!(
            scanner.scan_image(&settings, ChannelExposures::default()),
            Err(ScanError::Scsi(scsi::Error::Unsupported(_)))
        ));
    }

    #[test]
    fn autoexposure_measures_then_applies() {
        // The AE pass measures, GET WINDOW reports it, and the real pass carries it
        let measured = [1111u32, 2222, 3333];
        let mut mock = ScanMock::new(rgb_image(), 10);
        mock.expect_exposure = measured;
        mock.measured_exposure = Some(measured);

        let mut scanner = Ls50ed::new(mock).unwrap();
        let settings = settings(false);
        scanner.warm_up().unwrap();
        let gain = scanner
            .autoexpose(&settings, ChannelExposures::default())
            .unwrap();
        assert_eq!([gain.red, gain.green, gain.blue], measured);

        let frame = scanner.scan_image(&settings, gain).unwrap();
        assert_eq!(frame.rgb.dimensions(), (1, 2));
    }

    /// Every frame declares the whole strip so the feed keeps advancing, and takes its window
    /// off the table. There is no host feed command: the motor moves because the geometry says.
    #[test]
    fn a_strip_yields_a_frame_per_request() {
        let capabilities = capabilities::fixture::capabilities();
        let boundaries = FrameBoundaries::evenly_spaced(
            3,
            capabilities.frame_pitch,
            &[0.0],
            capabilities.max_x(),
        );
        let mut scanner = Ls50ed::new(ScanMock::new(rgb_image(), 10)).unwrap();
        scanner.warm_up().unwrap();

        for rect in &boundaries.0 {
            let settings = ScanSettings {
                // The frame's own origin, over a window the mock has data for
                window: ScanArea {
                    y_pos: rect.scan_area(capabilities).y_pos,
                    ..TINY
                },
                ..settings(false)
            };
            // Re-declared per pass, the way the hardware was driven
            scanner.set_frame_boundaries(&boundaries).unwrap();
            let frame = scanner
                .scan_image(&settings, ChannelExposures::default())
                .unwrap();
            assert_eq!(frame.rgb.dimensions(), (1, 2));
            assert_eq!(frame.rgb.get_pixel(0, 0).0, [1u16, 1, 1]);
            assert_eq!(frame.rgb.get_pixel(0, 1).0, [2u16, 2, 2]);
        }
    }

    #[test]
    fn autofocus_targets_the_frame_center() {
        // Nothing streams, so this can aim at a whole frame: native (3944, 5956) at 1000 DPI,
        // centered on (1972, 2978)
        let mut scanner = Ls50ed::new(ScanMock::new(rgb_image(), 10)).unwrap();
        let whole_frame = ScanSettings {
            window: ScanArea::frame(0, capabilities::fixture::capabilities()),
            ..settings(false)
        };
        scanner.autofocus(whole_frame.center()).unwrap();

        let mut expected = vec![0u8; 9];
        expected[1..5].copy_from_slice(&1972u32.to_be_bytes());
        expected[5..9].copy_from_slice(&2978u32.to_be_bytes());
        assert_eq!(scanner.transport.autofocus_payload, Some(expected));
    }

    #[test]
    fn a_truncated_pass_is_an_error() {
        // Two lines declared, one delivered
        let mut scanner = Ls50ed::new(ScanMock::new(be_line(&[1, 0, 1, 0, 1]), 10)).unwrap();
        scanner.warm_up().unwrap();
        assert!(matches!(
            scanner.scan_image(&settings(false), ChannelExposures::default()),
            Err(ScanError::Scsi(scsi::Error::InvalidResponse(_)))
        ));
    }

    /// A window past the reported travel asks the feed for room it has not claimed
    #[test]
    fn a_window_past_the_reported_area_is_refused() {
        let capabilities = capabilities::fixture::capabilities();
        let mut scanner = Ls50ed::new(ScanMock::new(rgb_image(), 10)).unwrap();
        let settings = ScanSettings {
            window: ScanArea {
                y_size: capabilities.boundary_y * 2,
                ..ScanArea::frame(0, capabilities)
            },
            ..settings(false)
        };
        assert!(matches!(
            scanner.scan_image(&settings, ChannelExposures::default()),
            Err(ScanError::Scsi(scsi::Error::Unsupported(_)))
        ));
    }

    /// Keeps whatever MODE SELECT was handed
    #[derive(Default)]
    struct RecordingTransport {
        mode_select: Vec<u8>,
    }

    impl Transport for RecordingTransport {
        fn execute(
            &mut self,
            cdb: &[u8],
            _direction: DataDirection,
            data: &mut [u8],
            _sense: &mut [u8],
        ) -> Result<(), Error> {
            match cdb[0] {
                0x00 | 0x16 => Ok(()),
                // INQUIRY: opening reads the capability page before anything else
                0x12 => {
                    serve_inquiry(cdb, data);
                    Ok(())
                }
                0x15 => {
                    self.mode_select = data.to_vec();
                    Ok(())
                }
                other => panic!("unexpected opcode {other:#04x}"),
            }
        }
    }

    /// Assembled from the mode page rather than held as a captured blob, so the wire bytes
    /// have to stay the ones Nikon Scan sends
    #[test]
    fn opening_pins_the_units_with_the_captured_parameter_list() {
        let scanner = Ls50ed::new(RecordingTransport::default()).unwrap();
        assert_eq!(
            scanner.transport.mode_select,
            [
                0x00, 0x00, 0x00, 0x08, // header, block descriptor length 8
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // block descriptor
                0x03, 0x06, 0x00, 0x00, 0x0F, 0xA0, 0x00, 0x00, // page 0x03, 4000 DPI
            ]
        );
    }

    /// Opening reads the geometry off the device rather than assuming it
    #[test]
    fn opening_takes_its_geometry_from_the_device() {
        let scanner = Ls50ed::new(RecordingTransport::default()).unwrap();
        assert_eq!(
            scanner.capabilities(),
            capabilities::fixture::capabilities()
        );
    }

    /// Subcode 0x42 as the scanner answered it with a strip loaded. Byte 3 is a state code,
    /// 0 loaded and 2 ejected, so nothing here reads as a position.
    #[test]
    fn a_vendor_register_comes_back_verbatim() {
        struct MotorState;

        impl Transport for MotorState {
            fn execute(
                &mut self,
                cdb: &[u8],
                _direction: DataDirection,
                data: &mut [u8],
                _sense: &mut [u8],
            ) -> Result<(), Error> {
                match cdb[0] {
                    0x00 | 0x15 | 0x16 => Ok(()),
                    0x12 => {
                        serve_inquiry(cdb, data);
                        Ok(())
                    }
                    0xE1 => {
                        assert_eq!(cdb[2], 0x42);
                        let captured = [
                            0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x58, 0x00,
                            0x00,
                        ];
                        data[..captured.len()].copy_from_slice(&captured);
                        Ok(())
                    }
                    other => panic!("unexpected opcode {other:#04x}"),
                }
            }
        }

        let mut scanner = Ls50ed::new(MotorState).unwrap();
        assert_eq!(
            scanner.probe_vendor(0x42, 13).unwrap(),
            [
                0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x58, 0x00, 0x00
            ]
        );
    }

    /// Page 0x00 advertises 0xC1 on every unit seen, so silence means something is wrong and
    /// hardcoded constants would hide it behind plausible geometry
    #[test]
    fn opening_without_a_capability_page_fails() {
        struct NoCapabilities;

        impl Transport for NoCapabilities {
            fn execute(
                &mut self,
                cdb: &[u8],
                _direction: DataDirection,
                data: &mut [u8],
                _sense: &mut [u8],
            ) -> Result<(), Error> {
                match cdb[0] {
                    0x00 | 0x12 => {
                        data.fill(0);
                        Ok(())
                    }
                    other => panic!("unexpected opcode {other:#04x}"),
                }
            }
        }

        assert!(matches!(
            Ls50ed::new(NoCapabilities),
            Err(scsi::Error::InvalidResponse(_))
        ));
    }
}
