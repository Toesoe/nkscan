//! The rows themselves: what each model can do, and what each adapter can do
//!
//! The only place in the library where a model name sits next to a behavior. Everything above
//! this asks [`Capabilities`] rather than matching on a model, which is what keeps adding a model
//! to a row here instead of a branch somewhere else.
//!
//! The model half is from the published specifications. The adapter half is from Nikon Scan's own
//! per-holder tables.

use super::{Capabilities, EjectAction, ExposureControl, FrameLocation, Ice, Resolution};
use crate::adapter::Adapter;
use crate::model::{Family, Model};

const MULTISAMPLE_FULL: &[u8] = &[1, 2, 4, 8, 16];
/// The two entry bodies average nothing in hardware
const MULTISAMPLE_NONE: &[u8] = &[1];

/// What the model alone decides
struct ModelRow {
    optical: u16,
    ladder: &'static [u16],
    depth: u8,
    multisample: &'static [u8],
    single_line: bool,
    kodachrome_ice: bool,
    exposure: ExposureControl,
}

/// The 4000 DPI bodies divide their sensor down this far
const LADDER_4000: &[u16] = &[4000, 2000, 1333, 1000, 800, 500, 250];
/// The medium format bodies stop at 666: 333 divides the sensor evenly and is still refused,
/// because the sensor bar reports that floor
const LADDER_MEDIUM: &[u16] = &[4000, 2000, 1333, 666];
/// The LS-40's sensor is 2900 DPI, so its rungs are its own rather than the 4000 ladder
const LADDER_2900: &[u16] = &[2900, 1450, 966, 725, 580, 362, 181];

fn per_model(model: Model) -> ModelRow {
    let host = ExposureControl::Host {
        lock_white_balance: true,
    };
    match model {
        Model::Ls9000 => ModelRow {
            optical: 4000,
            ladder: LADDER_MEDIUM,
            depth: 16,
            multisample: MULTISAMPLE_FULL,
            single_line: true,
            kodachrome_ice: true,
            exposure: host,
        },
        // The LS-9000 with a narrower converter and no Kodachrome profile
        Model::Ls8000 => ModelRow {
            optical: 4000,
            ladder: LADDER_MEDIUM,
            depth: 14,
            multisample: MULTISAMPLE_FULL,
            single_line: true,
            kodachrome_ice: false,
            exposure: host,
        },
        Model::Ls5000 => ModelRow {
            optical: 4000,
            ladder: LADDER_4000,
            depth: 16,
            multisample: MULTISAMPLE_FULL,
            single_line: false,
            kodachrome_ice: true,
            exposure: host,
        },
        Model::Ls4000 => ModelRow {
            optical: 4000,
            ladder: LADDER_4000,
            depth: 14,
            multisample: MULTISAMPLE_FULL,
            single_line: false,
            kodachrome_ice: false,
            exposure: host,
        },
        Model::Ls50 => ModelRow {
            optical: 4000,
            ladder: LADDER_4000,
            depth: 14,
            multisample: MULTISAMPLE_NONE,
            single_line: false,
            kodachrome_ice: true,
            // The firmware meters this one, which is why it has no white balance lock. Whether
            // that is true of the LS-5000 too is unsettled: docs/OPEN_QUESTIONS.md section 13.
            exposure: ExposureControl::Firmware,
        },
        Model::Ls40 => ModelRow {
            optical: 2900,
            ladder: LADDER_2900,
            depth: 12,
            multisample: MULTISAMPLE_NONE,
            single_line: false,
            kodachrome_ice: false,
            // Inherited from the LS-50, its direct successor. Unverified, like everything else
            // about this body.
            exposure: ExposureControl::Firmware,
        },
    }
}

/// What the adapter decides
struct AdapterRow {
    eject: EjectAction,
    overview: bool,
    frames: FrameLocation,
    batch: bool,
    strip_offset: bool,
}

