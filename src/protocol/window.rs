//! Data around the GET/SET WINDOW commands. Section 2-10

use super::caps::set_window::{BitDepth, ColorInterleaving, ScanKind, ScanMode};
use crate::{error::Error, protocol::caps::Capabilities};
use bitflags::bitflags;
use tracing::*;

/// How many bytes one descriptor occupies, per table 2-10-3
pub const LENGTH: usize = 50;

/// The infrared channel's window identifier
///
/// This one is not mentioned in either spec and was REed
pub const IR: u8 = 9;

/// The default color, which is green: 2-11-3 gives qualifier 00h as the
/// G-component. 2-7 will only scan it on its own
pub const DEFAULT: u8 = 0;

/// Byte 26 of a descriptor vs byte 10 of `D1h`, deepest first so a search for what a unit offers finds the best of them
const DEPTHS: [(u8, BitDepth); 6] = [
    (16, BitDepth::BIT_16),
    (14, BitDepth::BIT_14),
    (12, BitDepth::BIT_12),
    (10, BitDepth::BIT_10),
    (8, BitDepth::BIT_8),
    (1, BitDepth::BIT_1),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    /// Window identifier, 0,1,2,3,9 (Default 0 is the same as green 2). Byte 0.
    /// 2-10-4 stops at 4 (neutral gray, unsupported); 9 is infrared, which only
    /// the hardware admits to
    pub id: u8,
    /// Byte 1 bit 0
    pub auto: bool,
    /// Resolution in DPI (X/Y). Bytes 2-5
    ///
    /// Only X is a setting. SET WINDOW ignores Y outright and GET WINDOW answers
    /// it with whatever X is, so the two can never actually differ.
    ///
    /// An X the unit cannot do is rounded to `optical / round(optical / asked)`
    /// and reported with `01h-37h-00h`
    pub resolution: (u16, u16),
    /// Window origin (upper left) of X and Y. Bytes 6-13
    ///
    /// The wire unit is inches times the measurement unit divisor, but
    /// [`Session::open`] pins that to the maximum resolution, so these are pixels
    pub origin: (u32, u32),
    /// Size of the window, in the same pixels as [`origin`].
    /// Bounded by `C1h`'s boundary, which the power-on descriptor exceeds. Bytes 14-21
    pub size: (u32, u32),
    /// Brightness. Byte 22
    pub brightness: u8,
    /// Threshold. Byte 23
    pub threshold: u8,
    /// Contrast. Byte 24
    pub contrast: u8,
    /// Image composition. Byte 25
    pub composition: Composition,
    /// Pixel composition (bit depth). Byte 26
    pub bpp: u8,
    /// Halftone pattern. Bytes 27-28
    pub halftone_pattern: u16,
    /// Reverse in bit 7, padding type in the low bits. Byte 29
    pub padding_type: u8,
    /// Bit ordering: Byte 30-31
    pub bit_ordering: u16,
    /// Compression type. Byte 32
    pub compression_type: u8,
    /// Compression argument. Byte 33
    pub compression_argument: u8,
    /// One less than the number of times each line is read, so 0 is a single ordinary pass. Byte 40 high nibble
    pub multiple_reading: u8,
    /// What order to read this window's color in: 0 asks for the unit's own ordering, R=1, G=2, B=3. Byte 40 low nibble
    ///
    /// Across a window set this must be all-zero or all-nonzero with no repeats, or SCAN answers `05h-2Ch-02h`
    pub color_ordering: u8,
    /// Byte 41
    pub flags: Flags,
    /// Setup mode. Byte 41 bits 3-1
    ///
    /// Only meaningful when `D1h` advertises [`ScanKind::SETUP_2`], which caps it with the setup-mode count in byte 11 of that page
    pub setup_mode: u8,
    /// Byte 42. `D1h` reports which of these the unit will do, so a selection is checkable against it with `contains`
    pub scanning_kind: ScanKind,
    /// Byte 43, likewise checkable against `D1h`
    pub scanning_mode: ScanMode,
    /// Byte 44, likewise checkable against `D1h`
    pub color_interleaving: ColorInterleaving,
    /// Target output value for auto exposure. Byte 45, default 255. Sending 0 sets 255, and GET WINDOW then reports 255
    pub ae_value: u8,
    /// Integration time in units of 10 ns, up to `3FFFFFFh`. Bytes 46-49
    pub exposure: u32,
}

