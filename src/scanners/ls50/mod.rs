use crate::scanners::nikon::status_usb::UsbStatus as Status;
use crate::scanners::nikon::usb::{POLL_INTERVAL, READY_TIMEOUT, UsbCoolscan, is_not_ready};
use crate::{
    adapter::Adapter,
    decode::{Image, StreamDecoder},
    scanners::{
        FilmHolder, Flow, Focus, Scanner,
        nikon::{
            Channel, ChannelExposures,
            cdbs::{Subcode, VendorTrigger, VendorWrite},
            limits::DeviceLimits,
        },
    },
    scsi::{self as scsi, Transport, TransportExt, cdbs::*},
};
use cdbs::vendor_read_write::{VendorPayload, VendorRead};
use decode::{DecodeError, frame_decoder};
use dtc::Dtc;
use geometry::ScanSettings;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use tracing::*;
use window::{ScanMode, WindowParams};

pub mod adapter;
pub mod boundaries;
pub mod calibration;
pub mod capabilities;
pub mod cdbs;
pub mod decode;
pub mod dtc;
pub mod frame_detection;
pub mod geometry;
pub mod window;

use frame_detection::FrameDetector;

/// For [`UsbTransport::open`](crate::scsi::usb::UsbTransport::open)
pub const VENDOR_ID: u16 = 0x04B0;
/// The LS-50 ED and the LS-5000 ED share 0x04B0 and are told apart here
pub const PRODUCT_ID: u16 = 0x4001;

/// 40 standardized bytes plus 10 vendor
const WINDOW_DESCRIPTOR_LEN: u32 = 50;
/// SCSI-2 leaves control bits 7-6 vendor-specific. Only SET WINDOW needs bit 7 here.
const VENDOR_CONTROL: u8 = 0x80;

/// SCAN answers CHECK CONDITION while the lamp and carriage warm up, so retry it
const MAX_SCAN_ATTEMPTS: usize = 30;
/// Long enough for the lamp to make progress between tries, so the budget covers ~15 s
const SCAN_RETRY_PAUSE: Duration = Duration::from_millis(500);
/// Mid-pass not-ready means the next line isn't out of the carriage yet, not end of data
const IMAGE_IDLE_TIMEOUT: Duration = Duration::from_secs(120);
/// Shorter than the ready poll: a line arrives in tens of milliseconds once the carriage moves
const IMAGE_IDLE_PAUSE: Duration = Duration::from_millis(200);

/// The channels one pass captures
fn channels(settings: &ScanSettings) -> &'static [Channel] {
    if settings.ir {
        &Channel::RGBI
    } else {
        &Channel::RGB
    }
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
pub struct Ls50<T> {
    pub(crate) transport: T,
    capabilities: DeviceLimits,
}

