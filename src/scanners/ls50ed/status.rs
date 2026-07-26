//! Interpreting TEST UNIT READY sense data for the LS-50
//!
//! NOT READY and UNIT ATTENTION are transient by definition (SPC-2 Table 69), so anything
//! under those keys is a readiness state, named or not.

use crate::scsi::{SenseData, SenseKey, asc::AdditionalSenseCode};

/// Scanner state, as reported by TEST UNIT READY
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ready,
    /// 02 04/01, warming up
    Initializing,
    /// 02 04/02. Does not clear on its own: the scanner wants
    /// [`warm_up`](super::Ls50ed::warm_up) to run the self-test.
    NeedsInit,
    /// 02 3A/00. The adapter is still reported by
    /// [`holder`](crate::scanners::FilmHolder::holder).
    NoFilm,
    /// 02 05/00
    Ejecting,
    /// 06 29/00, power-on or bus reset
    Reset,
    /// 06 28/00, film loaded or removed
    MediumChanged,
    /// 06 3F/03, adapter swapped, so anything derived from VPD is stale
    HolderChanged,
    /// A NotReady or UnitAttention we haven't named. Still not a hard failure, and the
    /// key tells the two apart.
    Other(SenseKey, AdditionalSenseCode),
}

impl Status {
    /// `None` for sense keys that are genuine errors, which the caller passes on as `Err`
    pub(crate) fn from_sense(sense: &SenseData) -> Option<Self> {
        match (sense.sense_key(), sense.asc, sense.ascq) {
            (SenseKey::NotReady, 0x04, 0x01) => Some(Self::Initializing),
            (SenseKey::NotReady, 0x04, 0x02) => Some(Self::NeedsInit),
            (SenseKey::NotReady, 0x3A, 0x00) => Some(Self::NoFilm),
            (SenseKey::NotReady, 0x05, 0x00) => Some(Self::Ejecting),
            (SenseKey::UnitAttention, 0x29, 0x00) => Some(Self::Reset),
            (SenseKey::UnitAttention, 0x28, 0x00) => Some(Self::MediumChanged),
            (SenseKey::UnitAttention, 0x3F, 0x03) => Some(Self::HolderChanged),
            (key @ (SenseKey::NotReady | SenseKey::UnitAttention), ..) => {
                Some(Self::Other(key, sense.condition()))
            }
            _ => None,
        }
    }

    /// The device queues these and reports one per command, clearing it as it goes, so a
    /// caller has to keep asking until it gets something else
    pub fn is_unit_attention(self) -> bool {
        matches!(
            self,
            Self::Reset
                | Self::MediumChanged
                | Self::HolderChanged
                | Self::Other(SenseKey::UnitAttention, _)
        )
    }

    /// Whether polling can still resolve this. NeedsInit needs the self-test and NoFilm
    /// needs film, so waiting on either just burns the retry budget.
    pub fn is_transient(self) -> bool {
        !matches!(self, Self::Ready | Self::NeedsInit | Self::NoFilm)
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
    fn recognizes_the_not_ready_states() {
        for (asc, ascq, expected) in [
            (0x04, 0x01, Status::Initializing),
            (0x04, 0x02, Status::NeedsInit),
            (0x3A, 0x00, Status::NoFilm),
            (0x05, 0x00, Status::Ejecting),
        ] {
            assert_eq!(Status::from_sense(&sense(0x02, asc, ascq)), Some(expected));
        }
    }

    #[test]
    fn recognizes_cold_start_unit_attentions() {
        // A real cold start drains these in order before GOOD
        for (asc, ascq, expected) in [
            (0x29, 0x00, Status::Reset),
            (0x28, 0x00, Status::MediumChanged),
            (0x3F, 0x03, Status::HolderChanged),
        ] {
            let status = Status::from_sense(&sense(0x06, asc, ascq));
            assert_eq!(status, Some(expected));
            assert!(status.unwrap().is_unit_attention());
        }
    }

    /// 3F/04 is another Coolscan's reset code, not ours, so it lands in `Other`
    #[test]
    fn another_models_reset_code_is_not_our_named_reset() {
        assert_eq!(
            Status::from_sense(&sense(0x06, 0x3F, 0x04)),
            Some(Status::Other(
                SenseKey::UnitAttention,
                AdditionalSenseCode::Other(0x3F, 0x04)
            ))
        );
    }

    #[test]
    fn unrecognized_sense_is_not_a_state() {
        assert_eq!(Status::from_sense(&sense(0x05, 0x24, 0x00)), None);
    }

    /// Draining a NotReady would spin until the limit, so only unit attentions qualify
    #[test]
    fn only_unit_attentions_are_drainable() {
        for state in [Status::Ready, Status::Initializing, Status::NoFilm] {
            assert!(!state.is_unit_attention(), "{state:?}");
        }
        assert!(
            !Status::Other(
                SenseKey::NotReady,
                AdditionalSenseCode::LogicalUnitNotReadyCauseNotReportable
            )
            .is_unit_attention()
        );
    }

    #[test]
    fn only_ready_needs_init_and_no_film_end_a_poll() {
        for state in [Status::Ready, Status::NeedsInit, Status::NoFilm] {
            assert!(!state.is_transient(), "{state:?}");
        }
        for state in [Status::Initializing, Status::Ejecting, Status::Reset] {
            assert!(state.is_transient(), "{state:?}");
        }
    }
}
