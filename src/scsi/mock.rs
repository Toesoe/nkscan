//! A transport that answers from a script and remembers what it was asked
//!
//! For testing the order a driver issues commands in, which is the part of a driver that no
//! amount of byte-level CDB testing covers. Every command is logged; anything the test has not
//! set up succeeds with a zeroed buffer, so a test only has to describe what it cares about.

use super::{DataDirection, Error, Transport};
use std::collections::{BTreeMap, VecDeque};

/// One command as the mock saw it
pub struct Sent {
    pub cdb: Vec<u8>,
    /// What the caller wrote, empty for anything that is not a data-out command
    pub data: Vec<u8>,
}

#[derive(Default)]
pub struct MockTransport {
    log: Vec<Sent>,
    /// Full VPD responses, header included, by page code
    pages: BTreeMap<u8, Vec<u8>>,
    /// Errors to hand back, consumed in order, by opcode
    failures: BTreeMap<u8, VecDeque<Error>>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self::default()
    }

    /// Answer an EVPD inquiry for `page_code` with `raw`, header included
    pub fn with_page(mut self, page_code: u8, raw: Vec<u8>) -> Self {
        self.pages.insert(page_code, raw);
        self
    }

    /// Fail the next command with this opcode. Queue several to fail several.
    pub fn failing(mut self, opcode: u8, error: Error) -> Self {
        self.failures.entry(opcode).or_default().push_back(error);
        self
    }

    /// Every command executed, in order
    pub fn log(&self) -> &[Sent] {
        &self.log
    }

    /// What the first command with this opcode wrote
    pub fn data_out(&self, opcode: u8) -> Option<&[u8]> {
        self.log
            .iter()
            .find(|sent| sent.cdb[0] == opcode)
            .map(|sent| sent.data.as_slice())
    }

    /// The opcodes seen, with consecutive repeats collapsed
    ///
    /// Collapsed because the interesting thing is the order a driver does things in, and a poll
    /// loop repeating TEST UNIT READY an unpredictable number of times is not a difference worth
    /// pinning a test to.
    pub fn opcode_sequence(&self) -> Vec<u8> {
        let mut sequence: Vec<u8> = Vec::new();
        for sent in &self.log {
            let opcode = sent.cdb[0];
            if sequence.last() != Some(&opcode) {
                sequence.push(opcode);
            }
        }
        sequence
    }

    /// Every CDB sent with this opcode, in order
    pub fn cdbs(&self, opcode: u8) -> Vec<&[u8]> {
        self.log
            .iter()
            .filter(|sent| sent.cdb[0] == opcode)
            .map(|sent| sent.cdb.as_slice())
            .collect()
    }

    pub fn count(&self, opcode: u8) -> usize {
        self.log.iter().filter(|sent| sent.cdb[0] == opcode).count()
    }
}

impl Transport for MockTransport {
    fn execute(
        &mut self,
        cdb: &[u8],
        direction: DataDirection,
        data: &mut [u8],
        _sense: &mut [u8],
    ) -> Result<(), Error> {
        self.log.push(Sent {
            cdb: cdb.to_vec(),
            data: match direction {
                DataDirection::Write => data.to_vec(),
                _ => Vec::new(),
            },
        });

        if let Some(queued) = self.failures.get_mut(&cdb[0])
            && let Some(error) = queued.pop_front()
        {
            return Err(error);
        }

        data.fill(0);
        // EVPD inquiry, where the page code is in byte 2
        if cdb[0] == 0x12
            && cdb[1] & 1 == 1
            && let Some(raw) = self.pages.get(&cdb[2])
        {
            let n = raw.len().min(data.len());
            data[..n].copy_from_slice(&raw[..n]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scsi::{SenseData, TransportExt, cdbs::TestUnitReady};

    #[test]
    fn a_poll_loop_collapses_to_one_entry() {
        let mut mock = MockTransport::new();
        for _ in 0..5 {
            mock.send(&TestUnitReady::new()).unwrap();
        }
        assert_eq!(mock.count(0x00), 5);
        assert_eq!(mock.opcode_sequence(), [0x00]);
    }

    #[test]
    fn queued_failures_are_consumed_in_order() {
        let error = || Error::Status {
            status: 0x02,
            sense: Some(SenseData {
                key: 0x05,
                asc: 0x2C,
                ascq: 0x00,
                ili: false,
                deferred: false,
            }),
        };
        let mut mock = MockTransport::new()
            .failing(0x00, error())
            .failing(0x00, error());

        assert!(mock.send(&TestUnitReady::new()).is_err());
        assert!(mock.send(&TestUnitReady::new()).is_err());
        // The queue is empty, so the command goes through
        assert!(mock.send(&TestUnitReady::new()).is_ok());
    }
}