bitflags! {
    /// Byte 41
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Flags: u8 {
        /// Average along the scan line. This is binning across the sensor bar,
        /// not repeated reads of a line -- that is `multiple_reading`
        const AVERAGING = 1 << 7;
        const MATRIX    = 1 << 6;
        const FILTER    = 1 << 5;
        // bits 3-1 are the setup mode, kept as a field
        /// Set for positive film, clear for negative
        const POSITIVE  = 1 << 0;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Composition {
    BilevelBW,
    DitheredBW,
    MultilevelBW,
    BilevelRGB,
    DitheredRGB,
    MultilevelRGB,
    Unknown(u8),
}

impl From<u8> for Composition {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::BilevelBW,
            0x01 => Self::DitheredBW,
            0x02 => Self::MultilevelBW,
            0x03 => Self::BilevelRGB,
            0x04 => Self::DitheredRGB,
            0x05 => Self::MultilevelRGB,
            x => Self::Unknown(x),
        }
    }
}

impl From<Composition> for u8 {
    fn from(value: Composition) -> Self {
        match value {
            Composition::BilevelBW => 0x00,
            Composition::DitheredBW => 0x01,
            Composition::MultilevelBW => 0x02,
            Composition::BilevelRGB => 0x03,
            Composition::DitheredRGB => 0x04,
            Composition::MultilevelRGB => 0x05,
            Composition::Unknown(x) => x,
        }
    }
}

/// A descriptor shorter than [`LENGTH`]
#[derive(Debug, thiserror::Error)]
#[error("window descriptor was {got} bytes, need {LENGTH}")]
pub struct TooShort {
    pub got: usize,
}

impl TryFrom<&[u8]> for Window {
    type Error = TooShort;

    /// Takes at least [`LENGTH`] bytes and ignores any beyond it, since a unit
    /// may report a stride larger than the fields it defines
    fn try_from(d: &[u8]) -> Result<Self, TooShort> {
        let d: &[u8; LENGTH] = d
            .get(..LENGTH)
            .and_then(|d| d.try_into().ok())
            .ok_or(TooShort { got: d.len() })?;
        let be16 = |i: usize| u16::from_be_bytes([d[i], d[i + 1]]);
        let be32 = |i: usize| u32::from_be_bytes([d[i], d[i + 1], d[i + 2], d[i + 3]]);

        Ok(Self {
            id: d[0],
            auto: d[1] & 1 != 0,
            resolution: (be16(2), be16(4)),
            origin: (be32(6), be32(10)),
            size: (be32(14), be32(18)),
            brightness: d[22],
            threshold: d[23],
            contrast: d[24],
            composition: d[25].into(),
            bpp: d[26],
            halftone_pattern: be16(27),
            padding_type: d[29],
            bit_ordering: be16(30),
            compression_type: d[32],
            compression_argument: d[33],
            multiple_reading: d[40] >> 4,
            color_ordering: d[40] & 0x0F,
            flags: Flags::from_bits_truncate(d[41]),
            setup_mode: (d[41] >> 1) & 0b111,
            scanning_kind: ScanKind::from_bits_truncate(d[42]),
            scanning_mode: ScanMode::from_bits_truncate(d[43]),
            color_interleaving: ColorInterleaving::from_bits_truncate(d[44]),
            ae_value: d[45],
            exposure: be32(46),
        })
    }
}

