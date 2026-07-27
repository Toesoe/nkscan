//! Platform-agnostic SCSI interactions following the SCSI-2 scanner specification

use std::{fmt, io};

pub mod asc;
pub mod cdbs;
pub(crate) mod fields;
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(test)]
pub mod mock;
pub mod mode_pages;
pub mod usb;
#[cfg(target_os = "windows")]
pub mod windows;

/// A SCSI command descriptor block
pub struct Cdb<const N: usize>(pub [u8; N]);

impl<const N: usize> AsRef<[u8]> for Cdb<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DataDirection {
    None,
    Read,
    Write,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SCSI transport error: {0}")]
    Transport(#[from] io::Error),

    #[error(
        "SCSI command failed with status 0x{status:02x}: {}",
        sense.as_ref().map_or_else(|| "no sense data".to_string(), ToString::to_string)
    )]
    Status {
        status: u8,
        sense: Option<SenseData>,
    },

    /// The command never reached the device, or the bus faulted carrying it. Distinct from
    /// `Status`, which means the device answered and had something to say.
    #[error("SCSI host adapter reported status 0x{status:02x}")]
    HostAdapter { status: u16 },

    #[error("invalid SCSI response: {0}")]
    InvalidResponse(&'static str),

    /// Refused before reaching the bus, because the device reports it cannot do it
    #[error("refused to send a command outside what the device supports: {0}")]
    Unsupported(&'static str),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// SPC-2 Table 69/70: the sense key's top-level category for a CHECK CONDITION
/// This is just a coarse status, ASC/ASCQ give the actual detail
pub enum SenseKey {
    NoSense,
    RecoveredError,
    NotReady,
    MediumError,
    HardwareError,
    IllegalRequest,
    UnitAttention,
    DataProtect,
    BlankCheck,
    VendorSpecific,
    CopyAborted,
    AbortedCommand,
    Equal,
    VolumeOverflow,
    Miscompare,
    /// 0Fh is reserved
    Reserved,
}

impl SenseKey {
    fn from_nibble(nibble: u8) -> Self {
        match nibble & 0x0F {
            0x0 => SenseKey::NoSense,
            0x1 => SenseKey::RecoveredError,
            0x2 => SenseKey::NotReady,
            0x3 => SenseKey::MediumError,
            0x4 => SenseKey::HardwareError,
            0x5 => SenseKey::IllegalRequest,
            0x6 => SenseKey::UnitAttention,
            0x7 => SenseKey::DataProtect,
            0x8 => SenseKey::BlankCheck,
            0x9 => SenseKey::VendorSpecific,
            0xA => SenseKey::CopyAborted,
            0xB => SenseKey::AbortedCommand,
            0xC => SenseKey::Equal,
            0xD => SenseKey::VolumeOverflow,
            0xE => SenseKey::Miscompare,
            _ => SenseKey::Reserved,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct SenseData {
    /// Raw sense key nibble; see `sense_key()` for the decoded form
    pub key: u8,
    pub asc: u8,
    pub ascq: u8,
    /// Incorrect Length Indicator (SPC-4 4.5.3): set when the actual transfer length didn't match what was requested
    pub ili: bool,
    /// True for a deferred error (response code 71h/73h) rather than a current one (70h/72h)
    /// the error relates to a command that already completed, not the one that returned this sense data.
    pub deferred: bool,
}

impl SenseData {
    /// The decoded sense key (SPC-2 Table 69/70)
    pub fn sense_key(&self) -> SenseKey {
        SenseKey::from_nibble(self.key)
    }

    /// The decoded additional sense code/qualifier (SPC-2 Table 71), for
    /// logging or matching in error handling instead of raw ASC/ASCQ bytes.
    pub fn condition(&self) -> asc::AdditionalSenseCode {
        asc::AdditionalSenseCode::from_asc_ascq(self.asc, self.ascq)
    }

    /// Parse sense data, as returned by any transport's sense buffer
    ///
    /// SCSI defines two independent sense data layouts, distinguished by the
    /// response code in the low 7 bits of byte 0 (SPC-4 4.5 "Sense data"):
    ///   - 70h/71h (current/deferred): fixed format, 4.5.3
    ///   - 72h/73h (current/deferred): descriptor format, 4.5.2
    ///
    /// A transport is free to hand back either, so both must be handled
    /// rather than assuming fixed format.
    ///
    /// Returns `None` if the response code is unrecognized, or `sense` is too
    /// short to hold the fields its format requires.
    pub(crate) fn parse(sense: &[u8]) -> Option<Self> {
        let response_code = *sense.first()? & 0x7f;
        match response_code {
            // Fixed format: SENSE KEY at byte 2 (low nibble; ILI is bit 5 of
            // the same byte), ASC/ASCQ at bytes 12/13 following SPC-4 4.5.3.
            0x70 | 0x71 => {
                if sense.len() < 14 {
                    return None;
                }
                Some(Self {
                    key: sense[2] & 0x0f,
                    asc: sense[12],
                    ascq: sense[13],
                    ili: sense[2] & 0x20 != 0,
                    deferred: response_code == 0x71,
                })
            }
            // Descriptor format: SENSE KEY at byte 1 (low nibble), ASC/ASCQ
            // at bytes 2/3 following SPC-4 4.5.2.
            0x72 | 0x73 => {
                if sense.len() < 4 {
                    return None;
                }
                Some(Self {
                    key: sense[1] & 0x0f,
                    asc: sense[2],
                    ascq: sense[3],
                    ili: false,
                    deferred: response_code == 0x73,
                })
            }
            _ => None,
        }
    }
}

impl fmt::Display for SenseData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {:?}", self.sense_key(), self.condition())
    }
}

impl fmt::Debug for SenseData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SenseData")
            .field("sense_key", &self.sense_key())
            .field("condition", &self.condition())
            .field("ili", &self.ili)
            .field("deferred", &self.deferred)
            .finish()
    }
}

/// The data phase of a SCSI command
#[derive(Debug, Clone, Copy)]
pub enum CommandData<'a> {
    /// No data transfer
    None,
    /// Host reads this many bytes from the device
    Read(usize),
    /// Host writes these bytes to the device
    Write(&'a [u8]),
}

impl CommandData<'_> {
    fn direction(&self) -> DataDirection {
        match self {
            CommandData::None => DataDirection::None,
            CommandData::Read(_) => DataDirection::Read,
            CommandData::Write(_) => DataDirection::Write,
        }
    }
}

