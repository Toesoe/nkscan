//! Interpreting TEST UNIT READY sense data for the LS-9000.
//!
//! SCSI itself has no notion of "the device is fine, just not ready yet" vs
//! "something is actually wrong", both arrive as CHECK CONDITION with a
//! sense key/ASC/ASCQ triple. These particular codes are what this scanner
//! has been observed to return; anything else is a genuine error, not a state.

use crate::scsi::SenseData;

/// Scanner state, as reported by TEST UNIT READY.
///
/// Only sense codes confirmed to show up during normal operation (warm-up,
/// holder swaps, medium changes) are represented here. Any other sense data
/// stays a real `Err(ScsiError)` rather than being folded into a variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ready,
    /// NotReady 0x04/0x01, warming up / initializing.
    Initializing,
    /// NotReady 0x3A/0x00, no film holder loaded.
    NoFilmHolder,
    /// UnitAttention 0x3F/0x04, "microcode has been changed". Seen on the
    /// first command after SBP-2 login; clear and retry.
    Reset,
    /// UnitAttention 0x3F/0x03, "inquiry data has changed". The medium
    /// changed; INQUIRY/EVPD-derived state may be stale and worth re-reading.
    MediumChanged,
    /// UnitAttention 0x28/0x00, "not ready to ready change". A film holder
    /// was inserted or removed; re-enumerate it.
    HolderChanged,
}

impl Status {
    /// Classify a CHECK CONDITION's sense data as a known readiness state.
    /// Returns `None` for anything not recognized as "not ready yet"; the
    /// caller should treat that as a real error.
    pub(crate) fn from_sense(sense: &SenseData) -> Option<Self> {
        match (sense.key, sense.asc, sense.ascq) {
            (0x02, 0x04, 0x01) => Some(Self::Initializing),
            (0x02, 0x3A, 0x00) => Some(Self::NoFilmHolder),
            (0x06, 0x3F, 0x04) => Some(Self::Reset),
            (0x06, 0x3F, 0x03) => Some(Self::MediumChanged),
            (0x06, 0x28, 0x00) => Some(Self::HolderChanged),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sense(key: u8, asc: u8, ascq: u8) -> SenseData {
        SenseData { key, asc, ascq }
    }

    #[test]
    fn recognizes_initializing() {
        assert_eq!(
            Status::from_sense(&sense(0x02, 0x04, 0x01)),
            Some(Status::Initializing)
        );
    }

    #[test]
    fn recognizes_no_film_holder() {
        assert_eq!(
            Status::from_sense(&sense(0x02, 0x3A, 0x00)),
            Some(Status::NoFilmHolder)
        );
    }

    #[test]
    fn recognizes_reset() {
        assert_eq!(
            Status::from_sense(&sense(0x06, 0x3F, 0x04)),
            Some(Status::Reset)
        );
    }

    #[test]
    fn recognizes_medium_changed() {
        assert_eq!(
            Status::from_sense(&sense(0x06, 0x3F, 0x03)),
            Some(Status::MediumChanged)
        );
    }

    #[test]
    fn recognizes_holder_changed() {
        assert_eq!(
            Status::from_sense(&sense(0x06, 0x28, 0x00)),
            Some(Status::HolderChanged)
        );
    }

    #[test]
    fn unrecognized_sense_is_not_a_state() {
        assert_eq!(Status::from_sense(&sense(0x05, 0x24, 0x00)), None);
    }
}