fn per_adapter(family: Family, adapter: Adapter) -> AdapterRow {
    /// Nothing loaded, and nothing an unfitted adapter could do
    const INERT: AdapterRow = AdapterRow {
        eject: EjectAction::Unavailable,
        overview: false,
        frames: FrameLocation::Single,
        batch: false,
        strip_offset: false,
    };

    if !adapter.fits(family) {
        return INERT;
    }

    match (family, adapter) {
        (_, Adapter::None) => INERT,

        // --- 35 mm
        // Six mechanically fixed apertures, but the film within them may need shifting
        (_, Adapter::StripFilm) => AdapterRow {
            eject: EjectAction::EjectFilm,
            overview: true,
            frames: FrameLocation::Mechanical(6),
            batch: true,
            strip_offset: true,
        },
        (_, Adapter::RollFilm) => AdapterRow {
            eject: EjectAction::EjectFilm,
            overview: true,
            frames: FrameLocation::Reported,
            batch: true,
            strip_offset: true,
        },
        // Placed and taken out by hand, one at a time, with nothing to preview
        (_, Adapter::MountedSlide) => INERT,
        (_, Adapter::SlideFeeder) => AdapterRow {
            eject: EjectAction::FeedNextSlide,
            overview: false,
            frames: FrameLocation::Single,
            batch: true,
            strip_offset: false,
        },
        (_, Adapter::Ix240) => AdapterRow {
            eject: EjectAction::RewindFilm,
            overview: true,
            frames: FrameLocation::Reported,
            batch: true,
            strip_offset: false,
        },

        // --- medium format. Every holder comes out of the body the same way.
        (_, Adapter::Fh869Gr) => AdapterRow {
            eject: EjectAction::EjectHolder,
            overview: false,
            frames: FrameLocation::Detected,
            batch: false,
            strip_offset: false,
        },
        (_, Adapter::Fh869S) | (_, Adapter::Fh869G) => AdapterRow {
            eject: EjectAction::EjectHolder,
            overview: true,
            frames: FrameLocation::Detected,
            batch: true,
            strip_offset: true,
        },
        (_, Adapter::Fh835M)
        | (_, Adapter::Fh835S)
        | (_, Adapter::Fh869M)
        | (_, Adapter::Fh816)
        | (_, Adapter::Fh8G1) => AdapterRow {
            eject: EjectAction::EjectHolder,
            overview: true,
            // The aperture counts are not written down anywhere in this project, so they are
            // found rather than assumed. See `Adapter::Unknown` below for the same reason.
            frames: FrameLocation::Detected,
            batch: true,
            strip_offset: false,
        },

        // A holder the firmware named and this library does not recognize
        //
        // Not the inert row. On a medium format body every holder but the FH-869GR has an
        // overview and can be batched, and detection there reports a class rather than a part —
        // so an unrecognized holder is far more likely to be an ordinary one than to be the sole
        // exception. Falling back to "nothing works" would refuse the overview pass that the
        // LS-9000 has always had. On a 35 mm body there is no such majority, so it stays inert.
        (Family::MediumFormat, Adapter::Unknown(_)) => AdapterRow {
            eject: EjectAction::EjectHolder,
            overview: true,
            frames: FrameLocation::Detected,
            batch: true,
            strip_offset: false,
        },
        (Family::ThirtyFiveMm, Adapter::Unknown(_)) => INERT,
    }
}

/// The film a body takes, before an open unit narrows it to the loaded adapter
fn family_area_mm(family: Family) -> (f32, f32) {
    match family {
        // 56 mm across the bar, a whole 120 strip along the feed
        Family::MediumFormat => (56.9, 220.0),
        Family::ThirtyFiveMm => (25.1, 36.8),
    }
}

