//! SCAN(6) - Requests a target begin a scan operation

use crate::scsi::{Cdb, Command, CommandData, Error};

pub struct Scan {
    /// Logical unit number (3 bits)
    lun: u8,
    /// Window identifiers (as previously defined by SET WINDOW) to scan
    window_ids: Vec<u8>,
    /// Control,
    control: u8,
}

impl Scan {
    pub fn new(lun: u8, window_ids: Vec<u8>, control: u8) -> Self {
        Self {
            lun,
            window_ids,
            control,
        }
    }
}

impl Command for Scan {
    type Response = ();
    type Cdb = Cdb<6>;

    fn cdb(&self) -> Self::Cdb {
        Cdb([
            0x1B, // opcode
            (self.lun & 0b111) << 5,
            0x00, // reserved
            0x00, // reserved
            self.window_ids.len() as u8,
            self.control,
        ])
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::Write(&self.window_ids)
    }

    fn decode(&self, _data: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cdb_encodes_opcode_and_lun() {
        let scan = Scan::new(3, vec![], 0);
        let cdb = scan.cdb().0;
        assert_eq!(cdb[0], 0x1B);
        assert_eq!(cdb[1], 3 << 5);
    }

    #[test]
    fn cdb_encodes_transfer_length_as_window_id_count() {
        let scan = Scan::new(0, vec![1, 2, 3], 0);
        assert_eq!(scan.cdb().0[4], 3);
    }

    #[test]
    fn cdb_encodes_control_byte_verbatim() {
        let scan = Scan::new(0, vec![], 0x80);
        assert_eq!(scan.cdb().0[5], 0x80);
    }

    #[test]
    fn cdb_matches_real_capture() {
        // LS-9000ED RGB scan capture: CDB `1B 00 00 00 03 00`, DATA-OUT `01 02 03`
        let scan = Scan::new(0, vec![1, 2, 3], 0x00);
        assert_eq!(scan.cdb().0, [0x1B, 0x00, 0x00, 0x00, 0x03, 0x00]);
        assert!(matches!(scan.data(), CommandData::Write(ids) if ids == [1, 2, 3]));
    }

    #[test]
    fn data_is_write_with_window_ids() {
        let scan = Scan::new(0, vec![9], 0);
        assert!(matches!(scan.data(), CommandData::Write(ids) if ids == [9]));
    }

    #[test]
    fn empty_window_id_list_is_not_an_error() {
        let scan = Scan::new(0, vec![], 0);
        assert_eq!(scan.cdb().0[4], 0);
        assert!(matches!(scan.data(), CommandData::Write(ids) if ids.is_empty()));
    }

    #[test]
    fn decode_ignores_input() {
        let scan = Scan::new(0, vec![], 0);
        assert_eq!(scan.decode(&[]).unwrap(), ());
        assert_eq!(scan.decode(&[1, 2, 3]).unwrap(), ());
    }
}
