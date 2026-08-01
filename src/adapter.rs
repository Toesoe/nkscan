//! The film adapters and holders, as one vocabulary across every model
//!
//! Each model recognises its adapter differently on the wire — the medium format bodies report a
//! class byte, the 35 mm bodies advertise a page code — but what they are recognising is the same
//! short list of physical objects. Naming that list once is what lets the capability table be
//! keyed on the adapter instead of on the model.
//!
//! Variants are named for what the adapter *is* rather than for its part number, because Nikon
//! gave the same object two numbers depending on the body it shipped with: the strip adapter is
//! an SA-21 on most bodies and an SA-20 on the LS-40. The part number comes back out of
//! [`Adapter::part_number`].

use crate::model::{Family, Model};

/// A film adapter or holder
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Adapter {
    /// Nothing loaded
    None,

    // --- 35 mm bodies: LS-40, LS-50, LS-4000, LS-5000
    /// Cut strip film, two to six frames. SA-21, or SA-20 on the LS-40.
    StripFilm,
    /// A whole uncut roll, up to forty frames. SA-30.
    RollFilm,
    /// One mounted slide, placed and removed by hand. MA-21, or MA-20 on the LS-40.
    MountedSlide,
    /// A hopper that feeds mounted slides one at a time. SF-210, or SF-200 on the LS-40.
    SlideFeeder,
    /// An IX240 (APS) cartridge, wound through by the body. IA-21, or IA-20 on the LS-40.
    Ix240,

    // --- medium format bodies: LS-8000, LS-9000
    /// FH-835M, 35 mm mounted slides on a medium format body
    Fh835M,
    /// FH-835S, 35 mm strip film on a medium format body
    Fh835S,
    /// FH-869S, 120/220 strip film
    Fh869S,
    /// FH-869G, 120/220 strip film between glass
    Fh869G,
    /// FH-869GR, 120/220 rotated, between glass
    Fh869Gr,
    /// FH-869M, 120/220 mounted
    Fh869M,
    /// FH-816, 16 mm film
    Fh816,
    /// FH-8G1, the medical slide holder. Takes a 35 mm mounted slide on its reverse.
    Fh8G1,

    /// The firmware named a holder this vocabulary does not know
    ///
    /// Carries the wire value so a bug report can say which one. Treated as the model's
    /// conservative floor rather than guessed at, because guessing is how the SA-21 and the SA-30
    /// came to be conflated in the first place.
    Unknown(u8),
}

impl Adapter {
    /// Every adapter this vocabulary names, `Unknown` aside
    pub const ALL: [Adapter; 14] = [
        Adapter::None,
        Adapter::StripFilm,
        Adapter::RollFilm,
        Adapter::MountedSlide,
        Adapter::SlideFeeder,
        Adapter::Ix240,
        Adapter::Fh835M,
        Adapter::Fh835S,
        Adapter::Fh869S,
        Adapter::Fh869G,
        Adapter::Fh869Gr,
        Adapter::Fh869M,
        Adapter::Fh816,
        Adapter::Fh8G1,
    ];

    /// Whether this adapter physically goes into a body of this family
    ///
    /// [`Adapter::None`] and [`Adapter::Unknown`] fit everything, since they are statements about
    /// the reading rather than about an object.
    pub fn fits(self, family: Family) -> bool {
        match self {
            Adapter::None | Adapter::Unknown(_) => true,
            Adapter::StripFilm
            | Adapter::RollFilm
            | Adapter::MountedSlide
            | Adapter::SlideFeeder
            | Adapter::Ix240 => family == Family::ThirtyFiveMm,
            Adapter::Fh835M
            | Adapter::Fh835S
            | Adapter::Fh869S
            | Adapter::Fh869G
            | Adapter::Fh869Gr
            | Adapter::Fh869M
            | Adapter::Fh816
            | Adapter::Fh8G1 => family == Family::MediumFormat,
        }
    }

