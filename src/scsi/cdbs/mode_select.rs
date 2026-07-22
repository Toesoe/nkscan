//! MODE SELECT has two variants, a 6 and 10 byte
//! It seems the scanners we've used only use the 6 byte form
use crate::scsi::{Cdb, Command, CommandData, Error};

#[derive(Debug, Clone)]
/// MODE SELECT(6) command provides a means for the initiator to specify medium,
/// logical unit, or peripheral device parameters to the target.
/// Targets that implement the MODE SELECT command shall also implement the MODE SENSE command.
/// Initiators should issue MODE SENSE prior to each MODE SELECT to determine supported pages, page lengths, and other parameters.
pub struct ModeSelect {
    /// Logical unit number
    lun: u8,
    /// Page format.
    /// False indicates that the parameters follow SCSI-1
    /// True indicates the parameters following the header and block descriptors are structured as pages
    pf: bool,
    /// Save pages,
    /// False indicates the taget shall perform the requested mode select but not save the pages
    sp: bool,
    /// Parameter list bytes sent to the target
    parameter_list: Vec<u8>,
    /// Control byte
    control: u8,
}

impl ModeSelect {
    pub fn new(lun: u8, pf: bool, sp: bool, parameter_list: Vec<u8>, control: u8) -> Self {
        Self {
            lun,
            pf,
            sp,
            control,
            parameter_list,
        }
    }
}

impl Command for ModeSelect {
    type Response = ();
    type Cdb = Cdb<6>;

    fn cdb(&self) -> Self::Cdb {
        Cdb([
            0x15, // opcode
            ((self.lun & 0b111) << 5) | ((self.pf as u8) << 4) | (self.sp as u8),
            0x00,
            0x00,
            self.parameter_list.len() as u8,
            self.control,
        ])
    }

    fn data(&self) -> CommandData<'_> {
        CommandData::Write(&self.parameter_list)
    }

    fn decode(&self, _data: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}
