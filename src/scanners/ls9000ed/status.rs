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

use crate::{
    scanners::ScannerStatus,
    scsi::{SenseData, SenseKey, asc::AdditionalSenseCode},
};

/// Scanner state, as reported by TEST UNIT READY
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ready,
    /// Still coming up: NotReady 0x04/0x01 while warming up, or AbortedCommand 0x3E/0x00
    /// straight after power-on
    Initializing,
    /// NotReady 0x3A/0x00, no film holder loaded
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
    /// haven't specifically named yet. Carries the sense key so callers can
    /// still tell the two apart, and the decoded condition so logs get a name
    /// instead of raw ASC/ASCQ bytes.
    Other(SenseKey, AdditionalSenseCode),
}

impl ScannerStatus for Status {
    fn ready() -> Self {
        Status::Ready
    }

    /// Classify a CHECK CONDITION's sense data as a readiness state
    /// Returns `None` only for sense keys that are genuine errors (anything
    /// other than NOT READY / UNIT ATTENTION); the caller should treat that
    /// as a real `Err`.
    fn from_sense(sense: &SenseData) -> Option<Self> {
        match (sense.sense_key(), sense.asc, sense.ascq) {
            (SenseKey::NotReady, 0x04, 0x01) => Some(Self::Initializing),
            // The one exception to the sense-key rule above. AbortedCommand is a real fault
            // as a key, but 0x3E/0x00 is the device saying it hasn't finished coming up, so
            // it clears on its own. Seen on the first commands after a power cycle.
            (SenseKey::AbortedCommand, 0x3E, 0x00) => Some(Self::Initializing),
            (SenseKey::NotReady, 0x3A, 0x00) => Some(Self::NoFilmHolder),
            (SenseKey::UnitAttention, 0x3F, 0x04) => Some(Self::Reset),
            (SenseKey::UnitAttention, 0x3F, 0x03) => Some(Self::MediumChanged),
            (SenseKey::UnitAttention, 0x28, 0x00) => Some(Self::HolderChanged),
            (key @ (SenseKey::NotReady | SenseKey::UnitAttention), ..) => {
                Some(Self::Other(key, sense.condition()))
            }
            _ => None,
        }
    }

    fn is_initializing(&self) -> bool {
        matches!(self, Self::Initializing)
    }

    /// Whether this state came from a unit attention
    ///
    /// The device queues these and reports one per command, clearing it as it goes, so a
    /// caller has to keep asking until it gets something else.
    fn is_unit_attention(&self) -> bool {
        matches!(
            self,
            Self::Reset
                | Self::MediumChanged
                | Self::HolderChanged
                | Self::Other(SenseKey::UnitAttention, _)
        )
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

    /// The named exception: this one AbortedCommand is transient, the rest are not
    #[test]
    fn recognizes_self_configuring_after_power_on() {
        assert_eq!(
            Status::from_sense(&sense(0x0B, 0x3E, 0x00)),
            Some(Status::Initializing)
        );
        assert_eq!(Status::from_sense(&sense(0x0B, 0x3E, 0x01)), None);
        assert_eq!(Status::from_sense(&sense(0x0B, 0x47, 0x00)), None);
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
                SenseKey::UnitAttention,
                AdditionalSenseCode::TargetOperatingConditionsHaveChanged
            ))
        );
    }

    /// Only the unit attentions clear themselves by being reported, so only those may be
    /// drained. Draining a NotReady would spin until the limit.
    #[test]
    fn only_unit_attentions_are_drainable() {
        for state in [Status::Reset, Status::MediumChanged, Status::HolderChanged] {
            assert!(state.is_unit_attention(), "{state:?}");
        }
        for state in [Status::Ready, Status::Initializing, Status::NoFilmHolder] {
            assert!(!state.is_unit_attention(), "{state:?}");
        }
        assert!(
            Status::Other(
                SenseKey::UnitAttention,
                AdditionalSenseCode::TargetOperatingConditionsHaveChanged
            )
            .is_unit_attention()
        );
        assert!(
            !Status::Other(
                SenseKey::NotReady,
                AdditionalSenseCode::LogicalUnitNotReadyCauseNotReportable
            )
            .is_unit_attention()
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
                SenseKey::NotReady,
                AdditionalSenseCode::LogicalUnitNotReadyCauseNotReportable
            ))
        );
    }
}