/// How many planes a composition puts in the stream, per 2-10-6
///
/// Only the two multi-level codes are supported, and [`Window::validate`]
/// refuses the rest
fn planes(composition: Composition) -> Option<usize> {
    match composition {
        Composition::MultilevelBW => Some(1),
        Composition::MultilevelRGB => Some(3),
        _ => None,
    }
}

/// Check the rules that span a whole window set rather than one descriptor
///
/// These are what SCAN refuses rather than SET WINDOW: a descriptor is legal on
/// its own and only the combination is not. Each descriptor is checked by
/// [`Window::validate`] when it is set
pub fn validate_set(windows: &[Window]) -> Result<(), Error> {
    let bad = |op: &'static str, reason: String| Error::Unsupported { op, reason };

    let Some((first, rest)) = windows.split_first() else {
        return Err(bad("window set", "a scan needs at least one window".into()));
    };

    // 2-7: "The default color is valid when only the default color is read"
    if windows.len() > 1 && windows.iter().any(|w| w.id == DEFAULT) {
        return Err(bad(
            "window set",
            "the default color cannot be scanned alongside another".into(),
        ));
    }

    // 2-7 calls a disagreement here an invalid combination of windows. A set
    // carries one descriptor per channel so each can hold its own exposure
    let common = |w: &Window| {
        (
            w.resolution.0,
            w.size,
            w.origin,
            w.bpp,
            w.composition,
            w.color_interleaving,
            w.scanning_kind,
            w.scanning_mode,
            w.multiple_reading,
            w.flags,
        )
    };
    if let Some(odd) = rest.iter().find(|w| common(w) != common(first)) {
        return Err(bad(
            "window set",
            format!(
                "window {} differs from window {} in a parameter common to the set",
                odd.id, first.id
            ),
        ));
    }

    // 2-10 byte 40: the read positions are all zero, or all nonzero and
    // distinct. SCAN answers anything else with 05h-2Ch-02h
    let orders: Vec<u8> = windows.iter().map(|w| w.color_ordering).collect();
    if !orders.iter().all(|&o| o == 0) {
        if let Some(w) = windows.iter().find(|w| w.color_ordering == 0) {
            return Err(bad(
                "color ordering",
                format!(
                    "window {} leaves the order to the unit while the rest of the set pins it",
                    w.id
                ),
            ));
        }
        for (n, &order) in orders.iter().enumerate() {
            if orders[..n].contains(&order) {
                return Err(bad(
                    "color ordering",
                    format!("read position {order} is claimed twice"),
                ));
            }
        }
    }

    // 2-10-6's composition says how many planes the stream carries, and the id
    // list says how many channels are being scanned. A unit answers a
    // disagreement with common error 2, 05h-26h, which no section documents
    let planes = planes(first.composition).ok_or_else(|| {
        bad(
            "image composition",
            format!(
                "{:?} puts no known number of planes in the stream",
                first.composition
            ),
        )
    })?;
    if planes != windows.len() {
        return Err(bad(
            "image composition",
            format!(
                "{:?} carries {planes} plane(s) and this set scans {} channel(s)",
                first.composition,
                windows.len()
            ),
        ));
    }

    Ok(())
}

