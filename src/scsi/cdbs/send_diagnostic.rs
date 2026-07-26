//! SEND DIAGNOSTIC(6), SCSI-2 8.2.1

use crate::scsi::{Cdb, Command, CommandData, Error, fields::lun_byte};

/// SEND DIAGNOSTIC(6): run a self-test, or hand the device a diagnostic parameter list
pub struct SendDiagnostic {
    /// Logical unit number (3 bits)
    lun: u8,
    /// Page Format: the parameter list follows the standard page layout
    pf: bool,
    /// SelfTest: run the device's own default test
    self_test: bool,
    /// Device Offline: the test may disturb other logical units
    devofl: bool,
    /// Unit Offline: the test may disturb this logical unit's medium
    unitofl: bool,
    /// Diagnostic parameter list, empty for a plain self-test
    parameters: Vec<u8>,
    /// Control byte
    control: u8,
}

impl SendDiagnostic {
    pub fn new(
        lun: u8,
        pf: bool,
        self_test: bool,
        devofl: bool,
        unitofl: bool,
        parameters: Vec<u8>,
        control: u8,
    ) -> Self {
        Self {
            lun,
            pf,
            self_test,
            devofl,
            unitofl,
            parameters,
            control,
        }
    }

    /// SelfTest set and no parameter list, so the device picks the test itself
    pub fn self_test() -> Self {
        Self::new(0, false, true, false, false, Vec::new(), 0x00)
    }
}

impl Command for SendDiagnostic {
    type Response = ();
    type Cdb = Cdb<6>;

    fn cdb(&self) -> Self::Cdb {
        let [length_hi, length_lo] = (self.parameters.len() as u16).to_be_bytes();
        Cdb([
            0x1D, // opcode
            lun_byte(self.lun)
                | ((self.pf as u8) << 4)
                | ((self.self_test as u8) << 2)
                | ((self.devofl as u8) << 1)
                | self.unitofl as u8,
            0x00, // reserved
            length_hi,
            length_lo,
            self.control,
        ])
    }

    fn data(&self) -> CommandData<'_> {
        if self.parameters.is_empty() {
            CommandData::None
        } else {
            CommandData::Write(&self.parameters)
        }
    }

    fn parse_response(&self, _data: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_test_matches_real_capture() {
        // Nikon Scan's self-test on the LS-50: `1D 04 00 00 00 00`, no data phase
        let cdb = SendDiagnostic::self_test();
        assert_eq!(cdb.cdb().0, [0x1D, 0x04, 0x00, 0x00, 0x00, 0x00]);
        assert!(matches!(cdb.data(), CommandData::None));
    }

    #[test]
    fn byte_1_packs_lun_and_the_four_flags() {
        let cdb = SendDiagnostic::new(3, true, true, true, true, Vec::new(), 0x00)
            .cdb()
            .0;
        assert_eq!(cdb[1], (3 << 5) | 0b0001_0111);
    }

    /// Byte 1 carries the flags below the LUN, so an over-large one must not reach them
    #[test]
    fn out_of_range_lun_does_not_spill_into_the_flags() {
        let cdb = SendDiagnostic::new(9, false, true, false, false, Vec::new(), 0x00)
            .cdb()
            .0;
        assert_eq!(cdb[1], (1 << 5) | 0b100);
    }

    #[test]
    fn cdb_encodes_parameter_list_length_and_control() {
        let send = SendDiagnostic::new(0, true, false, false, false, vec![0xAA; 0x1234], 0x80);
        let cdb = send.cdb().0;
        assert_eq!([cdb[3], cdb[4]], [0x12, 0x34]);
        assert_eq!(cdb[5], 0x80);
        assert!(matches!(send.data(), CommandData::Write(p) if p.len() == 0x1234));
    }
}
