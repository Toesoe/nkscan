//! The Nikon LS-5000 ED (Super Coolscan 5000 ED), a 35 mm USB film scanner
//!
//! 16-bit samples, hardware multi-sampling, and a motorized feeder that senses frames along a
//! whole roll and reports where they are.
//!
//! Nothing here has been run against a real unit. See `docs/LS5000_HARDWARE_CHECKLIST.md`.

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
use window::WindowParams;

pub mod adapter;
pub mod boundaries;
pub mod calibration;
pub mod capabilities;
pub mod cdbs;
pub mod decode;
pub mod dtc;
pub mod geometry;
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

impl<T> Focus for Ls5000<T>
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

impl<T> UsbCoolscan for Ls5000<T> where T: Transport {}

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
}

#[cfg(test)]
mod tests;
