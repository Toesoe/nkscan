//! READ and SEND data types. Section 2-11

use super::caps::other::DataTypes;

/// The header 2-11-6 puts in front of every type from 80h up
pub const HEADER: usize = 6;

/// One row of table 2-11-2, which both specs give identically for every code
/// they both define
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Row {
    /// Byte 2 of a READ or SEND
    pub code: u8,
    /// Bytes per element. `None` where the caller picks from 1, 2 or 4, or where
    /// neither spec documents one because neither unit implements the type.
    /// Not ours to invent: 2-11 answers common error 1 when the qualifier's low
    /// byte disagrees with this column
    pub width: Option<u8>,
    /// Elements, where 2-11-2 fixes a number rather than saying Variable
    pub count: Option<u32>,
    /// Whether the 6-byte data header precedes the valid data
    pub header: bool,
    /// The `E1h` bit that says a unit will READ it, where the page has one
    pub read: Option<DataTypes>,
    /// The `E1h` bit that says a unit will SEND it
    pub write: Option<DataTypes>,
}

/// Table 2-11-2 in full, so a type either unit implements can be named even
/// where ours does not. Support is never baked in here -- `E1h` decides that at
/// runtime through [`Row::read`] and [`Row::write`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Image,
    HalftoneMask,
    Lut,
    Histogram,
    MaxValue,
    Matrix,
    Filter,
    Shading,
    DarkVoltage,
    Magnetic,
    Cooperation,
    Boundary,
    AnalogGamma,
    AnalogGain,
    DigitalGain,
    WhiteBalanceExposure,
    /// 8Dh, which the 9000 calls Reserved while advertising it in `E1h`
    Setup,
    Perforation,
    Boundary2,
    ShipmentWhiteBalance,
    CcdData,
    DriverVersion,
    LeakVolume,
    RamBuffer,
    EepromBuffer,
}

impl DataType {
    pub const fn row(self) -> Row {
        use DataTypes as D;
        /// Widths and counts for the rows neither spec fills in
        const NONE: (Option<u8>, Option<u32>) = (None, None);
        let (code, (width, count), header, read, write) = match self {
            // 1 or 2 bytes, so the caller picks, and always available
            Self::Image => (0x00, (None, None), false, None, None),
            Self::HalftoneMask => (
                0x02,
                NONE,
                true,
                Some(D::HALFTONE_READ),
                Some(D::HALFTONE_WRITE),
            ),
            Self::Lut => (
                0x03,
                (Some(2), Some(16384)),
                false,
                Some(D::GAMMA_READ),
                Some(D::GAMMA_WRITE),
            ),
            Self::Histogram => (0x80, NONE, true, Some(D::HISTOGRAM_READ), None),
            Self::MaxValue => (
                0x81,
                (Some(2), Some(1)),
                true,
                Some(D::MAX_VALUE_READ),
                None,
            ),
            Self::Matrix => (
                0x82,
                NONE,
                true,
                Some(D::MATRIX_READ),
                Some(D::MATRIX_WRITE),
            ),
            Self::Filter => (
                0x83,
                NONE,
                true,
                Some(D::FILTER_READ),
                Some(D::FILTER_WRITE),
            ),
            Self::Shading => (
                0x84,
                (Some(2), Some(47352)),
                true,
                Some(D::SHADING_READ),
                Some(D::SHADING_WRITE),
            ),
            Self::DarkVoltage => (
                0x85,
                NONE,
                true,
                Some(D::DARK_VOLTAGE_READ),
                Some(D::DARK_VOLTAGE_WRITE),
            ),
            Self::Magnetic => (
                0x86,
                NONE,
                true,
                Some(D::MAGNETIC_READ),
                Some(D::MAGNETIC_WRITE),
            ),
            // 18 elements on the 9000, Variable on the 5000, so left open
            Self::Cooperation => (0x87, (Some(1), None), true, Some(D::COOP_PARAMS_READ), None),
            Self::Boundary => (
                0x88,
                (Some(4), None),
                true,
                Some(D::BOUNDARY_READ),
                Some(D::BOUNDARY_WRITE),
            ),
            Self::AnalogGamma => (0x89, NONE, true, Some(D::ANALOG_GAMMA_READ), None),
            Self::AnalogGain => (
                0x8A,
                (Some(4), Some(2)),
                true,
                Some(D::ANALOG_GAIN_READ),
                None,
            ),
            Self::DigitalGain => (0x8B, NONE, true, Some(D::DIGITAL_GAIN_READ), None),
            Self::WhiteBalanceExposure => {
                (0x8C, (Some(4), Some(1)), true, Some(D::EXPOSURE_READ), None)
            }
            Self::Setup => (
                0x8D,
                (None, None),
                true,
                Some(D::SETUP_READ),
                Some(D::SETUP_WRITE),
            ),
            Self::Perforation => (0x8E, (None, None), true, Some(D::PERFORATION_READ), None),
            Self::Boundary2 => (
                0x8F,
                (None, None),
                true,
                Some(D::BOUNDARY2_READ),
                Some(D::BOUNDARY2_WRITE),
            ),
            Self::ShipmentWhiteBalance => (0x90, NONE, true, Some(D::INITIAL_WB_READ), None),
            Self::CcdData => (0x91, (Some(2), None), true, Some(D::CCD_DATA_READ), None),
            Self::DriverVersion => (
                0x92,
                NONE,
                true,
                Some(D::DRIVER_VERSION_READ),
                Some(D::DRIVER_VERSION_WRITE),
            ),
            Self::LeakVolume => (0x93, (Some(2), Some(3)), true, Some(D::LEAK_READ), None),
            // Both buffers are always there, so E1h has no bit for them
            Self::RamBuffer => (0xE0, (None, None), true, None, None),
            Self::EepromBuffer => (0xE1, (None, None), true, None, None),
        };
        Row {
            code,
            width,
            count,
            header,
            read,
            write,
        }
    }

