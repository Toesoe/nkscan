//! Film formats and their frame heights
//!
//! The frame height (film format) is the one piece of information the scanner
//! cannot derive from a thumbnail. It is not a property of the scanner or the
//! holder — the same 6×9 holder accepts 6×6, 6×7, 6×8 and 6×9 film — so the
//! holder ID narrows the choices but does not fix the answer.
//!
//! The holder ID does fix it for the FH-869GR, which has a mask that selects
//! the format physically. For everything else the operator supplies it.

/// A film format, keyed by its frame height in millimetres
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilmFormat {
    /// 135 film (35mm), 24 × 36 mm
    F135,
    /// 16mm film, 16 × 20 mm (FH-816)
    F16,
    /// 120 film, 6 × 4.5 cm
    F645,
    /// 120 film, 6 × 6 cm
    F66,
    /// 120 film, 6 × 7 cm
    F67,
    /// 120 film, 6 × 8 cm
    F68,
    /// 120 film, 6 × 9 cm
    F69,
    /// Operator-supplied height in millimetres
    Custom(u32),
}

impl FilmFormat {
    /// Frame height along the feed, in mm
    pub const fn height_mm(self) -> u32 {
        match self {
            Self::F135 => 36,
            Self::F16 => 20,
            Self::F645 => 45,
            Self::F66 => 56,
            Self::F67 => 70,
            Self::F68 => 80,
            Self::F69 => 84,
            Self::Custom(mm) => mm,
        }
    }

    /// Frame height in scanner address units (dots at optical DPI)
    pub fn height_dots(self, dpi: u16) -> u32 {
        // mm × dpi / 25.4, rounded to nearest
        let num = u64::from(self.height_mm()) * u64::from(dpi) * 10;
        ((num + 127) / 254) as u32
    }

    /// The format a holder ID implies, where the holder fixes it
    ///
    /// Returns `None` where the holder accepts more than one format, or where
    /// the holder is unknown. The FH-869GR is the only holder that physically
    /// selects the format via its mask
    pub fn from_holder(holder_id: u8) -> Option<Self> {
        match holder_id {
            0x12 => Some(Self::F16),  // FH-816
            0x19 => Some(Self::F645), // FH-869GR 6×4.5
            0x1A => Some(Self::F66),  // FH-869GR 6×6
            0x1B => Some(Self::F67),  // FH-869GR 6×7
            0x1C => Some(Self::F68),  // FH-869GR 6×8
            0x1D => Some(Self::F69),  // FH-869GR 6×9
            _ => None,
        }
    }

    /// The formats a holder ID accepts, where it accepts more than one
    ///
    /// Returns `None` where the holder fixes the format (see [`from_holder`])
    /// or is unknown. Used to offer the operator a choice
    pub fn choices_for_holder(holder_id: u8) -> Option<&'static [Self]> {
        match holder_id {
            0x14 => Some(&[Self::F66, Self::F67, Self::F69]), // FH-835M
            0x15 => Some(&[Self::F66, Self::F67, Self::F69]), // FH-835S
            0x16 => Some(&[Self::F66, Self::F67, Self::F69]), // FH-869M
            0x17 => Some(&[Self::F66, Self::F67, Self::F69]), // FH-869S
            0x18 => Some(&[Self::F66, Self::F67, Self::F69]), // FH-869G
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_heights() {
        assert_eq!(FilmFormat::F135.height_mm(), 36);
        assert_eq!(FilmFormat::F16.height_mm(), 20);
        assert_eq!(FilmFormat::F66.height_mm(), 56);
        assert_eq!(FilmFormat::F69.height_mm(), 84);
        assert_eq!(FilmFormat::Custom(100).height_mm(), 100);
    }

    #[test]
    fn dots_at_4000_dpi() {
        assert_eq!(FilmFormat::F66.height_dots(4000), 8819);
        assert_eq!(FilmFormat::F69.height_dots(4000), 13228);
    }

    #[test]
    fn gr_holder_fixes_format() {
        assert_eq!(FilmFormat::from_holder(0x1A), Some(FilmFormat::F66));
        assert_eq!(FilmFormat::from_holder(0x1D), Some(FilmFormat::F69));
        assert_eq!(FilmFormat::from_holder(0x17), None);
    }

    #[test]
    fn strip_holder_offers_choices() {
        let choices = FilmFormat::choices_for_holder(0x17).unwrap();
        assert!(choices.contains(&FilmFormat::F66));
        assert!(choices.contains(&FilmFormat::F69));
        assert!(FilmFormat::choices_for_holder(0x1A).is_none());
    }
}
