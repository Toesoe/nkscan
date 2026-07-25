use crate::{
    decode::StreamDecoder,
    scanners::ReadError,
    scsi::{
        Error as ScsiError, SenseKey, Transport,
        cdbs::*,
        mode_pages::{BasicUnit, MeasurementUnits},
    },
};
use cdbs::{Subcode, VendorPayload, VendorRead, VendorTrigger, VendorWrite};
use dtc::Dtc;
use holder::Holder;
use status::Status;
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
pub use geometry::{CcdMode, Dpi, Multisample, ScanSettings, Window};

/// This scanner always works in u16 pixels
pub const BITS_PER_PIXEL: usize = 16;
/// This scanner's window descriptors are 50 bytes: 40 standardized bytes plus 10 vendor-specific
const WINDOW_DESCRIPTOR_LEN: u32 = 50;
/// This scanner always defines exactly 5 windows: 0 = all/composite, 1/2/3 = R/G/B, 9 = IR.
const WINDOW_COUNT: u32 = 5;

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
    IR,
}

impl Channel {
    pub(crate) fn to_id(self) -> u8 {
        match self {
            Channel::All => 0,
            Channel::Red => 1,
            Channel::Green => 2,
            Channel::Blue => 3,
            Channel::IR => 9,
        }
    }
}