    /// Whether the qualifier's upper byte names a channel, per 2-11-3
    pub const fn per_color(self) -> bool {
        matches!(
            self,
            Self::Lut
                | Self::Histogram
                | Self::MaxValue
                | Self::Shading
                | Self::DarkVoltage
                | Self::WhiteBalanceExposure
        )
    }

    /// What one element holds. Width alone does not say: the two 4-byte types
    /// differ, and analog gain is IEEE-754 -- `3F800000` reads back as 1.0
    pub const fn scalar(self) -> Scalar {
        match self {
            Self::AnalogGain => Scalar::F32,
            Self::Boundary | Self::WhiteBalanceExposure => Scalar::U32,
            _ => match self.row().width {
                Some(1) => Scalar::U8,
                Some(4) => Scalar::U32,
                _ => Scalar::U16,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scalar {
    U8,
    U16,
    U32,
    F32,
}

/// Valid data, split into elements
///
/// Leak volume arrives as [`Words`](Self::Words) scaled by a million, and
/// boundary information as [`Longs`](Self::Longs) whose first is a length and a
/// frame count rather than a coordinate. Neither is unpacked here
#[derive(Debug, Clone, PartialEq)]
pub enum Values {
    Bytes(Vec<u8>),
    Words(Vec<u16>),
    Longs(Vec<u32>),
    Floats(Vec<f32>),
}

impl Values {
    /// Split `bytes` into elements, dropping any tail too short to fill one
    pub fn decode(scalar: Scalar, bytes: &[u8]) -> Self {
        fn each<const N: usize, T>(bytes: &[u8], f: impl Fn([u8; N]) -> T) -> Vec<T> {
            bytes
                .chunks_exact(N)
                .map(|c| f(c.try_into().expect("chunks_exact")))
                .collect()
        }
        match scalar {
            Scalar::U8 => Self::Bytes(bytes.to_vec()),
            Scalar::U16 => Self::Words(each(bytes, u16::from_be_bytes)),
            Scalar::U32 => Self::Longs(each(bytes, u32::from_be_bytes)),
            Scalar::F32 => Self::Floats(each(bytes, f32::from_be_bytes)),
        }
    }
}

/// What the qualifier's low byte calls an element of `width` bytes, per 2-11-4
pub const fn width_code(width: u8) -> Option<u8> {
    Some(match width {
        1 => 0x00,
        2 => 0x01,
        4 => 0x03,
        _ => return None,
    })
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// The operation parameter SET PARAMETER carries, table 2-15-2
///
/// The table runs to 13 bytes and `C1h` claims 15, but Nikon Scan sends 9 and it
/// works, leaving off the speed, torque and driving method. What the two setting
/// values mean depends on the operation: for AF they are the address to focus
/// on, for a focus move the first is the position
pub struct Operation {
    /// Which channel, where the operation takes one
    pub color: u8,
    pub first: u32,
    pub second: u32,
}

impl Operation {
    pub fn to_bytes(&self) -> [u8; 9] {
        let mut b = [0u8; 9];
        b[0] = self.color;
        b[1..5].copy_from_slice(&self.first.to_be_bytes());
        b[5..9].copy_from_slice(&self.second.to_be_bytes());
        b
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Table 2-11-6
pub struct Header {
    /// Byte 0, echoing the type asked for
    pub code: u8,
    /// Byte 1. Valid bits per element, which can be fewer than the bytes carry:
    /// 14-bit data arrives in 2 bytes and reports 14
    pub bits: u8,
    /// Bytes 2-5. What the unit holds, and it is *not* cut down to match a short
    /// transfer length, so one short read tells us how much to ask for
    pub length: u32,
}

impl Header {
    /// Read the header and return the rest of the slice
    pub fn from_bytes(b: &[u8]) -> Option<(Self, &[u8])> {
        let head = b.get(..HEADER)?;
        Some((
            Self {
                code: head[0],
                bits: head[1],
                length: u32::from_be_bytes([head[2], head[3], head[4], head[5]]),
            },
            &b[HEADER..],
        ))
    }
}