impl Window {
    /// Check this descriptor against what the unit says it will accept
    ///
    /// Per-window rules only. The rules spanning a whole set are
    /// [`validate_set`], since a descriptor can be legal alone and not in company
    ///
    /// A resolution off the unit's ladder is deliberately not refused: 2-10 says
    /// the unit rounds it and reports `01h-37h-00h`, so it is an adjustment
    /// rather than a rejection
    pub fn validate(&self, caps: &Capabilities) -> Result<(), Error> {
        let bad = |op: &'static str, reason: String| Error::Unsupported { op, reason };
        let (x, y) = (&caps.address.x_axis, &caps.address.y_axis);
        let f = &caps.set_window;

        if !matches!(self.id, 0..=3 | IR) {
            return Err(bad(
                "window identifier",
                format!(
                    "{} is not a scanning color: 2-10-4 defines 0 to 3, and 9 is infrared",
                    self.id
                ),
            ));
        }

        // SET WINDOW ignores Y, so only X is worth bounding
        let dpi = self.resolution.0;
        if dpi < x.dpi_range.start || dpi > x.dpi_range.last {
            return Err(bad(
                "resolution",
                format!(
                    "{dpi} dpi is outside the {} to {} this unit offers",
                    x.dpi_range.start, x.dpi_range.last
                ),
            ));
        }

        for (axis, name, origin, size) in [
            (x, 'X', self.origin.0, self.size.0),
            (y, 'Y', self.origin.1, self.size.1),
        ] {
            if origin < axis.address_range.start || origin > axis.address_range.last {
                return Err(bad(
                    "window origin",
                    format!(
                        "{name} {origin} is outside {} to {}",
                        axis.address_range.start, axis.address_range.last
                    ),
                ));
            }
            if size == 0 {
                return Err(bad("window size", format!("{name} is empty")));
            }
            // The boundary is the holder's opening, not the unit's limit. A wider
            // holder scans past it -- 9996 has been read off a 9000 against 8964
            // here -- and the unit's own power-on descriptors exceed it too, so
            // this is worth saying and not worth refusing
            if size > axis.boundary {
                warn!(
                    %name, size, aperture = axis.boundary,
                    "window reaches past the holder opening"
                );
            }
            // 2-2-2-3: an axis with no address range has to be read whole
            if !axis.croppable() && size != axis.boundary {
                return Err(bad(
                    "window size",
                    format!(
                        "{name} cannot be cropped, so it has to be exactly {}",
                        axis.boundary
                    ),
                ));
            }
        }

        // Reading every CCD line at once walks the bar in blocks of Line Gap
        // Count, and C1h quotes its own geometry in whole ones. A width that is
        // not divides the bar mid-block and the columns come back interleaved
        // wrong, so it is worth saying -- but only in the mode that reads that way
        let block = u32::from(caps.address.line_gap);
        if self
            .color_interleaving
            .contains(ColorInterleaving::MULTILINE_SIMULTANEOUS)
            && block != 0
            && !self.size.0.is_multiple_of(block)
        {
            warn!(
                width = self.size.0,
                block, "width is not a whole number of line-gap blocks"
            );
        }

        // The sensor is the one width that really is a limit
        let ccd = u32::from(caps.address.ccd_pixels);
        if self.origin.0 + self.size.0 > ccd {
            return Err(bad(
                "window size",
                format!(
                    "X {} from {} runs past the {ccd} pixel sensor",
                    self.size.0, self.origin.0
                ),
            ));
        }

        // Comparing raw bits keeps one loop over three unrelated flag types
        for (chosen, offered, op) in [
            (self.scanning_kind.bits(), f.kind.bits(), "scanning kind"),
            (self.scanning_mode.bits(), f.mode.bits(), "scanning mode"),
            (
                self.color_interleaving.bits(),
                f.interleaving.bits(),
                "color interleaving",
            ),
        ] {
            if chosen == 0 || chosen & !offered != 0 {
                return Err(bad(
                    op,
                    format!("asked for {chosen:#04x} of the {offered:#04x} this unit offers"),
                ));
            }
        }

        let Some((_, depth)) = DEPTHS.iter().find(|(n, _)| *n == self.bpp) else {
            return Err(bad(
                "pixel composition",
                format!("{} bits is not a depth 2-2-2-4 defines", self.bpp),
            ));
        };
        if !f.depth.contains(*depth) {
            return Err(bad(
                "pixel composition",
                format!("this unit does not offer {} bits", self.bpp),
            ));
        }

        // 2-10-6 marks only the two multi-level codes supported, in both specs
        if !matches!(
            self.composition,
            Composition::MultilevelBW | Composition::MultilevelRGB
        ) {
            return Err(bad(
                "image composition",
                format!(
                    "2-10-6 supports neither {:?} nor anything else it lists past the two multi-level codes",
                    self.composition
                ),
            ));
        }

        if self.setup_mode != 0 {
            if !f.kind.contains(ScanKind::SETUP_2) {
                return Err(bad(
                    "setup mode",
                    "this unit does not offer setup scanning 2, which is what makes byte 41 bits 3-1 mean anything".into(),
                ));
            }
            if self.setup_mode > f.setup_modes {
                return Err(bad(
                    "setup mode",
                    format!(
                        "{} is past the {} this unit offers",
                        self.setup_mode, f.setup_modes
                    ),
                ));
            }
        }

        if self.multiple_reading != 0 {
            if self.multiple_reading > 0x0F {
                return Err(bad(
                    "multiple reading",
                    format!(
                        "{} does not fit the nibble byte 40 gives it",
                        self.multiple_reading
                    ),
                ));
            }
            if !f.mode.contains(ScanMode::MULTI_READING) {
                return Err(bad(
                    "multiple reading",
                    "this unit reads each line once".into(),
                ));
            }
        }

        // The value is a read position, 0 meaning the unit's own order. `D1h`
        // bytes 8-9 also pin which component may sit at each position, but
        // nothing states how a window identifier maps to a component, so that
        // half is left to SCAN
        if self.color_ordering > 3 {
            return Err(bad(
                "color ordering",
                format!("{} is not a read position", self.color_ordering),
            ));
        }

        // 0 hands the choice to the unit, which then reports what it picked
        if self.exposure != 0
            && (self.exposure < f.exposure.start || self.exposure > f.exposure.last)
        {
            return Err(bad(
                "exposure",
                format!(
                    "{} is outside the {} to {} ten-nanosecond units this unit offers",
                    self.exposure, f.exposure.start, f.exposure.last
                ),
            ));
        }

        // 2-10 pins these to 0 for both units
        for (value, op) in [
            (self.padding_type, "padding type"),
            (self.compression_type, "compression type"),
            (self.compression_argument, "compression argument"),
        ] {
            if value != 0 {
                return Err(bad(op, format!("2-10 defines this as 0, not {value}")));
            }
        }
        if self.bit_ordering != 0 {
            return Err(bad(
                "bit ordering",
                format!("2-10 defines this as 0, not {}", self.bit_ordering),
            ));
        }

        Ok(())
    }