/// Everything this pairing can do
pub fn compute(model: Model, adapter: Adapter) -> Capabilities {
    let family = model.family();
    let m = per_model(model);
    let a = per_adapter(family, adapter);
    Capabilities {
        model,
        adapter,
        interface: model.interface(),
        resolution: Resolution {
            optical: m.optical,
            ladder: m.ladder.to_vec(),
        },
        depth: m.depth,
        multisample: m.multisample,
        single_line: m.single_line,
        ice: Ice {
            infrared: true,
            kodachrome: m.kodachrome_ice,
        },
        focus_range: None,
        exposure: m.exposure,
        eject: a.eject,
        overview: a.overview,
        frames: a.frames,
        batch: a.batch,
        strip_offset: a.strip_offset,
        max_area_mm: family_area_mm(family),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every pairing answers, including the ones that cannot physically happen
    #[test]
    fn the_table_is_total() {
        for model in Model::ALL {
            for adapter in Adapter::ALL {
                let caps = compute(model, adapter);
                assert_eq!(caps.model, model);
                assert_eq!(caps.adapter, adapter);
            }
            // And for a holder the firmware named but this library does not know
            let _ = compute(model, Adapter::Unknown(0x7F));
        }
    }

    /// An adapter that does not fit the body can do nothing, rather than borrowing another
    /// family's row
    #[test]
    fn an_adapter_from_the_wrong_family_is_inert() {
        let caps = compute(Model::Ls9000, Adapter::RollFilm);
        assert_eq!(caps.eject, EjectAction::Unavailable);
        assert!(!caps.overview);
        assert!(!caps.batch);
    }

    /// The owner's eject table, which is the clearest case for this not being a boolean
    #[test]
    fn eject_does_a_different_thing_on_each_adapter() {
        for (adapter, expected) in [
            (Adapter::StripFilm, EjectAction::EjectFilm),
            (Adapter::RollFilm, EjectAction::EjectFilm),
            (Adapter::Ix240, EjectAction::RewindFilm),
            (Adapter::SlideFeeder, EjectAction::FeedNextSlide),
            (Adapter::MountedSlide, EjectAction::Unavailable),
        ] {
            assert_eq!(compute(Model::Ls50, adapter).eject, expected, "{adapter:?}");
        }
        assert_eq!(
            compute(Model::Ls9000, Adapter::Fh869S).eject,
            EjectAction::EjectHolder
        );
    }

    /// The `*` column of the owner's holder list
    #[test]
    fn the_adapters_with_no_thumbnail_pass_are_the_starred_ones() {
        for adapter in [Adapter::MountedSlide, Adapter::SlideFeeder] {
            assert!(!compute(Model::Ls50, adapter).overview);
        }
        assert!(!compute(Model::Ls9000, Adapter::Fh869Gr).overview);
        // And it is the only medium format holder without one
        for adapter in [Adapter::Fh869S, Adapter::Fh869G, Adapter::Fh835S] {
            assert!(compute(Model::Ls9000, adapter).overview);
        }
    }

    /// The `+` column: these four hold film whose frames are not mechanically pinned
    #[test]
    fn the_adapters_that_reposition_film_are_the_plus_ones() {
        assert!(compute(Model::Ls50, Adapter::StripFilm).strip_offset);
        assert!(compute(Model::Ls50, Adapter::RollFilm).strip_offset);
        assert!(compute(Model::Ls9000, Adapter::Fh869S).strip_offset);
        assert!(compute(Model::Ls9000, Adapter::Fh869G).strip_offset);
        // and nothing else does
        assert!(!compute(Model::Ls50, Adapter::MountedSlide).strip_offset);
        assert!(!compute(Model::Ls9000, Adapter::Fh835S).strip_offset);
    }

    /// Batch is off on exactly the two the owner's table excludes
    #[test]
    fn batch_is_refused_on_the_mount_adapter_and_the_rotating_holder() {
        assert!(!compute(Model::Ls50, Adapter::MountedSlide).batch);
        assert!(!compute(Model::Ls9000, Adapter::Fh869Gr).batch);
        assert!(compute(Model::Ls50, Adapter::StripFilm).batch);
        assert!(compute(Model::Ls9000, Adapter::Fh869S).batch);
    }

    /// An unrecognized medium format holder must not lose the overview pass the LS-9000 has
    /// always had, since detection there reports a class rather than a part
    #[test]
    fn an_unrecognized_medium_format_holder_keeps_the_family_behavior() {
        let caps = compute(Model::Ls9000, Adapter::Unknown(2));
        assert!(caps.overview);
        assert_eq!(caps.eject, EjectAction::EjectHolder);
        assert!(caps.batch);
    }

    /// On a 35 mm body there is no such majority to fall back on
    #[test]
    fn an_unrecognized_thirty_five_mm_adapter_is_inert() {
        let caps = compute(Model::Ls50, Adapter::Unknown(0x43));
        assert!(!caps.overview);
        assert_eq!(caps.eject, EjectAction::Unavailable);
    }

    /// The owner's pixel-depth table
    #[test]
    fn each_model_reports_its_own_converter_width() {
        for (model, native) in [
            (Model::Ls9000, 16),
            (Model::Ls5000, 16),
            (Model::Ls8000, 14),
            (Model::Ls4000, 14),
            (Model::Ls50, 14),
            (Model::Ls40, 12),
        ] {
            assert_eq!(Capabilities::of(model).depth, native, "{model:?}");
        }
    }

    /// The LS-40 is the one body whose sensor is not 4000 DPI
    #[test]
    fn the_ls40_ladder_is_its_own_rather_than_the_4000_one() {
        let caps = Capabilities::of(Model::Ls40);
        assert_eq!(caps.resolution.optical, 2900);
        assert!(caps.allows_dpi(2900));
        assert!(!caps.allows_dpi(4000));
    }

    /// Multi-sample stops at the LS-4000: the two entry bodies average nothing
    #[test]
    fn only_the_models_with_hardware_averaging_offer_repeats() {
        for model in [Model::Ls8000, Model::Ls9000, Model::Ls4000, Model::Ls5000] {
            assert_eq!(Capabilities::of(model).multisample, &[1, 2, 4, 8, 16]);
        }
        for model in [Model::Ls50, Model::Ls40] {
            assert_eq!(Capabilities::of(model).multisample, &[1]);
        }
    }

    /// Only the two medium format bodies let a caller pick the readout
    #[test]
    fn the_single_line_readout_is_selectable_on_the_medium_format_bodies_only() {
        for model in [Model::Ls8000, Model::Ls9000] {
            assert!(Capabilities::of(model).single_line, "{model:?}");
        }
        for model in [Model::Ls5000, Model::Ls4000, Model::Ls50, Model::Ls40] {
            assert!(!Capabilities::of(model).single_line, "{model:?}");
        }
    }

    /// Infrared is on every one of them; the Kodachrome profile is on three
    #[test]
    fn kodachrome_ice_is_offered_on_three_models() {
        for model in Model::ALL {
            assert!(Capabilities::of(model).ice.infrared, "{model:?}");
        }
        let kodachrome: Vec<_> = Model::ALL
            .into_iter()
            .filter(|m| Capabilities::of(*m).ice.kodachrome)
            .collect();
        assert_eq!(kodachrome, vec![Model::Ls9000, Model::Ls5000, Model::Ls50]);
    }

    /// The LS-50 meters in firmware, which is a different claim from having no autoexposure
    #[test]
    fn the_ls50_meters_in_firmware_and_so_has_no_white_balance_lock() {
        assert_eq!(
            Capabilities::of(Model::Ls50).exposure,
            ExposureControl::Firmware
        );
        assert_eq!(
            Capabilities::of(Model::Ls9000).exposure,
            ExposureControl::Host {
                lock_white_balance: true
            }
        );
    }
}
