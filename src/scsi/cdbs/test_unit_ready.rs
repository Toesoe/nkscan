use crate::scsi::{Cdb, Command, DataDirection, Error};

#[derive(Debug, Default)]
/// TEST UNIT READY - main SCSI status CDB
pub struct TestUnitReady;

impl TestUnitReady {
    pub fn new() -> Self {
        Self
    }
}

impl Command for TestUnitReady {
    type Response = ();
    type Cdb = Cdb<6>;

    fn cdb(&self) -> Self::Cdb {
        Cdb([0; 6])
    }

    fn direction(&self) -> DataDirection {
        DataDirection::None
    }

    fn data_length(&self) -> usize {
        0
    }

    fn decode(&self, _data: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}