    /// Write one descriptor
    pub fn to_bytes(&self) -> [u8; LENGTH] {
        let mut d = [0u8; LENGTH];
        let be16 = |d: &mut [u8; LENGTH], i: usize, v: u16| {
            d[i..i + 2].copy_from_slice(&v.to_be_bytes());
        };

        d[0] = self.id;
        d[1] = u8::from(self.auto);
        be16(&mut d, 2, self.resolution.0);
        be16(&mut d, 4, self.resolution.1);
        d[6..10].copy_from_slice(&self.origin.0.to_be_bytes());
        d[10..14].copy_from_slice(&self.origin.1.to_be_bytes());
        d[14..18].copy_from_slice(&self.size.0.to_be_bytes());
        d[18..22].copy_from_slice(&self.size.1.to_be_bytes());
        d[22] = self.brightness;
        d[23] = self.threshold;
        d[24] = self.contrast;
        d[25] = self.composition.into();
        d[26] = self.bpp;
        be16(&mut d, 27, self.halftone_pattern);
        d[29] = self.padding_type;
        be16(&mut d, 30, self.bit_ordering);
        d[32] = self.compression_type;
        d[33] = self.compression_argument;
        // 34-39 reserved
        d[40] = (self.multiple_reading << 4) | (self.color_ordering & 0x0F);
        d[41] = self.flags.bits() | ((self.setup_mode & 0b111) << 1);
        d[42] = self.scanning_kind.bits();
        d[43] = self.scanning_mode.bits();
        d[44] = self.color_interleaving.bits();
        d[45] = self.ae_value;
        d[46..50].copy_from_slice(&self.exposure.to_be_bytes());
        d
    }
}

