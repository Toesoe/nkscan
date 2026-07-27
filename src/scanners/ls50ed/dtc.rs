//! Data-type codes
//!
//! READ(10) and SEND(10) address a data structure with a data-type code and a 16-bit qualifier
//! whose high byte is the channel. Two of these are SCSI-2 codes rather than vendor ones, hence
//! the per-variant mapping to [`DataTypeCode`].

use super::{Channel, Ls50ed};
use crate::scsi::{
    self as scsi, Transport, TransportExt,
    cdbs::{DataTypeCode, Read, Send},
};

/// A data structure this scanner addresses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dtc {
    /// The image stream a pass produces
    Image,
    /// Per-channel gamma table, 16384 big-endian words
    Gamma,
    /// Frame table, where the frames sit on the loaded film
    ///
    /// The reverse-engineered firmware calls 0x88 per-channel calibration, not frame positions.
    /// Against that: the write is accepted, and dropping it blackens frames after the first.
    FrameBoundaries,
    /// Something uncharacterized; the qualifier is the caller's problem
    ///
    /// 0x92 echoes four bytes at qualifier 3 and moves nothing; 0x8E refuses every read.
    Other { code: u8, qualifier: u8 },
}

impl Dtc {
    /// The low byte of the qualifier
    pub fn qualifier(self) -> u8 {
        match self {
            Dtc::Image => 0x00,
            Dtc::Gamma => 0x01,
            Dtc::FrameBoundaries => 0x03,
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
            Dtc::Gamma => DataTypeCode::GammaFunction,
            Dtc::FrameBoundaries => DataTypeCode::Vendor(0x88),
            Dtc::Other { code, .. } => DataTypeCode::Vendor(code),
        }
    }
}

/// READ(10)/SEND(10) plumbing
///
/// Both carry a plain `0x00` control byte: only SET WINDOW needs bit 7 on this model.
impl<T> Ls50ed<T>
where
    T: Transport,
{
    pub(super) fn read_dtc(
        &mut self,
        dtc: Dtc,
        channel: Option<Channel>,
        length: u32,
    ) -> Result<Vec<u8>, scsi::Error> {
        self.transport
            .send(&Read::new(0, dtc.into(), dtc.dtq(channel), length, 0x00))
    }

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

    /// Every code and qualifier the scanner has been driven with
    #[test]
    fn matches_every_driven_dtq() {
        let cases = [
            (Dtc::Image, None, DataTypeCode::Image, 0x0000),
            (
                Dtc::Gamma,
                Some(Channel::Red),
                DataTypeCode::GammaFunction,
                0x0101,
            ),
            (
                Dtc::Gamma,
                Some(Channel::Green),
                DataTypeCode::GammaFunction,
                0x0201,
            ),
            (
                Dtc::Gamma,
                Some(Channel::Blue),
                DataTypeCode::GammaFunction,
                0x0301,
            ),
            (
                Dtc::Gamma,
                Some(Channel::Ir),
                DataTypeCode::GammaFunction,
                0x0901,
            ),
            (
                Dtc::FrameBoundaries,
                None,
                DataTypeCode::Vendor(0x88),
                0x0003,
            ),
        ];

        for (dtc, channel, code, dtq) in cases {
            assert_eq!(DataTypeCode::from(dtc), code, "{dtc:?}");
            assert_eq!(dtc.dtq(channel), dtq, "{dtc:?} on {channel:?}");
        }
    }

    #[test]
    fn other_passes_code_and_qualifier_through() {
        let motor = Dtc::Other {
            code: 0x92,
            qualifier: 0x03,
        };
        assert_eq!(DataTypeCode::from(motor), DataTypeCode::Vendor(0x92));
        assert_eq!(motor.dtq(None), 0x0003);

        let focus = Dtc::Other {
            code: 0x8E,
            qualifier: 0x00,
        };
        assert_eq!(focus.dtq(Some(Channel::Green)), 0x0200);
    }
}