/// The coolscan 9000 is SCSI-only, so we can gate here on scsi backends
impl<T> Ls9000ed<T>
where
    T: Transport,
{
    pub fn new(transport: T) -> Result<Self, ScsiError> {
        let mut scanner = Ls9000ed { transport };

        // The first command issued after SBP-2 login always comes back
        // UNIT ATTENTION (typically 0x3F/0x04, "microcode has been
        // changed"). This is not a real error, just the device reporting it was
        // reset. Absorb it here via status(), which already treats any
        // NotReady/UnitAttention as a state rather than an Err, so it
        // doesn't surface from reserve() below.
        let initial_status = scanner.status()?;
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

    pub fn inquiry(&mut self) -> Result<InquiryResponse, ScsiError> {
        self.transport.send(&Inquiry::new())
    }

    /// Current status/state of the scanner
    pub fn status(&mut self) -> Result<Status, ScsiError> {
        // Unfold "Errors" from the state buffer into normal ok states
        match self.transport.send(&TestUnitReady::new()) {
            Ok(()) => Ok(Status::Ready),
            Err(err) => {
                if let ScsiError::Status {
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

    /// Which film holder, if any, is currently loaded.
    pub fn holder(&mut self) -> Result<Holder, ScsiError> {
        let page = self.transport.send(&VpdInquiry::new(
            Holder::PAGE_CODE,
            Holder::ALLOCATION_LENGTH,
        ))?;
        Holder::from_page(&page).ok_or(ScsiError::InvalidResponse(
            "unrecognized VPD page 0xC8 holder data",
        ))
    }

    /// Gain exclusive access to the scanner
    pub fn reserve(&mut self) -> Result<(), ScsiError> {
        self.transport.send(&ReserveUnit::default())
    }

    /// Release exclusive access to the scanner
    pub fn release(&mut self) -> Result<(), ScsiError> {
        self.transport.send(&ReleaseUnit::default())
    }

    /// Sets the scanner's measurement units mode page (basic unit + divisor).
    ///
    /// NOTE: We will hard-code a set to 4000dpi as then we don't have to do math later
    fn set_measurement_units(&mut self, units: MeasurementUnits) -> Result<(), ScsiError> {
        let header = ModeParameterHeader {
            mode_data_length: 0x00, // reserved for MODE SELECT
            medium_type: 0x00,
            device_specific: 0x00,
            block_descriptor_length: 8,
        };
        let block_descriptor = BlockDescriptor {
            density_code: 0x00,
            number_of_blocks: 0x00,
            block_length: 0x01,
        };

        let mut parameter_list = header.to_bytes().to_vec();
        parameter_list.extend_from_slice(&block_descriptor.to_bytes());
        parameter_list.extend_from_slice(&units.page_bytes());

        self.transport
            .send(&ModeSelect::new(0, true, false, parameter_list, 0x00))
    }

    /// Set our working units to "points" in 4000DPI increments
    fn set_global_units(&mut self) -> Result<(), ScsiError> {
        self.set_measurement_units(MeasurementUnits {
            basic_unit: BasicUnit::Inches,
            divisor: 4000,
        })
    }

    /// Stage a focus target (arbitrary units) and commit it via TRIGGER.
    pub fn set_focus(&mut self, focus: u16) -> Result<(), ScsiError> {
        self.transport
            .send(&VendorWrite::new(VendorPayload::Focus(focus)))?;
        self.transport.send(&VendorTrigger)?;
        Ok(())
    }

    /// Read back the focus value currently staged in firmware. May be a
    /// setpoint rather than the motor's actual physical position
    /// See [`VendorPayload::Focus`].
    pub fn get_focus(&mut self) -> Result<u16, ScsiError> {
        match self.transport.send(&VendorRead::new(Subcode::Focus, 9))? {
            VendorPayload::Focus(focus) => Ok(focus),
            // A VendorRead built with Subcode::Focus always decodes to
            // VendorPayload::Focus - see VendorRead::decode.
            VendorPayload::Preheat => unreachable!(),
        }
    }

    /// `Some(channel)` fetches just that window's descriptor; `None` fetches every window this scanner has defined.
    pub fn get_window(
        &mut self,
        channel: Option<Channel>,
    ) -> Result<Vec<WindowDescriptor>, ScsiError> {
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
            0x80,
        ))
    }

    /// Configure `channel`'s window
    pub fn set_window(
        &mut self,
        channel: Channel,
        mut descriptor: WindowDescriptor,
    ) -> Result<(), ScsiError> {
        descriptor.id = channel.to_id();
        self.transport.send(&SetWindow::new(0, &[descriptor], 0x80))
    }

    /// Scan parameters
    ///
    /// Only readable while a scan is pending: once the pass finishes this returns
    /// `CommandSequenceError`, so [`scan`](Self::scan) reads it as part of its retry rather
    /// than exposing it as a post-scan query.
    pub fn scan_parameters(&mut self) -> Result<Vec<u8>, ScsiError> {
        self.read_framed_dtc(Dtc::ScanParameters, None, dtc::HEADER_LEN)
    }

    /// Stream a scan straight into a decoder, `chunk` bytes at a time
    ///
    /// The decoder says how much to read, so the geometry lives in one place. `chunk` has to
    /// stay under whatever the transport carries in one command; the Linux sg driver rejects
    /// anything past its 32 KiB reserved buffer with EINVAL.
    pub fn read_into<D>(&mut self, decoder: &mut D, chunk: u32) -> Result<(), ReadError<D::Error>>
    where
        D: StreamDecoder,
    {
        let expected = decoder.expected_bytes();
        let mut received = 0u64;

        while received < expected {
            let want = u64::from(chunk).min(expected - received) as u32;
            let bytes =
                self.transport
                    .send(&Read::new(0, DataTypeCode::Image, 0x0000, want, 0x80))?;
            if bytes.is_empty() {
                return Err(ScsiError::InvalidResponse(
                    "image read returned nothing before the expected length",
                )
                .into());
            }
            received += bytes.len() as u64;
            trace!(got = bytes.len(), received, expected, "Image chunk");
            decoder.push(&bytes).map_err(ReadError::Decode)?;
        }
        Ok(())
    }

    /// Read `length` bytes of image data, `chunk` bytes at a time
    ///
    /// Buffers the whole transfer, so this is for small passes and raw dumps. Anything
    /// full-resolution wants [`read_into`](Self::read_into) instead.
    ///
    /// The scanner streams: each READ returns the next chunk, so this is just repeated reads.
    /// `chunk` has to stay under whatever the transport will carry in one command (the Linux sg
    /// driver rejects anything past its 32 KiB reserved buffer with EINVAL) and should be a whole
    /// number of output lines.
    pub fn read_image(&mut self, length: u32, chunk: u32) -> Result<Vec<u8>, ScsiError> {
        let mut image = Vec::with_capacity(length as usize);
        while (image.len() as u32) < length {
            let want = chunk.min(length - image.len() as u32);
            let bytes =
                self.transport
                    .send(&Read::new(0, DataTypeCode::Image, 0x0000, want, 0x80))?;
            if bytes.is_empty() {
                return Err(ScsiError::InvalidResponse(
                    "image read returned nothing before the expected length",
                ));
            }
            trace!(
                got = bytes.len(),
                total = image.len() + bytes.len(),
                "Image chunk"
            );
            image.extend_from_slice(&bytes);
        }
        Ok(image)
    }

    /// Trigger a scan using the given channels' previously-configured windows
    ///
    /// The scanner often rejects the first SCAN with a vendor sense code (0x80/0x01) and only
    /// accepts it once the scan parameters have been read. Nikon Scan does the same
    /// read-and-retry, and the payload it gets back is sometimes all zeros, so it's the read
    /// itself that clears the condition rather than anything in it.
    pub fn scan(&mut self, channels: &[Channel]) -> Result<(), ScsiError> {
        let window_ids: Vec<_> = channels.iter().map(|c| c.to_id()).collect();

        match self.transport.send(&Scan::new(0, window_ids.clone(), 0x00)) {
            Err(ScsiError::Status {
                sense: Some(sense), ..
            }) if sense.sense_key() == SenseKey::VendorSpecific => {
                debug!(
                    ?sense,
                    "SCAN rejected, reading scan parameters and retrying"
                );
                let parameters =
                    self.read_framed_dtc(Dtc::ScanParameters, None, dtc::HEADER_LEN)?;
                trace!(?parameters, "Scan parameters");
                self.transport.send(&Scan::new(0, window_ids, 0x00))
            }
            other => other,
        }
    }
}