/// Both headers are eight bytes, though they do not agree on what is in them
pub const HEADER: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Header tacked on to the front of an incoming GET WINDOW payload
/// Table 2-10-2
pub struct GetWindowHeader {
    /// Bytes 0,1. Counts what follows it, so the whole reply is two more
    pub data_length: u16,
    /// Bytes 6,7. A unit may report a stride longer than 2-10-3 defines
    pub descriptor_length: u16,
}

impl GetWindowHeader {
    /// Read the header and return the rest of the slice
    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), TooShort> {
        if bytes.len() < HEADER {
            return Err(TooShort { got: bytes.len() });
        }
        let data_length = u16::from_be_bytes([bytes[0], bytes[1]]);
        let descriptor_length = u16::from_be_bytes([bytes[6], bytes[7]]);
        Ok((
            Self {
                data_length,
                descriptor_length,
            },
            &bytes[HEADER..],
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Header tacked on to the front of an outgoing SET WINDOW payload
/// Table 2-9-2
pub struct SetWindowHeader {
    /// Bytes 6,7. Sending 50 against a unit claiming more is fine: 2-9 note 3
    /// leaves the rest unchanged, and Nikon Scan sends 50
    pub descriptor_length: u16,
}

impl SetWindowHeader {
    /// Pack to bytes to send
    pub fn to_bytes(&self) -> [u8; HEADER] {
        let mut bytes = [0u8; HEADER];
        bytes[6..8].copy_from_slice(&self.descriptor_length.to_be_bytes());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The five descriptors an LS-9000 reports at power-on, header stripped.
    /// Real bytes: full frame, 4000 dpi, per-channel exposures, and an
    /// infrared window whose identifier appears in neither spec
    const LS9000: &[u8] = &[
        // id 0, default color
        0x00, 0x00, 0x0F, 0xA0, 0x0F, 0xA0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x27, 0x10, 0x00, 0x00, 0x36, 0x24, 0x00, 0x00, 0x00, 0x02, 0x10, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x02, 0x02,
        0xFF, 0x00, 0x00, 0xC6, 0x9A, // id 1, R
        0x01, 0x00, 0x0F, 0xA0, 0x0F, 0xA0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x27, 0x10, 0x00, 0x00, 0x36, 0x24, 0x00, 0x00, 0x00, 0x02, 0x10, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x02, 0x02,
        0xFF, 0x00, 0x01, 0x15, 0xD5, // id 9, infrared
        0x09, 0x00, 0x0F, 0xA0, 0x0F, 0xA0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x27, 0x10, 0x00, 0x00, 0x36, 0x24, 0x00, 0x00, 0x00, 0x02, 0x10, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x02, 0x02,
        0xFF, 0x00, 0x01, 0x6B, 0x4C,
    ];

    fn nth(n: usize) -> Window {
        Window::try_from(&LS9000[n * LENGTH..]).expect("descriptor")
    }

    /// Three descriptors Nikon Scan actually sent, lifted out of the capture
    /// corpus with the 8-byte SET WINDOW header stripped. The first two are the
    /// same 4000 dpi scan in the two sensor modes; the third is a 16x prescan
    mod captured {
        /// `full_session_cold_start`, normal CCD mode
        pub const MULTI_LINE: &[u8] = &[
            0x01, 0x00, 0x0F, 0xA0, 0x0F, 0xA0, 0x00, 0x00, 0x02, 0x06, 0x00, 0x00, 0x29, 0x10,
            0x00, 0x00, 0x23, 0x04, 0x00, 0x00, 0x1A, 0x28, 0x00, 0x00, 0x00, 0x05, 0x10, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x81,
            0x01, 0x02, 0x40, 0xFF, 0x00, 0x07, 0xAB, 0xDD,
        ];

        /// `singleline_ccd`, the mode Nikon Scan calls Super Fine
        pub const SINGLE_LINE: &[u8] = &[
            0x09, 0x00, 0x0F, 0xA0, 0x0F, 0xA0, 0x00, 0x00, 0x02, 0x06, 0x00, 0x00, 0x48, 0xF0,
            0x00, 0x00, 0x23, 0x04, 0x00, 0x00, 0x33, 0x78, 0x00, 0x00, 0x00, 0x05, 0x10, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x81,
            0x01, 0x02, 0x02, 0xFF, 0x00, 0x08, 0x39, 0xDE,
        ];

        /// `16x_multisample`, the 666 dpi pass before the scan
        pub const PRESCAN_16X: &[u8] = &[
            0x01, 0x00, 0x02, 0x9A, 0x01, 0x4D, 0x00, 0x00, 0x02, 0x06, 0x00, 0x00, 0x0E, 0xA0,
            0x00, 0x00, 0x23, 0x04, 0x00, 0x00, 0x33, 0x78, 0x00, 0x00, 0x00, 0x05, 0x10, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x01,
            0x01, 0x14, 0x40, 0xFF, 0x00, 0x07, 0x55, 0xAB,
        ];
    }

    /// The sensor mode is byte 44 and nothing else: two scans at the same
    /// resolution, same quality, same averaging, differing in that one field
    #[test]
    fn the_two_sensor_modes_differ_only_in_the_interleaving() {
        let multi = Window::try_from(captured::MULTI_LINE).expect("descriptor");
        let single = Window::try_from(captured::SINGLE_LINE).expect("descriptor");

        for w in [&multi, &single] {
            assert_eq!(w.resolution, (4000, 4000));
            assert_eq!(w.flags, Flags::AVERAGING | Flags::POSITIVE);
            assert_eq!(w.scanning_kind, ScanKind::IMAGE);
            assert_eq!(w.scanning_mode, ScanMode::NORMAL_QUALITY);
            assert_eq!(w.multiple_reading, 0);
            // Every window is one color, yet Nikon Scan asks for the RGB code
            assert_eq!(w.composition, Composition::MultilevelRGB);
        }

        assert_eq!(
            multi.color_interleaving,
            ColorInterleaving::MULTILINE_SIMULTANEOUS
        );
        assert_eq!(
            single.color_interleaving,
            ColorInterleaving::LINE_WITHOUT_DISTANCE
        );
    }

    /// Multisampling is a count in byte 40 and a mode bit in byte 43, together
    #[test]
    fn sixteen_times_is_fifteen_extra_reads_and_a_mode_bit() {
        let w = Window::try_from(captured::PRESCAN_16X).expect("descriptor");
        assert_eq!(w.multiple_reading, 15);
        assert_eq!(
            w.scanning_mode,
            ScanMode::HIGH_SPEED | ScanMode::MULTI_READING
        );
        // High speed is what clears averaging, and it is the preview that asks
        // for high speed -- resolution never selects it on its own
        assert_eq!(w.flags, Flags::POSITIVE);
        // Y resolution is sent as something else entirely, and ignored
        assert_eq!(w.resolution, (666, 333));
    }

    #[test]
    fn a_power_on_descriptor_reads_the_whole_frame_at_full_resolution() {
        let w = nth(0);
        assert_eq!(w.resolution, (4000, 4000));
        assert_eq!(w.origin, (0, 0));
        // 10000 x 13860 at 4000 dpi is the 6x9 frame, matching C1h's y boundary
        assert_eq!(w.size, (10000, 13860));
        assert_eq!(w.bpp, 16);
        assert_eq!(w.composition, Composition::MultilevelBW);
        assert_eq!(w.ae_value, 255);
        // The same flags D1h advertises, so a window can be checked against it
        assert_eq!(w.scanning_kind, ScanKind::IMAGE);
        assert_eq!(w.scanning_mode, ScanMode::NORMAL_QUALITY);
        assert_eq!(
            w.color_interleaving,
            ColorInterleaving::LINE_WITHOUT_DISTANCE
        );
        // Byte 41 is 01h at power-on: positive film, no averaging
        assert!(w.flags.contains(Flags::POSITIVE));
        assert!(!w.flags.contains(Flags::AVERAGING));
    }

    /// Exposures are per channel, and infrared needs nearly twice green's
    #[test]
    fn each_window_carries_its_own_exposure() {
        assert_eq!((nth(0).id, nth(0).exposure), (0, 50842));
        assert_eq!((nth(1).id, nth(1).exposure), (1, 71125));
        assert_eq!((nth(2).id, nth(2).exposure), (9, 93004));
    }

    /// Whatever we send has to survive the trip, or SET WINDOW will not match
    /// what GET WINDOW reported
    #[test]
    fn descriptors_round_trip_byte_for_byte() {
        for n in 0..3 {
            let bytes = &LS9000[n * LENGTH..(n + 1) * LENGTH];
            assert_eq!(nth(n).to_bytes(), bytes, "descriptor {n}");
        }
    }

    #[test]
    fn a_short_descriptor_is_refused() {
        assert!(Window::try_from(&LS9000[..LENGTH - 1]).is_err());
    }

    fn set(ids: &[u8], composition: Composition) -> Vec<Window> {
        ids.iter()
            .map(|&id| {
                let mut w = Window::try_from(&[0u8; LENGTH][..]).unwrap();
                w.id = id;
                w.composition = composition;
                w
            })
            .collect()
    }

    /// What the hardware refused with common error 2: three channels declared
    /// as a one-plane composition
    #[test]
    fn the_composition_has_to_carry_one_plane_per_channel() {
        assert!(validate_set(&set(&[1], Composition::MultilevelBW)).is_ok());
        assert!(validate_set(&set(&[1, 2, 3], Composition::MultilevelRGB)).is_ok());
        assert!(validate_set(&set(&[1, 2, 3], Composition::MultilevelBW)).is_err());
        assert!(validate_set(&set(&[1], Composition::MultilevelRGB)).is_err());
    }

    /// 2-7: "The default color is valid when only the default color is read"
    #[test]
    fn the_default_color_scans_alone_or_not_at_all() {
        assert!(validate_set(&set(&[DEFAULT], Composition::MultilevelBW)).is_ok());
        assert!(validate_set(&set(&[DEFAULT, 1, 3], Composition::MultilevelRGB)).is_err());
    }

    /// 2-10 byte 40, which SCAN answers with 05h-2Ch-02h
    #[test]
    fn a_window_set_orders_every_color_or_none() {
        let ordered = |orders: &[u8]| {
            let mut windows = set(&[1, 2, 3], Composition::MultilevelRGB);
            for (w, &o) in windows.iter_mut().zip(orders) {
                w.color_ordering = o;
            }
            validate_set(&windows)
        };
        assert!(ordered(&[0, 0, 0]).is_ok());
        assert!(ordered(&[1, 2, 3]).is_ok());
        assert!(ordered(&[1, 0, 3]).is_err());
        assert!(ordered(&[1, 2, 2]).is_err());
    }

    /// 2-7 calls a disagreement an invalid combination of windows, but the
    /// per-channel exposure is exactly what a set is meant to differ in
    #[test]
    fn a_set_agrees_on_everything_but_its_exposures() {
        let mut windows = set(&[1, 2, 3], Composition::MultilevelRGB);
        windows[2].exposure = 71125;
        assert!(validate_set(&windows).is_ok());

        windows[2].size.0 = 9000;
        assert!(validate_set(&windows).is_err());
    }

    #[test]
    fn an_empty_window_set_is_refused() {
        assert!(validate_set(&[]).is_err());
    }
}
