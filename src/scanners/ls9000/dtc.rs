//! Vendor data-type codes
//!
//! READ(10) and SEND(10) address a vendor structure with a data-type code and a 16-bit qualifier.

use super::Ls9000;
use crate::scanners::nikon::Channel;
use crate::scsi::{
    self as scsi, Transport, TransportExt,
    cdbs::{DataTypeCode, Read, Send},
};

/// Vendor DTC reads are framed by a fixed 6-byte header
pub use crate::scanners::nikon::dtc::HEADER_LEN;

/// A vendor data structure on this scanner
///
/// Sizes below are the payload the scanner reported, excluding the 6-byte framing header
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dtc {
    /// Scan parameters, read once a SCAN has been accepted
    ScanParameters,
    /// Frame table, where the frames sit on the loaded film. 20 bytes read, variable written
    FrameBoundaries,
    /// Per-channel dark current, 4 bytes
    DarkCurrent,
    /// Per-channel extended line data, 27 bytes
    ExtendedLine,
    /// Per-channel frame setup, carrying the film-frame name and the gain the window tail uses
    FrameSetup,
    /// Film adapter info
    AdapterInfo,
    /// Something we haven't characterized; qualifier is the caller's problem
    Other { code: u8, qualifier: u8 },
}

impl Dtc {
    pub fn code(self) -> u8 {
        match self {
            Dtc::ScanParameters => 0x87,
            Dtc::FrameBoundaries => 0x88,
            Dtc::DarkCurrent => 0x8C,
            Dtc::ExtendedLine => 0x8D,
            Dtc::FrameSetup => 0x91,
            Dtc::AdapterInfo => 0x93,
            Dtc::Other { code, .. } => code,
        }
    }

    /// The low byte of the qualifier
    pub fn qualifier(self) -> u8 {
        match self {
            Dtc::ScanParameters | Dtc::ExtendedLine => 0x00,
            Dtc::FrameSetup | Dtc::AdapterInfo => 0x01,
            Dtc::FrameBoundaries | Dtc::DarkCurrent => 0x03,
            Dtc::Other { qualifier, .. } => qualifier,
        }
    }

    /// The full 16-bit qualifier. `None` addresses the structure globally, which encodes as channel 0
    pub fn dtq(self, channel: Option<Channel>) -> u16 {
        let channel = channel.map_or(0, Channel::to_id);
        u16::from(channel) << 8 | u16::from(self.qualifier())
    }
}

impl From<Dtc> for DataTypeCode {
    fn from(dtc: Dtc) -> Self {
        DataTypeCode::Vendor(dtc.code())
    }
}

/// READ(10)/SEND(10) plumbing for vendor structures
impl<T> Ls9000<T>
where
    T: Transport,
{
    /// Read `length` bytes of a vendor data structure
    pub(super) fn read_dtc(
        &mut self,
        dtc: Dtc,
        channel: Option<Channel>,
        length: u32,
    ) -> Result<Vec<u8>, scsi::Error> {
        self.transport.send(&Read::new(
            0,
            dtc.into(),
            dtc.dtq(channel),
            length,
            super::VENDOR_CONTROL,
        ))
    }

    /// Read a whole vendor data structure, probing `probe` bytes for the payload length first
    pub(super) fn read_framed_dtc(
        &mut self,
        dtc: Dtc,
        channel: Option<Channel>,
        probe: u32,
    ) -> Result<Vec<u8>, scsi::Error> {
        crate::scanners::nikon::dtc::read_framed(
            &mut self.transport,
            dtc.into(),
            dtc.dtq(channel),
            probe,
            super::VENDOR_CONTROL,
        )
    }

    /// Write a vendor data structure
    pub(super) fn write_dtc(
        &mut self,
        dtc: Dtc,
        channel: Option<Channel>,
        parameters: Vec<u8>,
    ) -> Result<(), scsi::Error> {
        self.transport.send(&Send::new(
            0,
            dtc.into(),
            dtc.dtq(channel),
            parameters,
            0x00,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every DTC/DTQ pair observed across all five captures
    #[test]
    fn matches_every_captured_dtq() {
        let cases = [
            (Dtc::ScanParameters, None, 0x0000),
            (Dtc::FrameBoundaries, None, 0x0003),
            (Dtc::DarkCurrent, Some(Channel::Red), 0x0103),
            (Dtc::DarkCurrent, Some(Channel::Green), 0x0203),
            (Dtc::DarkCurrent, Some(Channel::Blue), 0x0303),
            (Dtc::DarkCurrent, Some(Channel::Ir), 0x0903),
            (Dtc::ExtendedLine, Some(Channel::Red), 0x0100),
            (Dtc::ExtendedLine, Some(Channel::Green), 0x0200),
            (Dtc::ExtendedLine, Some(Channel::Blue), 0x0300),
            (Dtc::FrameSetup, Some(Channel::Red), 0x0101),
            (Dtc::FrameSetup, Some(Channel::Green), 0x0201),
            (Dtc::FrameSetup, Some(Channel::Blue), 0x0301),
            (Dtc::FrameSetup, Some(Channel::Ir), 0x0901),
            (Dtc::AdapterInfo, None, 0x0001),
        ];

        for (dtc, channel, expected) in cases {
            assert_eq!(dtc.dtq(channel), expected, "{dtc:?} on {channel:?}");
        }
    }

    /// SCSI-2 reserves 0x00-0x7F, so nothing here may collide with a spec data-type code
    #[test]
    fn every_code_is_in_the_vendor_range() {
        for dtc in [
            Dtc::ScanParameters,
            Dtc::FrameBoundaries,
            Dtc::DarkCurrent,
            Dtc::ExtendedLine,
            Dtc::FrameSetup,
            Dtc::AdapterInfo,
        ] {
            assert!(dtc.code() >= 0x80, "{dtc:?} is not a vendor code");
            assert_eq!(DataTypeCode::from(dtc), DataTypeCode::Vendor(dtc.code()));
        }
    }

    #[test]
    fn other_passes_code_and_qualifier_through() {
        let dtc = Dtc::Other {
            code: 0x8A,
            qualifier: 0x02,
        };
        assert_eq!(DataTypeCode::from(dtc), DataTypeCode::Vendor(0x8A));
        assert_eq!(dtc.dtq(Some(Channel::Green)), 0x0202);
    }
}