impl<T> Ls50<T>
where
    T: Transport,
{
    /// Open a handle, without moving anything
    ///
    /// Film motion waits for [`warm_up`](Self::warm_up), so read-only callers never spin
    /// the motor.
    pub fn new(mut transport: T) -> Result<Self, scsi::Error> {
        // A scanner still coming up from power-on refuses everything, INQUIRY included
        let coming_up: Status =
            crate::scanners::wait_while_initializing(&mut transport, READY_TIMEOUT, POLL_INTERVAL)?;
        trace!(?coming_up, "Scanner state before the capability read");

        // A cold start queues several unit attentions and everything below would choke on
        // one. Drained before the capability read, so a stray CHECK CONDITION cannot look
        // like a device with no geometry to report.
        let initial_status: Status = crate::scanners::drain_unit_attentions(&mut transport)?;
        debug!(?initial_status, "Scanner state at open");

        // Everything this will accept comes from the device
        let capabilities = capabilities::read(&mut transport)?;
        debug!(?capabilities, "Scanner capabilities");

        let mut scanner = Ls50 {
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

    /// Frames sensed on the loaded strip, 0 for none
    ///
    /// Read fresh every call, since a pass or an eject changes it. Six frames read 6 before
    /// scanning and 1 after any pass, with nothing to say which you are looking at.
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

        let mut is_preview = false;

        if descriptor.length >= self.capabilities.boundary_y * 2 {
            debug!(
                ?descriptor,
                "Setting a window longer than the adapter's reported scan area, assuming a preview pass"
            );
            is_preview = true;
        }
        if !is_preview && descriptor.length > self.capabilities.boundary_y {
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
}

impl<T> Scanner for Ls50<T>
where
    T: Transport,
{
    type Status = Status;
    type Transport = T;

    fn transport(&mut self) -> &mut T {
        &mut self.transport
    }

    /// One line per read, gated on the carriage having produced it
    ///
    /// `want` is a whole padded line. An empty return means the pass ended early, which
    /// [`read_into`](Scanner::read_into) reports as a short stream.
    fn read_chunk(&mut self, want: u32) -> Result<Vec<u8>, scsi::Error> {
        self.read_line(want)
    }
}

impl<T> Focus for Ls50<T>
where
    T: Transport,
{
    fn focus(&mut self) -> Result<u16, scsi::Error> {
        self.read_focus(cdbs::FOCUS_READ_LEN)
    }

    fn set_focus(&mut self, focus: u16) -> Result<(), scsi::Error> {
        self.write_focus(focus)
    }
}

impl<T> UsbCoolscan for Ls50<T> where T: Transport {}

/// Either half of a pass can fail: the transport, or decoding what came back
pub type ScanError = crate::scanners::ReadError<DecodeError>;

/// The scan drive: warm-up, arming, and draining the image
impl<T> Ls50<T>
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
        // The motion below pushes film back out of anything holding it under power, which is
        // every adapter but the mounted-slide one. The SA-21 is a feeder despite the name it
        // reads under, so it has to be on this side of the test.
        let feeder = self.adapter().map(Adapter::is_powered).unwrap_or(false);
        self.tolerate_busy(&ReserveUnit::default())?;
        self.probe_adapter_pages();
        if !feeder {
            self.tolerate_busy(&SendDiagnostic::self_test())?;
            self.tolerate_busy(&VendorWrite::new(VendorPayload::Lamp))?;
            self.tolerate_busy(&VendorTrigger)?;
        }
        self.wait_until_ready()
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
        self.scan_image_with(settings, gain, |_, _| Flow::Continue)
    }

    pub fn preview_roll<F: FnMut(u64, u64) -> Flow>(
        &mut self,
        settings: &ScanSettings,
        progress: F,
    ) -> Result<Image, ScanError> {
        if settings.bytes_per_line() == 0 {
            return Err(scsi::Error::Unsupported("Window too narrow").into());
        }

        self.arm(
            settings,
            ChannelExposures::preview_gain(),
            ScanMode::Preview,
        )?;

        self.scan(channels(settings))?;

        let mut decoder = frame_decoder(settings);

        let chunk = settings.bytes_per_chunk() as u32;
        self.read_into_with(&mut decoder, chunk, progress)?;
        let frame = decoder.finish().map_err(ScanError::Decode)?.to_owned();
        debug!(
            width = frame.rgb.width(),
            height = frame.rgb.height(),
            "Image drained"
        );

        let detector = FrameDetector::default();
        let boundaries = detector.detect_frame_boundaries(&frame);

        Ok(frame)
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

        self.arm(settings, gain, ScanMode::Normal)?;
        self.scan(channels(settings))?;

        let mut decoder = frame_decoder(settings);
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

            let mut desc = params.descriptor(settings, channel);

            if mode == ScanMode::Preview {
                // override descriptor vars
                desc.x_resolution = 97;
                desc.y_resolution = 97;
                desc.width = self.capabilities.max_x();
                desc.length = self.capabilities.preview_roll_length()
            }

            self.set_window(channel, desc)?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanners::{ScanArea, ls50::geometry::Dpi};
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

    /// Answers the capability read that opening starts with
    fn mock() -> crate::scsi::mock::MockTransport {
        crate::scsi::mock::MockTransport::new().with_page(0xC1, capabilities::fixture::raw_page())
    }

    /// A mock that also answers page 0x00 with `codes`, which is how an adapter is recognized
    fn mock_with_pages(codes: &[u8]) -> crate::scsi::mock::MockTransport {
        let mut raw = vec![0x06, 0x00];
        raw.extend_from_slice(&(codes.len() as u16).to_be_bytes());
        raw.extend_from_slice(codes);
        mock().with_page(0x00, raw)
    }

    /// SEND DIAGNOSTIC, the self-test that moves the carriage
    const SELF_TEST: u8 = 0x1D;

    /// Warming up a powered adapter must not move the carriage or strike the lamp
    ///
    /// Both push film back out of whatever is holding it. This is the page 0x00 list a real
    /// LS-50 answers with a genuine SA-21 loaded: it advertises 0x46 and 0xE2, and reading that
    /// as an inert mounted-slide adapter is what made a warm-up eject the strip.
    #[test]
    fn a_strip_feeder_is_not_warmed_up_with_the_carriage() {
        let transport = mock_with_pages(&[
            0x00, 0x01, 0x40, 0x41, 0x46, 0x50, 0x51, 0x60, 0x61, 0xC1, 0xD1, 0xE1, 0xF0, 0xF8,
            0xE2, 0xFB, 0xFC,
        ]);
        let mut scanner = Ls50::new(transport.clone()).expect("opens");

        assert_eq!(
            scanner.adapter().expect("reads the adapter"),
            Adapter::StripFilm
        );
        scanner.warm_up().expect("warms up");

        assert_eq!(
            transport.count(SELF_TEST),
            0,
            "ran the self-test on a feeder"
        );
        assert_eq!(transport.count(0xC1), 0, "triggered the lamp on a feeder");
    }

    /// The counterpart: nothing is holding film, so the motion is what has to happen
    #[test]
    fn a_body_with_no_adapter_does_get_warmed_up() {
        let transport = mock_with_pages(&[0x00, 0x01, 0x40, 0x41, 0xF8, 0xFA, 0xFB, 0xFC]);
        let mut scanner = Ls50::new(transport.clone()).expect("opens");

        assert_eq!(scanner.adapter().expect("reads the adapter"), Adapter::None);
        scanner.warm_up().expect("warms up");

        assert_eq!(transport.count(SELF_TEST), 1);
        assert_eq!(transport.count(0xC1), 1);
    }

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
                    let seed = calibration::DEFAULT_GAIN;
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
    fn scan_one(scanner: &mut Ls50<ScanMock>, settings: &ScanSettings) -> Image {
        scanner.warm_up().unwrap();
        scanner
            .scan_image(settings, calibration::DEFAULT_GAIN)
            .unwrap()
    }

    #[test]
    fn scan_decodes_an_rgb_frame() {
        let mut scanner = Ls50::new(ScanMock::new(rgb_image(), 10)).unwrap();
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
        let mut scanner = Ls50::new(ScanMock::new(image, 14)).unwrap();
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
        let mut scanner = Ls50::new(ScanMock::new(rgb_image(), 10)).unwrap();
        let settings = ScanSettings {
            window: ScanArea { x_size: 0, ..TINY },
            ..settings(false)
        };

        assert!(matches!(
            scanner.scan_image(&settings, calibration::DEFAULT_GAIN),
            Err(ScanError::Scsi(scsi::Error::Unsupported(_)))
        ));
    }

    /// Documented as broken rather than enforced, which left the caller waiting out the idle
    /// timeout on a pass that was never going to stream
    #[test]
    fn multi_sampling_is_refused_rather_than_armed() {
        let mut scanner = Ls50::new(ScanMock::new(rgb_image(), 10)).unwrap();
        let settings = ScanSettings {
            samples: 2,
            ..settings(false)
        };

        assert!(matches!(
            scanner.scan_image(&settings, calibration::DEFAULT_GAIN),
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

        let mut scanner = Ls50::new(mock).unwrap();
        let settings = settings(false);
        scanner.warm_up().unwrap();
        let gain = scanner
            .autoexpose(&settings, calibration::DEFAULT_GAIN)
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
        let mut scanner = Ls50::new(ScanMock::new(rgb_image(), 10)).unwrap();
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
                .scan_image(&settings, calibration::DEFAULT_GAIN)
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
        let mut scanner = Ls50::new(ScanMock::new(rgb_image(), 10)).unwrap();
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
        let mut scanner = Ls50::new(ScanMock::new(be_line(&[1, 0, 1, 0, 1]), 10)).unwrap();
        scanner.warm_up().unwrap();
        assert!(matches!(
            scanner.scan_image(&settings(false), calibration::DEFAULT_GAIN),
            Err(ScanError::Scsi(scsi::Error::InvalidResponse(_)))
        ));
    }

    /// A window past the reported travel asks the feed for room it has not claimed
    #[test]
    fn a_window_past_the_reported_area_is_refused() {
        let capabilities = capabilities::fixture::capabilities();
        let mut scanner = Ls50::new(ScanMock::new(rgb_image(), 10)).unwrap();
        let settings = ScanSettings {
            window: ScanArea {
                y_size: capabilities.boundary_y * 2,
                ..ScanArea::frame(0, capabilities)
            },
            ..settings(false)
        };
        assert!(matches!(
            scanner.scan_image(&settings, calibration::DEFAULT_GAIN),
            Err(ScanError::Scsi(scsi::Error::Unsupported(_)))
        ));
    }

    /// Assembled from the mode page rather than held as a captured blob, so the wire bytes
    /// have to stay the ones Nikon Scan sends
    #[test]
    fn opening_pins_the_units_with_the_captured_parameter_list() {
        let scanner = Ls50::new(mock()).unwrap();
        assert_eq!(
            scanner
                .transport
                .data_out(0x15)
                .expect("MODE SELECT was sent"),
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
        let scanner = Ls50::new(mock()).unwrap();
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

        let mut scanner = Ls50::new(MotorState).unwrap();
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
        assert!(matches!(
            Ls50::new(crate::scsi::mock::MockTransport::new()),
            Err(scsi::Error::InvalidResponse(_))
        ));
    }
}
