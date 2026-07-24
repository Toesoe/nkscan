//! Interpreting TEST UNIT READY sense data for the LS-9000.
//!
//! SCSI itself has no notion of "the device is fine, just not ready yet" vs
//! "something is actually wrong", both arrive as CHECK CONDITION with a
//! sense key/ASC/ASCQ triple. But the sense key alone already tells us which
//! of those two buckets we're in: NOT READY and UNIT ATTENTION are defined
//! by the standard as transient conditions ("wait" / "something changed"),
//! never a real fault; see SPC-2 Table 69. So any sense data under those
//! two keys is a readiness state, not an error, whether or not we've seen
//! its specific ASC/ASCQ before.

use crate::scsi::{SenseData, SenseKey, asc::AdditionalSenseCode};

/// Scanner state, as reported by TEST UNIT READY.
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
    /// A NotReady or UnitAttention condition without a variant of its own
    /// above - still not a hard failure per the sense key alone, just one we
    /// haven't specifically named yet. Carries the decoded condition so
    /// callers/logs get a name instead of raw ASC/ASCQ bytes.
    Other(AdditionalSenseCode),
}

impl Status {
    /// Classify a CHECK CONDITION's sense data as a readiness state.
    /// Returns `None` only for sense keys that are genuine errors (anything
    /// other than NOT READY / UNIT ATTENTION); the caller should treat that
    /// as a real `Err`.
    pub(crate) fn from_sense(sense: &SenseData) -> Option<Self> {
        match (sense.sense_key(), sense.asc, sense.ascq) {
            (SenseKey::NotReady, 0x04, 0x01) => Some(Self::Initializing),
            (SenseKey::NotReady, 0x3A, 0x00) => Some(Self::NoFilmHolder),
            (SenseKey::UnitAttention, 0x3F, 0x04) => Some(Self::Reset),
            (SenseKey::UnitAttention, 0x3F, 0x03) => Some(Self::MediumChanged),
            (SenseKey::UnitAttention, 0x28, 0x00) => Some(Self::HolderChanged),
            (SenseKey::NotReady | SenseKey::UnitAttention, ..) => {
                Some(Self::Other(sense.condition()))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sense(key: u8, asc: u8, ascq: u8) -> SenseData {
        SenseData {
            key,
            asc,
            ascq,
            ili: false,
            deferred: false,
        }
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

    #[test]
    fn unrecognized_unit_attention_is_still_a_state_not_an_error() {
        // 0x3F/0x00 (TARGET OPERATING CONDITIONS HAVE CHANGED) isn't one of
        // this scanner's named variants, but UnitAttention itself is never a
        // hard error - it should still classify as a state.
        assert_eq!(
            Status::from_sense(&sense(0x06, 0x3F, 0x00)),
            Some(Status::Other(
                AdditionalSenseCode::TargetOperatingConditionsHaveChanged
            ))
        );
    }

    #[test]
    fn unrecognized_not_ready_is_still_a_state_not_an_error() {
        // 0x04/0x00 (LOGICAL UNIT NOT READY, CAUSE NOT REPORTABLE) isn't one
        // of this scanner's named variants, but NotReady itself is never a
        // hard error - it should still classify as a state.
        assert_eq!(
            Status::from_sense(&sense(0x02, 0x04, 0x00)),
            Some(Status::Other(
                AdditionalSenseCode::LogicalUnitNotReadyCauseNotReportable
            ))
        );
    }
}
