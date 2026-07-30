//! A transport that answers from a script and remembers what it was asked
//!
//! For testing the order a driver issues commands in, which is the part of a driver that no
//! amount of byte-level CDB testing covers. Every command is logged; anything the test has not
//! set up succeeds with a zeroed buffer, so a test only has to describe what it cares about.

use super::{DataDirection, Error, Transport};
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

/// One command as the mock saw it
pub struct Sent {
    pub cdb: Vec<u8>,
    /// What the caller wrote, empty for anything that is not a data-out command
    pub data: Vec<u8>,
}

/// State shared by every clone, so a test can still see what a driver it was given away to did
#[derive(Default)]
struct Shared {
    log: Vec<Sent>,
    /// Full VPD responses, header included, by page code
    pages: BTreeMap<u8, Vec<u8>>,
    /// Errors to hand back, consumed in order, by opcode
    failures: BTreeMap<u8, VecDeque<Error>>,
    /// A plain INQUIRY response, for a driver that checks what it is talking to
    identity: Option<Vec<u8>>,
}

/// Cloning shares the log rather than copying it, so a test can hand one clone to a driver and
/// keep the other to ask what happened
#[derive(Default, Clone)]
pub struct MockTransport {
    shared: Arc<Mutex<Shared>>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self::default()
    }

    /// Answer a plain INQUIRY as this vendor and product, the way a real unit introduces itself
    pub fn with_identity(self, vendor: &str, product: &str) -> Self {
        // Fixed-width ASCII fields: vendor at 8, product at 16, revision at 32
        let mut data = vec![b' '; 36];
        data[..8].copy_from_slice(&[0x06, 0x80, 0x02, 0x02, 0x1F, 0x00, 0x00, 0x00]);
        for (at, text) in [(8, vendor), (16, product), (32, "1.02")] {
            let bytes = text.as_bytes();
            let end = (at + bytes.len()).min(data.len());
            data[at..end].copy_from_slice(&bytes[..end - at]);
        }
        self.lock().identity = Some(data);
        self
    }

    /// Answer an EVPD inquiry for `page_code` with `raw`, header included
    pub fn with_page(self, page_code: u8, raw: Vec<u8>) -> Self {
        self.lock().pages.insert(page_code, raw);
        self
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Shared> {
        self.shared
            .lock()
            .expect("no test panics while holding this")
    }

    /// Fail the next command with this opcode. Queue several to fail several.
    pub fn failing(mut self, opcode: u8, error: Error) -> Self {
        self.fail_next(opcode, error);
        self
    }

    /// [`failing`](Self::failing) once the transport is already in a driver
    ///
    /// Opening a handle spends commands of its own, so a test aiming a failure at something
    /// later has to queue it after that rather than at construction.
    pub fn fail_next(&mut self, opcode: u8, error: Error) {
        self.lock()
            .failures
            .entry(opcode)
            .or_default()
            .push_back(error);
    }

    /// How many commands were executed
    pub fn len(&self) -> usize {
        self.lock().log.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// What every command with this opcode wrote, in order
    pub fn data_outs(&self, opcode: u8) -> Vec<Vec<u8>> {
        self.lock()
            .log
            .iter()
            .filter(|sent| sent.cdb[0] == opcode)
            .map(|sent| sent.data.clone())
            .collect()
    }

    /// What the first command with this opcode wrote
    pub fn data_out(&self, opcode: u8) -> Option<Vec<u8>> {
        self.lock()
            .log
            .iter()
            .find(|sent| sent.cdb[0] == opcode)
            .map(|sent| sent.data.clone())
    }

    /// The opcodes seen, with consecutive repeats collapsed
    ///
    /// Collapsed because the interesting thing is the order a driver does things in, and a poll
    /// loop repeating TEST UNIT READY an unpredictable number of times is not a difference worth
    /// pinning a test to.
    pub fn opcode_sequence(&self) -> Vec<u8> {
        let mut sequence: Vec<u8> = Vec::new();
        for sent in &self.lock().log {
            let opcode = sent.cdb[0];
            if sequence.last() != Some(&opcode) {
                sequence.push(opcode);
            }
        }
        sequence
    }

    /// Every CDB sent with this opcode, in order
    pub fn cdbs(&self, opcode: u8) -> Vec<Vec<u8>> {
        self.lock()
            .log
            .iter()
            .filter(|sent| sent.cdb[0] == opcode)
            .map(|sent| sent.cdb.clone())
            .collect()
    }

    pub fn count(&self, opcode: u8) -> usize {
        self.lock()
            .log
            .iter()
            .filter(|sent| sent.cdb[0] == opcode)
            .count()
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
        let mut shared = self.lock();
        shared.log.push(Sent {
            cdb: cdb.to_vec(),
            data: match direction {
                DataDirection::Write => data.to_vec(),
                _ => Vec::new(),
            },
        });

        if let Some(queued) = shared.failures.get_mut(&cdb[0])
            && let Some(error) = queued.pop_front()
        {
            return Err(error);
        }

        data.fill(0);
        // Byte 1 bit 0 is EVPD, which chooses between a vital product page and the identity
        let answer = match (cdb[0], cdb[1] & 1) {
            (0x12, 1) => shared.pages.get(&cdb[2]),
            (0x12, _) => shared.identity.as_ref(),
            _ => None,
        };
        if let Some(raw) = answer {
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
