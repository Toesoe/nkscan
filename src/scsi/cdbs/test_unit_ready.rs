use crate::scsi::{Cdb, Command, CommandData, Error};

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

    fn data(&self) -> CommandData<'_> {
        CommandData::None
    }

    fn parse_response(&self, _data: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}
