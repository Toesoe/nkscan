use crate::{
    decode::StreamDecoder,
    scanners::{FilmHolder, Focus, Scanner},
    scsi::{
        self as scsi, SenseKey, Transport, TransportExt,
        cdbs::*,
        mode_pages::{BasicUnit, MeasurementUnits},
    },
};
use cdbs::{Subcode, VendorAbort, VendorPayload, VendorRead, VendorTrigger, VendorWrite};
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

pub use calibration::{ChannelExposures, Metering};
pub use capabilities::Capabilities;
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
/// The most tries a SCAN gets before we call it a refusal. Nikon Scan needs up to four.
const MAX_SCAN_ATTEMPTS: usize = 8;
/// How often [`wait_until_ready`](Ls9000ed::wait_until_ready) asks. Nikon Scan polls at
/// roughly this rate through an autofocus.
pub const POLL_INTERVAL: Duration = Duration::from_millis(200);
/// How long [`wait_until_ready`](Ls9000ed::wait_until_ready) keeps asking before giving up.
/// Long enough for a calibration, which Nikon Scan's own UI says can take two minutes.
const READY_TIMEOUT: Duration = Duration::from_secs(300);

/// Vendor byte 2 of a window descriptor when the window is a frame, see [`WindowKind::Frame`]
const FRAME_WINDOW_KIND: u8 = 0x01;
/// Where a frame SET WINDOW puts the stage motor for a zero-length window
const STAGE_FRAME_ORIGIN: i32 = 7766;

/// Where the gain search of a bare-light pass starts
///
/// Biased low on purpose: a gain that suits film clips on bare light, and a clipped channel can
/// only be halved and tried again, where [`meter`](calibration::meter) lands it from under in
/// one proportional step.
const WHITE_BALANCE_START: u32 = 0x0000_2000;

/// The patch of open aperture a bare-light pass measures
///
/// Mid-travel and deliberately large, 45 mm of stage by the full sensor bar. Holders separate
/// their apertures with opaque bars, in different places per format, and dead center is where a
/// two-aperture 6x9 holder puts one. A span this wide overlaps open aperture in any of them,
/// which is all it needs: [`meter`](calibration::meter) reads the high tail rather than the
/// mean, so a bar across part of the window contributes nothing instead of dragging it dark.
fn white_balance_area() -> ScanArea {
    ScanArea::centered(
        (ScanArea::STRIP_DOTS - WHITE_BALANCE_LENGTH) / 2,
        ScanArea::FILM_WIDTH_DOTS,
        WHITE_BALANCE_LENGTH,
    )
}

/// Stage travel a bare-light pass covers, a whole number of interleave blocks like every
/// other window length
const WHITE_BALANCE_LENGTH: u32 = 7200;

/// Where a frame SET WINDOW drives the stage motor, given the window length
fn stage_target(length: u32) -> i32 {
    STAGE_FRAME_ORIGIN - (length / 2) as i32
}

/// The Nikon LS-9000 ED (Super Coolscan 9000)
pub struct Ls9000ed<T> {
    pub(crate) transport: T,
    capabilities: Capabilities,
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
    pub fn new(mut transport: T) -> Result<Self, scsi::Error> {
        // The geometry this will accept comes from the device
        let capabilities = capabilities::read(&mut transport)?;
        debug!(?capabilities, "Scanner capabilities");
        let mut scanner = Ls9000ed {
            transport,
            capabilities,
        };

        // Everything below would choke on a queued unit attention, and there can be several:
        // ejecting a holder raises both a holder change and a reset.
        let initial_status = scanner.drain_unit_attentions()?;
        debug!(?initial_status, "Scanner state at open");

        // We always want exclusive access for the lifetime of this handle
        scanner.reserve()?;

        // A killed process leaves its pass pending, and the scanner refuses everything else
        // until it is told to give up on it
        scanner.abort_scan()?;

        // On startup, make sure we set the working units to 4000 DPI
        // We will always assume these are the units everywhere (like NikonScan)
        // Without this, SET_WINDOW will fail because we haven't set a unit
        debug!("Setting global units to 4000 DPI");
        scanner.set_global_units()?;
        Ok(scanner)
    }