pub trait Command {
    type Response;
    type Cdb: AsRef<[u8]>;

    /// Build the command descriptor block
    fn cdb(&self) -> Self::Cdb;

    /// Data phase of this command: whether it reads, writes, or transfers no data
    fn data(&self) -> CommandData<'_>;

    /// Decode the returned bytes
    fn parse_response(&self, data: &[u8]) -> Result<Self::Response, Error>;
}

/// Default sense buffer size, shared across transports
/// Matches the Linux kernel's own `SCSI_SENSE_BUFFERSIZE`
const SENSE_BUFFER_LEN: usize = 96;

/// Somewhere to send a CDB and get bytes back
pub trait Transport {
    fn execute(
        &mut self,
        cdb: &[u8],
        direction: DataDirection,
        data: &mut [u8],
        sense: &mut [u8],
    ) -> Result<(), Error>;

    /// Largest single transfer this transport can carry
    ///
    /// What a caller reading an image should chunk at. The default is the 32 KB a Linux sg
    /// device starts with, which any transport that knows better should override.
    fn max_transfer(&self) -> u32 {
        32 * 1024
    }
}

/// The convenience layer over [`Transport::execute`], which every transport gets for free
pub trait TransportExt: Transport {
    /// Run one command through its data phase and decode the response
    fn send<C: Command>(&mut self, command: &C) -> Result<C::Response, Error> {
        let cdb = command.cdb();
        let mut sense = [0u8; SENSE_BUFFER_LEN];
        let payload = command.data();
        let mut data = match payload {
            CommandData::None => Vec::new(),
            CommandData::Read(len) => vec![0; len],
            CommandData::Write(bytes) => bytes.to_vec(),
        };
        self.execute(cdb.as_ref(), payload.direction(), &mut data, &mut sense)?;
        match payload {
            CommandData::None | CommandData::Write(_) => command.parse_response(&[]),
            CommandData::Read(_) => command.parse_response(&data),
        }
    }

