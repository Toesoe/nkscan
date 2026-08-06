//! The typed READ and SEND data records, and EXECUTE. Sections 2-11, 2-14, 2-15

use super::{PROBE_TIMEOUT, Session, malformed};
use crate::{
    error::Error,
    protocol::{
        cdbs::{Execute, Read, SetParameter},
        data,
    },
    transport::Data,
};
use std::time::Duration;
use tracing::*;

impl Session {
    /// READ one data type, in two passes so the data header can size the second
    ///
    /// Only for the codes from 80h up, which are the ones the data header
    /// precedes. Image data carries none and goes through
    /// [`read_image`](Session::read_image)
    pub fn read_data(
        &mut self,
        kind: data::DataType,
        color: u8,
    ) -> Result<(data::Header, data::Values), Error> {
        let row = kind.row();
        let code = row.code;
        let refuse = |reason| {
            Err(Error::Unsupported {
                op: "read data type",
                reason,
            })
        };

        if !row.header {
            return refuse(format!(
                "{code:02X}h carries no data header to size a read by"
            ));
        }
        match row.read {
            Some(bit) if self.caps.features.data_types.contains(bit) => {}
            _ => return refuse(format!("this unit does not offer {code:02X}h")),
        }
        let Some(width) = row.width else {
            return refuse(format!("{code:02X}h takes a width 2-11-2 does not fix"));
        };
        let qualifier = data::width_code(width).expect("2-11-2 widths are all encodable");
        let color = if kind.per_color() { color } else { 0 };

        let mut fetch = |len: u32| -> Result<Vec<u8>, Error> {
            let cmd = Read::new(code, color, qualifier, len);
            let mut buf = vec![0u8; cmd.allocation_length()];
            let completion = self.run(&cmd.cdb(), Data::In(&mut buf), PROBE_TIMEOUT)?;
            buf.truncate(completion.transferred);
            Ok(buf)
        };

        // The header reports what the unit holds whatever we asked for, so one
        // short read is enough to size the real one
        let probe = fetch(data::HEADER as u32)?;
        let (probe, _) = data::Header::from_bytes(&probe)
            .ok_or_else(|| malformed(format!("{code:02X}h header was {} bytes", probe.len())))?;

        let raw = fetch(data::HEADER as u32 + probe.length)?;
        let (header, payload) = data::Header::from_bytes(&raw)
            .ok_or_else(|| malformed(format!("{code:02X}h header was {} bytes", raw.len())))?;

        // Analog gain reports 16 bytes against a documented 8, and the tail is
        // stale, so the table wins wherever it fixes a count
        let valid = match row.count {
            Some(n) => payload
                .get(..n as usize * width as usize)
                .unwrap_or(payload),
            None => payload,
        };
        debug!(?header, bytes = valid.len(), "read data");
        Ok((header, data::Values::decode(kind.scalar(), valid)))
    }

    /// Read the initiator cooperative action parameter a SCAN just asked for
    pub fn cooperation(&mut self) -> Result<data::CooperativeAction, Error> {
        let (_, values) = self.read_data(data::DataType::Cooperation, 0)?;
        let data::Values::Bytes(record) = values else {
            return Err(malformed("87h did not come back as bytes".into()));
        };
        data::CooperativeAction::from_bytes(&record)
            .ok_or_else(|| malformed(format!("87h was {} bytes", record.len())))
    }

    /// Set the operation parameter, activate the operation, and confirm its
    /// termination
    ///
    /// 2-14: EXECUTE performs the operation *after* returning GOOD status, and
    /// no command other than a basic command may be issued before the operation
    /// termination is confirmed by TEST UNIT READY. So all three are one call
    pub fn execute(
        &mut self,
        operation: u8,
        params: data::Operation,
        timeout: Duration,
    ) -> Result<(), Error> {
        if !self.caps.features.execute.supports(operation) {
            return Err(Error::Unsupported {
                op: "execute operation",
                reason: format!("this unit does not offer {operation:02X}h"),
            });
        }

        let block = params.to_bytes();
        let cmd = SetParameter::new(operation, block.len() as u32);
        self.run(&cmd.cdb(), Data::Out(&block), PROBE_TIMEOUT)?;

        debug!(
            operation = format!("{operation:02X}h"),
            ?params,
            "executing"
        );
        self.run(&Execute.cdb(), Data::None, PROBE_TIMEOUT)?;
        self.test_unit_ready(timeout)
    }
}