    /// Throw away a pass nobody is going to read
    ///
    /// A program killed mid-read leaves the scanner holding the rest of the image, and every
    /// later command comes back `CommandSequenceError`. Refusal means nothing was pending.
    pub fn abort_scan(&mut self) -> Result<(), scsi::Error> {
        match self.transport.send(&VendorAbort) {
            Ok(()) => Ok(()),
            Err(err) => {
                trace!(?err, "Nothing to abort");
                Ok(())
            }
        }
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
                divisor: geometry::DOTS_PER_INCH as u16,
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

    /// What the scanner reported about itself when this handle was opened
    pub fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    /// Configure `channel`'s window
    pub fn set_window(
        &mut self,
        channel: Channel,
        mut descriptor: WindowDescriptor,
    ) -> Result<(), scsi::Error> {
        // The kind lives in the vendor tail, which is the only place it survives to
        if descriptor.vendor.get(2) == Some(&FRAME_WINDOW_KIND)
            && descriptor.length > self.capabilities.boundary_y
        {
            warn!(
                length = descriptor.length,
                boundary = self.capabilities.boundary_y,
                stage_target = stage_target(descriptor.length),
                "Refusing a frame window past the reported boundary"
            );
            return Err(scsi::Error::Unsupported(
                "frame window longer than the scanner's reported Y boundary would stall the stage",
            ));
        }

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
    type Transport = T;

    fn transport(&mut self) -> &mut T {
        &mut self.transport
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

/// Either half of a scan can fail: the transport, or decoding what came back
pub type ScanError = crate::scanners::ReadError<decode::DecodeError>;

impl<T> Ls9000ed<T>
where
    T: Transport,
{
    /// Send the film holder back out
    ///
    /// Staged and triggered the same way focus is. The holder leaving raises a holder change
    /// and a reset, so anything reusing this handle afterwards wants
    /// [`drain_unit_attentions`](Self::drain_unit_attentions) first.
    pub fn eject(&mut self) -> Result<(), scsi::Error> {
        debug!("Ejecting the holder");
        self.transport
            .send(&VendorWrite::new(VendorPayload::Eject))?;
        self.transport.send(&VendorTrigger)?;
        Ok(())
    }

    /// Stage the channels this pass needs, scan, and decode what comes back
    ///
    /// `settings.ir` adds the infrared readout, which every capture stages first and lists
    /// first in the SCAN. The decoded mask comes back in [`Image::ir`](decode::Image::ir).
    pub fn scan_image(
        &mut self,
        settings: &ScanSettings,
        gain: ChannelExposures,
    ) -> Result<decode::Image, ScanError> {
        self.scan_image_with(settings, gain, |_, _| {})
    }

    /// [`scan_image`](Self::scan_image), reporting (received, expected) bytes as it reads
    pub fn scan_image_with<F: FnMut(u64, u64)>(
        &mut self,
        settings: &ScanSettings,
        gain: ChannelExposures,
        progress: F,
    ) -> Result<decode::Image, ScanError> {
        let channels: &[Channel] = if settings.ir {
            &[Channel::Ir, Channel::Red, Channel::Green, Channel::Blue]
        } else {
            &Channel::RGB
        };
        for &channel in channels {
            let params = WindowParams {
                ccd: settings.ccd_mode,
                multisample: settings.multisample,
                quality: settings.quality,
                window_kind: WindowKind::Frame,
                exposure: gain.get(channel),
            };
            self.set_window(
                channel,
                params.descriptor(settings.dpi.to_dpi(), settings.window),
            )?;
        }

        self.scan(channels)?;
        self.wait_until_ready()?;

        let chunk = self.transport.max_transfer();
        let mut decoder = decode::FrameDecoder::new(settings).map_err(ScanError::Decode)?;
        self.read_into_with(&mut decoder, chunk, progress)?;

        Ok(decoder.finish().map_err(ScanError::Decode)?.to_owned())
    }

    /// Find the gain that fills the range, by scanning the window and measuring it
    ///
    /// Starts from `from` rather than what the scanner has staged, since gain persists across
    /// sessions and metering relative to it compounds.
    ///
    /// Returns the last image alongside the gain: metering acquires a preview whether anyone
    /// wants it or not, and it is what Nikon Scan shows the user. At least one pass runs.
    pub fn autoexpose(
        &mut self,
        settings: &ScanSettings,
        from: ChannelExposures,
        metering: &Metering,
    ) -> Result<(ChannelExposures, decode::Image), ScanError> {
        self.autoexpose_with(settings, from, metering, |_, _| {})
    }

    /// [`autoexpose`](Self::autoexpose), reporting (received, expected) bytes of each pass
    pub fn autoexpose_with<F: FnMut(u64, u64)>(
        &mut self,
        settings: &ScanSettings,
        from: ChannelExposures,
        metering: &Metering,
        mut progress: F,
    ) -> Result<(ChannelExposures, decode::Image), ScanError> {
        let mut gain = from;
        let mut preview = None;
        for pass in 0..metering.passes.max(1) {
            let image = self.scan_image_with(settings, gain, &mut progress)?;
            let metered = if metering.lock_white_balance {
                calibration::meter_locked(&image.rgb, gain, metering.percentile, metering.target)
            } else {
                calibration::meter(&image.rgb, gain, metering.percentile, metering.target)
            };
            // Only when the pass carried infrared, and never under the white balance lock,
            // which holds the visible channels together and has nothing to say about this one
            let metered = match &image.ir {
                Some(ir) => {
                    calibration::meter_ir(ir, metered, metering.percentile, metering.target)
                }
                None => metered,
            };
            debug!(pass, ?gain, ?metered, "Metered");
            gain = metered;
            preview = Some(image);
        }
        Ok((gain, preview.expect("at least one pass runs")))
    }

    /// The gain that makes the bare backlight read neutral, measured with no film loaded
    ///
    /// Each channel is scaled on its own until the three read alike, so afterwards R=G=B means
    /// "nothing in the light path" and a scan records film transmittance rather than the
    /// scanner's own spectral response.
    ///
    /// A white point and only a white point. Three gains can say what counts as neutral and
    /// nothing else; the LEDs' narrow bands onto real colorimetry needs a 3x3 from a target.
    ///
    /// Pass the result as the `from` gain of [`autoexpose`](Self::autoexpose). Under
    /// [`lock_white_balance`](Metering::lock_white_balance) these ratios are the ones every
    /// later scan preserves.
    ///
    /// Load the holder **empty**. Anything left in the aperture is measured as if it were the
    /// light source, and its cast is what becomes neutral.
    ///
    /// Metered to the same target a frame gets, not something conservative. Bare light is T=1,
    /// so gains that leave it just under the ceiling cannot clip on film, and every later pass
    /// starts unclipped. Driving all three channels to one level also cancels whatever the
    /// response curve does near full scale out of the ratios.
    pub fn white_balance(&mut self, metering: &Metering) -> Result<ChannelExposures, ScanError> {
        self.white_balance_with(metering, |_, _| {})
    }

    /// [`white_balance`](Self::white_balance), reporting (received, expected) bytes of each pass
    pub fn white_balance_with<F: FnMut(u64, u64)>(
        &mut self,
        metering: &Metering,
        progress: F,
    ) -> Result<ChannelExposures, ScanError> {
        // Locking would defeat the point: it scales the channels together, and telling them
        // apart is the entire measurement
        let metering = Metering {
            lock_white_balance: false,
            ..*metering
        };
        let settings = ScanSettings::autoexposure(white_balance_area(), false);
        // The visible channels start low and get metered up. Infrared is not in the preview to
        // measure, and `meter` leaves it alone, so it has to start where it should end up
        // rather than carrying the search's starting point out with it.
        let from = ChannelExposures {
            ir: ChannelExposures::default().ir,
            ..ChannelExposures::flat(WHITE_BALANCE_START)
        };
        let (gain, _) = self.autoexpose_with(&settings, from, &metering, progress)?;
        debug!(?gain, "Bare-light white balance");
        Ok(gain)
    }

    /// [`overview`](Self::overview), reporting (received, expected) bytes as it reads
    pub fn overview_with<F: FnMut(u64, u64)>(
        &mut self,
        gain: ChannelExposures,
        progress: F,
    ) -> Result<decode::Rgb16, ScanError> {
        let channels = Channel::RGB;
        for channel in channels {
            let params = WindowParams {
                ccd: CcdMode::SingleLine,
                multisample: Multisample::X1,
                quality: BaseQuality::Scan,
                window_kind: WindowKind::Overview,
                exposure: gain.get(channel),
            };
            self.set_window(channel, params.descriptor(83, ScanArea::overview()))?;
        }

        self.scan(&channels)?;
        self.wait_until_ready()?;

        let chunk = self.transport.max_transfer();
        let mut decoder = decode::OverviewDecoder::new();
        self.read_into_with(&mut decoder, chunk, progress)?;
        let view = decoder.finish().map_err(ScanError::Decode)?;
        Ok(
            decode::Rgb16::from_raw(view.width(), view.height(), view.to_vec())
                .expect("view is well formed"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scsi::mock::MockTransport;

    /// A transport that can answer the capability read `new` starts with
    fn mock() -> MockTransport {
        MockTransport::new().with_page(0xC1, capabilities::fixture::raw_page())
    }

    /// The order matters and is documented as mattering: the units have to be set before any
    /// SET WINDOW, and the abort has to clear a pass left pending by a killed process before
    /// anything else is attempted.
    #[test]
    fn opening_reads_geometry_then_reserves_aborts_and_sets_units() {
        let scanner = Ls9000ed::new(mock()).expect("opens");
        assert_eq!(
            scanner.transport.opcode_sequence(),
            [
                0x12, // INQUIRY, the capability page
                0x00, // TEST UNIT READY, draining unit attentions
                0x16, // RESERVE
                0xC0, // vendor ABORT
                0x15, // MODE SELECT, the measurement units
            ]
        );
    }

    /// Nikon Scan reads the frame setup before staging any window, and the staging covers
    /// infrared as well as the visible channels
    #[test]
    fn calibrating_reads_frame_setup_before_staging_windows() {
        let mut scanner = Ls9000ed::new(mock()).expect("opens");
        scanner
            .calibrate(ChannelExposures::default())
            .expect("calibrates");

        let sequence = &scanner.transport.opcode_sequence()[..];
        let calibration = &sequence[sequence.len() - 8..];
        assert_eq!(
            calibration,
            [
                0x28, // READ, the frame setup per channel
                0x24, // SET WINDOW per channel
                0xE1, // vendor read, the staged focus
                0xE0, // vendor write, committing it
                0xC1, // vendor TRIGGER
                0x28, // READ, the current frame table
                0x2A, // SEND, the nominal one
                0x28, // READ, the per-channel calibration
            ]
        );

        // One window per channel, infrared included, each naming the channel it stages
        let staged: Vec<u8> = scanner
            .transport
            .cdbs(0x24)
            .iter()
            .map(|cdb| cdb[1])
            .collect();
        assert_eq!(staged.len(), 4);
    }

    /// The scanner refuses SCAN until it is ready and says so with a vendor-specific key, so a
    /// refusal is a reason to wait rather than to give up
    #[test]
    fn scan_retries_a_vendor_specific_refusal() {
        let refusal = || scsi::Error::Status {
            status: 0x02,
            sense: Some(scsi::SenseData {
                key: 0x09,
                asc: 0x80,
                ascq: 0x01,
                ili: false,
                deferred: false,
            }),
        };
        let transport = mock().failing(0x1B, refusal()).failing(0x1B, refusal());
        let mut scanner = Ls9000ed::new(transport).expect("opens");

        scanner.scan(&Channel::RGB).expect("succeeds on the third");
        assert_eq!(scanner.transport.count(0x1B), 3);
    }

    /// Anything other than a vendor-specific refusal is a real error, not something to sit
    /// through eight times
    #[test]
    fn scan_does_not_retry_an_illegal_request() {
        let transport = mock().failing(
            0x1B,
            scsi::Error::Status {
                status: 0x02,
                sense: Some(scsi::SenseData {
                    key: 0x05,
                    asc: 0x24,
                    ascq: 0x00,
                    ili: false,
                    deferred: false,
                }),
            },
        );
        let mut scanner = Ls9000ed::new(transport).expect("opens");

        assert!(scanner.scan(&Channel::RGB).is_err());
        assert_eq!(scanner.transport.count(0x1B), 1);
    }

    /// The bare-light window has to be a window the scanner will actually take, and it is
    /// specified rather than derived, so nothing else checks it
    #[test]
    fn the_white_balance_window_is_scannable() {
        let area = white_balance_area();
        let settings = ScanSettings::autoexposure(area, false);
        assert!(
            settings.output_dims().is_some(),
            "{area:?} does not divide evenly at the metering resolution"
        );

        // Long windows drive the stage backwards into its home stop
        assert!(stage_target(area.y_size) > 0);
        assert!(area.y_pos + area.y_size <= ScanArea::STRIP_DOTS);
    }

    /// Centered on stage travel, which is the whole point of where it sits
    #[test]
    fn the_white_balance_window_is_mid_travel() {
        let area = white_balance_area();
        assert_eq!(area.y_pos + area.y_size / 2, ScanArea::STRIP_DOTS / 2);
    }

    /// Every length measured on hardware, with no residual. 16560 is the one that stalled,
    /// and it is the first of these to come out negative.
    #[test]
    fn stage_targets_match_the_measured_positions() {
        assert_eq!(stage_target(6696), 4418);
        assert_eq!(stage_target(9792), 2870);
        assert_eq!(stage_target(13176), 1178);
        assert!(stage_target(16560) < 0);
    }
}
