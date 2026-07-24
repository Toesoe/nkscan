use crate::scsi::{
    Error as ScsiError, Transport,
    cdbs::*,
    mode_pages::{BasicUnit, MeasurementUnits},
};
use cdbs::*;
use holder::Holder;
use status::Status;
use tracing::*;

pub mod cdbs;
pub mod decode;
pub mod holder;
pub mod status;
pub mod window;

/// This scanner always works in u16 pixels
pub const BITS_PER_PIXEL: usize = 16;
/// This scanner's window descriptors are 50 bytes: 40 standardized bytes plus 10 vendor-specific
const WINDOW_DESCRIPTOR_LEN: u32 = 50;
/// This scanner always defines exactly 5 windows: 0 = all/composite, 1/2/3 = R/G/B, 9 = IR.
const WINDOW_COUNT: u32 = 5;

/// The Nikon LS-9000 ED (Super Coolscan 9000)
///
/// Owning a handle means owning exclusive access: `new()` reserves the
/// scanner unconditionally, so there's no tracked "do we currently hold the
/// reservation" state to get out of sync - every operation can assume it.
pub struct Ls9000ed<T> {
    transport: T,
}

/// Scan settings for one frame
#[derive(Debug, Clone, Copy)]
pub struct ScanSettings {
    /// CCD read mode
    pub ccd_mode: CcdMode,
    /// Include an IR pass for dust removal
    pub ir: bool,
    /// Scan resolution in DPI
    pub dpi: Dpi,
    /// Multisample
    pub multisample: Multisample,
    /// The window in the scanner FoV to actually scan
    pub window: Window,
}

impl ScanSettings {
    /// `None` if the window doesn't divide evenly at this resolution.
    pub fn output_dims(&self) -> Option<(u32, u32)> {
        let k = self.dpi.divisor();
        (self.window.x_size.is_multiple_of(k) && self.window.y_size.is_multiple_of(k))
            .then(|| (self.window.y_size / k, self.window.x_size / k))
    }

    /// CCD lines read per stage position.
    pub fn lines(&self) -> u32 {
        match self.ccd_mode {
            CcdMode::ThreeLine => 3,
            CcdMode::SingleLine => 1,
        }
    }

    /// Readouts emitted per stage position: one RGB triple per multi-sample
    /// repeat, plus a single infrared readout when enabled.
    ///
    /// Infrared is captured once no matter the multi-sample setting.
    pub fn readouts(&self) -> u32 {
        3 * self.multisample.count() + u32::from(self.ir)
    }

    /// Spacing between the CCD's lines, in output columns
    pub fn ccd_block(&self) -> u32 {
        match self.ccd_mode {
            CcdMode::ThreeLine => 12 / self.dpi.divisor(),
            CcdMode::SingleLine => 1,
        }
    }

    /// Stage positions the scan will step through
    pub fn stages(&self) -> Option<u32> {
        self.output_dims().map(|(w, _)| w / self.lines())
    }

    /// Total bytes the scanner will return for this frame
    pub fn expected_bytes(&self) -> Option<u64> {
        let (_, height) = self.output_dims()?;
        let per_stage = u64::from(self.readouts()) * u64::from(height) * u64::from(self.lines());
        Some(2 * u64::from(self.stages()?) * per_stage)
    }
}

/// CCD read mode
#[derive(Debug, Copy, Clone)]
pub enum CcdMode {
    /// Nikon calls this "Super Fine" mode, which may reduce banding at the cost of scan speed
    SingleLine,
    /// Normal three-line simultaneous readout
    ThreeLine,
}

/// Multisampling mode
/// Increasing this value increases scan time for the benefit of decreased noise
/// NOTE: When IR is enabled, the IR channel isn't multisampled
#[derive(Debug, Copy, Clone)]
pub enum Multisample {
    X1,
    X2,
    X4,
    X8,
    X16,
}

impl Multisample {
    /// Number of repeats per stage position
    pub fn count(self) -> u32 {
        match self {
            Multisample::X1 => 1,
            Multisample::X2 => 2,
            Multisample::X4 => 4,
            Multisample::X8 => 8,
            Multisample::X16 => 16,
        }
    }
}

/// DPI mode to read out
/// The scanner nativley operates at 4000 DPI and does firmware-level division to downsample
#[derive(Debug, Copy, Clone)]
pub enum Dpi {
    _4000,
    _2000,
    _1333,
    _333,
}

impl Dpi {
    /// Firmware divisor; scan dpi = 4000/k
    pub fn divisor(self) -> u32 {
        match self {
            Dpi::_4000 => 1,
            Dpi::_2000 => 2,
            Dpi::_1333 => 3,
            Dpi::_333 => 12,
        }
    }
}

#[derive(Debug, Copy, Clone)]
/// A scan window in 1/4000-in dots, matching the sensor's native pitch.
/// Resolution-independent: changing DPI reframes the same physical area.
pub struct Window {
    /// Offset along the sensor bar (0..10_000, 63.5 mm).
    pub x_pos: u32,
    /// Offset along stage travel (0..~34_644, 220 mm, a full 120 strip). This is what selects which frame.
    pub y_pos: u32,
    /// Sensor extent or image HEIGHT. This is like the first dimension in MF film, the 6 in 6x9.
    pub x_size: u32,
    /// Stage extent or image WIDTH. Distinguishes 6x4.5 / 6x6 / 6x9.
    /// This must be a multiple of 36 (one CCD interleave block) at any resolution.
    pub y_size: u32,
}

impl Window {
    /// How long the sensor is, in dots
    pub const SENSOR_DOTS: u32 = 10_000;

    pub fn centred(y_pos: u32, x_size: u32, y_size: u32) -> Self {
        Self {
            x_pos: (Self::SENSOR_DOTS - x_size) / 2,
            y_pos,
            x_size,
            y_size,
        }
    }
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
    fn to_id(self) -> u8 {
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
        // changed") - not a real error, just the device reporting it was
        // reset. Absorb it here via status(), which already treats any
        // NotReady/UnitAttention as a state rather than an Err, so it
        // doesn't surface from reserve() below.
        let initial_status = scanner.status()?;
        debug!(?initial_status, "Scanner state at open");

        // We always want exclusive access for the lifetime of this handle -
        // there's no supported use case for sharing a scanner between
        // initiators - so reserve it once here instead of every operation
        // separately tracking whether it needs to.
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

    // TODO: Remove. Raw MODE SENSE access for probing which pages this scanner actually implements before we've decoded them.
    pub fn mode_sense(
        &mut self,
        pc: PageControl,
        page_code: PageCode,
        allocation_length: u8,
    ) -> Result<ModeSenseResponse, ScsiError> {
        self.transport.send(&ModeSense::new(
            0,
            false,
            pc,
            page_code,
            allocation_length,
            0x00,
        ))
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
    /// setpoint rather than the motor's actual physical position - see
    /// VendorPayload::Focus.
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
}