    /// Fetch and decode a Vital Product Data page
    fn vpd<P: cdbs::VendorPage>(&mut self) -> Result<P, Error> {
        let page = self.send(&cdbs::VpdInquiry::new(P::PAGE_CODE, P::ALLOCATION_LENGTH))?;
        P::from_page(&page).ok_or(Error::InvalidResponse("unrecognized VPD page contents"))
    }

    /// Fetch and decode a mode page
    fn mode_page<P: mode_pages::ModePage>(&mut self) -> Result<P, Error> {
        let response = self.send(&cdbs::ModeSense::new(
            0,
            false,
            cdbs::PageControl::Current,
            cdbs::PageCode::Page(P::PAGE_CODE),
            P::allocation_length(),
            0x00,
        ))?;
        P::from_response(&response).ok_or(Error::InvalidResponse("unrecognized mode page contents"))
    }

    /// Write a mode page back. `block_descriptor` is device-specific; most want `None`.
    fn set_mode_page<P: mode_pages::ModePage>(
        &mut self,
        page: &P,
        block_descriptor: Option<cdbs::BlockDescriptor>,
    ) -> Result<(), Error> {
        let header = cdbs::ModeParameterHeader {
            mode_data_length: 0x00, // reserved for MODE SELECT
            medium_type: 0x00,
            device_specific: 0x00,
            block_descriptor_length: if block_descriptor.is_some() { 8 } else { 0 },
        };

        let mut parameters = header.to_bytes().to_vec();
        if let Some(descriptor) = block_descriptor {
            parameters.extend_from_slice(&descriptor.to_bytes());
        }
        parameters.extend_from_slice(&page.page_bytes());

        self.send(&cdbs::ModeSelect::new(0, true, false, parameters, 0x00))
    }
}

impl<T: Transport + ?Sized> TransportExt for T {}

impl<T: Transport + ?Sized> Transport for &mut T {
    fn execute(
        &mut self,
        cdb: &[u8],
        direction: DataDirection,
        data: &mut [u8],
        sense: &mut [u8],
    ) -> Result<(), Error> {
        (**self).execute(cdb, direction, data, sense)
    }

    fn max_transfer(&self) -> u32 {
        (**self).max_transfer()
    }
}

impl<T: Transport + ?Sized> Transport for Box<T> {
    fn execute(
        &mut self,
        cdb: &[u8],
        direction: DataDirection,
        data: &mut [u8],
        sense: &mut [u8],
    ) -> Result<(), Error> {
        (**self).execute(cdb, direction, data, sense)
    }

    fn max_transfer(&self) -> u32 {
        (**self).max_transfer()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{scanners::ls9000ed::Ls9000ed, scsi::cdbs::TestUnitReady};

    /// Picking a backend at runtime has to compile. These never run.
    #[test]
    fn a_boxed_transport_still_works() {
        fn _sends(mut transport: Box<dyn Transport>) -> Result<(), Error> {
            transport.send(&TestUnitReady::new())
        }
        fn _drives_a_scanner(
            transport: Box<dyn Transport>,
        ) -> Result<Ls9000ed<Box<dyn Transport>>, Error> {
            Ls9000ed::new(transport)
        }
    }
}