    /// Nikon's part number for this adapter on this body
    ///
    /// `None` where the adapter does not fit the body, or where there is no object to name.
    ///
    /// The LS-40 column is Nikon's older numbering, one lower across the range: SA-20 for SA-21,
    /// MA-20 for MA-21, SF-200 for SF-210. That the IX240 adapter follows the same pattern is
    /// inferred from it rather than confirmed.
    pub fn part_number(self, model: Model) -> Option<&'static str> {
        if !self.fits(model.family()) {
            return None;
        }
        let older = model == Model::Ls40;
        Some(match self {
            Adapter::None | Adapter::Unknown(_) => return None,
            Adapter::StripFilm if older => "SA-20",
            Adapter::StripFilm => "SA-21",
            Adapter::RollFilm => "SA-30",
            Adapter::MountedSlide if older => "MA-20",
            Adapter::MountedSlide => "MA-21",
            Adapter::SlideFeeder if older => "SF-200",
            Adapter::SlideFeeder => "SF-210",
            Adapter::Ix240 if older => "IA-20",
            Adapter::Ix240 => "IA-21",
            Adapter::Fh835M => "FH-835M",
            Adapter::Fh835S => "FH-835S",
            Adapter::Fh869S => "FH-869S",
            Adapter::Fh869G => "FH-869G",
            Adapter::Fh869Gr => "FH-869GR",
            Adapter::Fh869M => "FH-869M",
            Adapter::Fh816 => "FH-816",
            Adapter::Fh8G1 => "FH-8G1",
        })
    }
}

/// What the adapter is, without needing to know which body it is in
///
/// [`part_number`](Adapter::part_number) is the one to print where the model is known, since a
/// user reads the number off the object in their hand.
impl std::fmt::Display for Adapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Adapter::None => "no adapter",
            Adapter::StripFilm => "35 mm strip film adapter",
            Adapter::RollFilm => "35 mm roll film adapter",
            Adapter::MountedSlide => "35 mm mounted slide adapter",
            Adapter::SlideFeeder => "35 mm slide feeder",
            Adapter::Ix240 => "IX240 cartridge adapter",
            Adapter::Fh835M => "35 mm mounted slide holder",
            Adapter::Fh835S => "35 mm strip film holder",
            Adapter::Fh869S => "120/220 strip film holder",
            Adapter::Fh869G => "120/220 strip film holder with glass",
            Adapter::Fh869Gr => "120/220 rotated film holder with glass",
            Adapter::Fh869M => "120/220 mounted film holder",
            Adapter::Fh816 => "16 mm film holder",
            Adapter::Fh8G1 => "medical slide holder",
            Adapter::Unknown(code) => return write!(f, "unrecognized holder {code:#04x}"),
        };
        f.write_str(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every adapter belongs to exactly one family, so a table keyed on the family is total
    #[test]
    fn each_adapter_fits_one_family_only() {
        for adapter in Adapter::ALL {
            if matches!(adapter, Adapter::None) {
                continue;
            }
            let families = [Family::MediumFormat, Family::ThirtyFiveMm]
                .into_iter()
                .filter(|f| adapter.fits(*f))
                .count();
            assert_eq!(families, 1, "{adapter:?} fits {families} families");
        }
    }

    /// The LS-40 takes the same objects under Nikon's older numbers
    #[test]
    fn the_ls40_reports_the_older_part_numbers() {
        for (adapter, ls40, ls50) in [
            (Adapter::StripFilm, "SA-20", "SA-21"),
            (Adapter::MountedSlide, "MA-20", "MA-21"),
            (Adapter::SlideFeeder, "SF-200", "SF-210"),
        ] {
            assert_eq!(adapter.part_number(Model::Ls40), Some(ls40));
            assert_eq!(adapter.part_number(Model::Ls50), Some(ls50));
        }
    }

    /// The SA-30 is the one 35 mm adapter Nikon did not renumber
    #[test]
    fn the_roll_adapter_has_one_number_everywhere() {
        for model in [Model::Ls40, Model::Ls50, Model::Ls4000, Model::Ls5000] {
            assert_eq!(Adapter::RollFilm.part_number(model), Some("SA-30"));
        }
    }

    /// Asking for a medium format holder on a 35 mm body is a question with no answer
    #[test]
    fn an_adapter_that_does_not_fit_has_no_part_number() {
        assert_eq!(Adapter::Fh869S.part_number(Model::Ls50), None);
        assert_eq!(Adapter::StripFilm.part_number(Model::Ls9000), None);
    }

    #[test]
    fn nothing_loaded_and_nothing_recognized_name_no_part() {
        assert_eq!(Adapter::None.part_number(Model::Ls9000), None);
        assert_eq!(Adapter::Unknown(0x43).part_number(Model::Ls50), None);
    }

    /// The wire value survives into the message, since it is the whole point of the variant
    #[test]
    fn an_unknown_holder_names_its_code() {
        assert_eq!(
            Adapter::Unknown(0x43).to_string(),
            "unrecognized holder 0x43"
        );
    }
}
