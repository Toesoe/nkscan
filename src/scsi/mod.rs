//! Platform-agnostic SCSI interactions

use std::{fmt, io};

pub mod cdbs;
pub mod linux;

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

    #[error("SCSI command failed with status 0x{status:02x}")]
    Status {
        status: u8,
        sense: Option<SenseData>,
    },

    #[error("invalid SCSI response: {0}")]
    InvalidResponse(&'static str),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SenseData {
    pub key: u8,
    pub asc: u8,
    pub ascq: u8,
}

impl SenseData {
    /// Parse fixed-format sense data (SPC-4), as returned by any transport's sense buffer.
    /// Returns `None` if `sense` is too short to hold the fields we read.
    pub(crate) fn parse(sense: &[u8]) -> Option<Self> {
        if sense.len() < 14 {
            return None;
        }
        Some(Self {
            key: sense[2] & 0x0f,
            asc: sense[12],
            ascq: sense[13],
        })
    }
}

impl fmt::Display for SenseData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "sense key=0x{:02x}, asc=0x{:02x}, ascq=0x{:02x}",
            self.key, self.asc, self.ascq
        )
    }
}

pub trait Command {
    type Response;
    type Cdb: AsRef<[u8]>;

    /// Build the command descriptor block
    fn cdb(&self) -> Self::Cdb;

    /// Direction of data transfer
    fn direction(&self) -> DataDirection;

    /// Size of the expected data buffer
    fn data_length(&self) -> usize;

    /// Decode the returned bytes
    fn decode(&self, data: &[u8]) -> Result<Self::Response, Error>;
}

/// Default sense buffer size, shared across transports. Matches the Linux
/// kernel's own `SCSI_SENSE_BUFFERSIZE`, which is large enough for
/// descriptor-format sense data with several descriptors, not just the
/// 18-byte fixed-format minimum that [`SenseData::parse`] reads.
const SENSE_BUFFER_LEN: usize = 96;

pub trait Transport {
    fn execute(
        &mut self,
        cdb: &[u8],
        direction: DataDirection,
        data: &mut [u8],
        sense: &mut [u8],
    ) -> Result<(), Error>;

    fn send<C: Command>(&mut self, command: &C) -> Result<C::Response, Error> {
        let cdb = command.cdb();
        let mut data = vec![0; command.data_length()];
        let mut sense = [0u8; SENSE_BUFFER_LEN];
        self.execute(cdb.as_ref(), command.direction(), &mut data, &mut sense)?;
        command.decode(&data)
    }
}
