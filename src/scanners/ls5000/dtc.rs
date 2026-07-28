//! Data-type codes
//!
//! READ(10) and SEND(10) address a data structure with a data-type code and a 16-bit qualifier
//! whose high byte is the channel. The vendor structures are framed: six bytes of header
//! carrying the payload length, so most reads here go through
//! [`read_framed_dtc`](Ls5000::read_framed_dtc).

use super::Ls5000;
use crate::scanners::nikon::Channel;
use crate::scsi::{
    self as scsi, Transport, TransportExt,
    cdbs::{DataTypeCode, Read},
};

pub use crate::scanners::nikon::dtc::HEADER_LEN;

/// A data structure this scanner addresses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dtc {
    /// The image stream a pass produces
    ///
    /// A spec data-type code rather than a vendor one, and unframed.
    Image,
    /// Scan parameters, readable only while a scan is pending
    ///
    /// SCAN is refused until this has been read, so [`scan`](Ls5000::scan) reads it between
    /// attempts. Payloads look like `NN 09 80 NN`.
    ScanParameters,
    /// Per-channel dark current
    DarkCurrent,
    /// The live transport table, whose length the roll's length decides
    LiveTable,
    /// The roll transport table: where the feeder senses frames along the loaded roll
    RollTable,
    /// Per-channel frame setup
    FrameSetup,
    /// Film adapter info
    AdapterInfo,
    /// Something uncharacterized; the qualifier is the caller's problem
    Other { code: u8, qualifier: u8 },
}

impl Dtc {
    pub fn code(self) -> u8 {
        match self {
            Dtc::Image => 0x00,
            Dtc::ScanParameters => 0x87,
            Dtc::DarkCurrent => 0x8C,
            Dtc::LiveTable => 0x8E,
            Dtc::RollTable => 0x8F,
            Dtc::FrameSetup => 0x91,
            Dtc::AdapterInfo => 0x93,
            Dtc::Other { code, .. } => code,
        }
    }

    /// The low byte of the qualifier
    ///
    /// The capture reads the whole-roll overview at qualifier 0 and every frame scan at 1.
    pub fn qualifier(self) -> u8 {
        match self {
            Dtc::ScanParameters | Dtc::LiveTable => 0x00,
            Dtc::Image | Dtc::FrameSetup | Dtc::AdapterInfo => 0x01,
            Dtc::DarkCurrent | Dtc::RollTable => 0x03,
            Dtc::Other { qualifier, .. } => qualifier,
        }
    }

    /// The full 16-bit qualifier. `None` addresses the structure globally, which encodes as
    /// channel 0.
    pub fn dtq(self, channel: Option<Channel>) -> u16 {
        let channel = channel.map_or(0, Channel::to_id);
        u16::from(channel) << 8 | u16::from(self.qualifier())
    }
}

impl From<Dtc> for DataTypeCode {
    fn from(dtc: Dtc) -> Self {
        match dtc {
            Dtc::Image => DataTypeCode::Image,
            other => DataTypeCode::Vendor(other.code()),
        }
    }
}

/// READ(10) plumbing
///
/// Every command here carries the vendor control byte. There is no writer: the only structure
/// written on this scanner is the roll table, which belongs with the whole-roll workflow.
impl<T> Ls5000<T>
where
    T: Transport,
{
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

    /// Read a whole framed vendor structure, probing for its length first
    pub(super) fn read_framed_dtc(
        &mut self,
        dtc: Dtc,
        channel: Option<Channel>,
    ) -> Result<Vec<u8>, scsi::Error> {
        crate::scanners::nikon::dtc::read_framed(
            &mut self.transport,
            dtc.into(),
            dtc.dtq(channel),
            HEADER_LEN,
            super::VENDOR_CONTROL,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every code and qualifier this driver addresses
    #[test]
    fn matches_every_driven_dtq() {
        let cases = [
            // The frame image stream
            (Dtc::Image, None, DataTypeCode::Image, 0x0001),
            (
                Dtc::ScanParameters,
                None,
                DataTypeCode::Vendor(0x87),
                0x0000,
            ),
            (
                Dtc::DarkCurrent,
                Some(Channel::Red),
                DataTypeCode::Vendor(0x8C),
                0x0103,
            ),
            (
                Dtc::DarkCurrent,
                Some(Channel::Ir),
                DataTypeCode::Vendor(0x8C),
                0x0903,
            ),
            (Dtc::LiveTable, None, DataTypeCode::Vendor(0x8E), 0x0000),
            (Dtc::RollTable, None, DataTypeCode::Vendor(0x8F), 0x0003),
            (
                Dtc::FrameSetup,
                Some(Channel::Red),
                DataTypeCode::Vendor(0x91),
                0x0101,
            ),
            (
                Dtc::FrameSetup,
                Some(Channel::Ir),
                DataTypeCode::Vendor(0x91),
                0x0901,
            ),
            (Dtc::AdapterInfo, None, DataTypeCode::Vendor(0x93), 0x0001),
        ];

        for (dtc, channel, code, dtq) in cases {
            assert_eq!(DataTypeCode::from(dtc), code, "{dtc:?}");
            assert_eq!(dtc.dtq(channel), dtq, "{dtc:?} on {channel:?}");
        }
    }

    /// A vendor code below 0x80 would collide with something the spec has assigned
    #[test]
    fn only_the_image_stream_uses_a_spec_code() {
        assert_eq!(DataTypeCode::from(Dtc::Image), DataTypeCode::Image);
        for dtc in [
            Dtc::ScanParameters,
            Dtc::DarkCurrent,
            Dtc::LiveTable,
            Dtc::RollTable,
            Dtc::FrameSetup,
            Dtc::AdapterInfo,
        ] {
            assert!(dtc.code() >= 0x80, "{dtc:?} is not a vendor code");
        }
    }

    /// A frame scan reads qualifier 1, not 0
    #[test]
    fn the_image_stream_is_qualifier_one() {
        assert_eq!(Dtc::Image.qualifier(), 0x01);
        assert_eq!(Dtc::Image.dtq(None), 0x0001);
    }

    #[test]
    fn other_passes_code_and_qualifier_through() {
        let dtc = Dtc::Other {
            code: 0x92,
            qualifier: 0x03,
        };
        assert_eq!(DataTypeCode::from(dtc), DataTypeCode::Vendor(0x92));
        assert_eq!(dtc.dtq(None), 0x0003);
        assert_eq!(dtc.dtq(Some(Channel::Green)), 0x0203);
    }
}
