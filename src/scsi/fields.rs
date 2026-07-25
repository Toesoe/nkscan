//! Field encoders shared by the command descriptor blocks

/// Byte 1 of a CDB: the logical unit number in bits 7-5
///
/// SCSI-2 gives the field three bits, so anything above 7 is the caller's mistake and is
/// masked off rather than allowed to spill into the bits below.
pub(crate) const fn lun_byte(lun: u8) -> u8 {
    (lun & 0b111) << 5
}

/// A 24-bit big-endian field, as transfer and parameter list lengths are carried
///
/// Values above 0xFFFFFF silently lose their top byte, which the debug assert catches in
/// testing but the wire cannot express either way.
pub(crate) const fn be_u24(value: u32) -> [u8; 3] {
    debug_assert!(value <= 0xFF_FFFF, "value does not fit a 24-bit field");
    let [_, hi, mid, lo] = value.to_be_bytes();
    [hi, mid, lo]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lun_lands_in_the_top_three_bits() {
        assert_eq!(lun_byte(0), 0x00);
        assert_eq!(lun_byte(1), 0x20);
        assert_eq!(lun_byte(7), 0xE0);
    }

    #[test]
    fn out_of_range_lun_does_not_spill_into_the_low_bits() {
        // Byte 1 carries other fields on several commands, so an over-large LUN must not
        // reach them
        for lun in 8..=u8::MAX {
            assert_eq!(lun_byte(lun) & 0b0001_1111, 0, "lun {lun}");
        }
    }

    #[test]
    fn u24_keeps_the_low_three_bytes() {
        assert_eq!(be_u24(0), [0x00, 0x00, 0x00]);
        assert_eq!(be_u24(0x01_2345), [0x01, 0x23, 0x45]);
        assert_eq!(be_u24(0xFF_FFFF), [0xFF, 0xFF, 0xFF]);
    }
}
