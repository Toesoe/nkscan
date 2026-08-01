//! Settings that have already been checked against the scanner they are for
//!
//! The fields are private and nothing outside this module can build one, so a driver cannot be
//! handed a setting the capability table did not allow. That is the whole mechanism: validation
//! stops being a call a caller might skip and becomes something the type system did.
//!
//! What is checked here is what the *hardware* offers. A driver may still refuse with
//! [`Reason::NotImplemented`](super::unsupported::Reason::NotImplemented) for something the
//! hardware has and this library does not drive yet — an LS-50 with a strip adapter really does
//! have a thumbnail pass, and no overview code is written for it. Those two refusals are
//! different answers to a caller and are deliberately raised in different places.

use super::unsupported::{Feature, Unsupported};
use super::{Capabilities, CcdMode, ExposureControl, FrameLocation, Overview};
use crate::session::{Exposure, FocusMode, FrameSettings, Placement, Prepare};
use std::time::Duration;

/// One frame's pass, checked against the unit it is for
///
/// Built only by [`Capabilities::resolve_frame`]. Read-only from anywhere else.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedFrame {
    dpi: u16,
    ir: bool,
    focus: FocusMode,
    multisample: u8,
    ccd_mode: Option<CcdMode>,
}

impl ResolvedFrame {
    pub fn dpi(&self) -> u16 {
        self.dpi
    }
    pub fn ir(&self) -> bool {
        self.ir
    }
    pub fn focus(&self) -> FocusMode {
        self.focus
    }
    pub fn multisample(&self) -> u8 {
        self.multisample
    }
    /// `None` on a model that offers no choice of readout
    pub fn ccd_mode(&self) -> Option<CcdMode> {
        self.ccd_mode
    }
    /// Whether the slower single-line readout was asked for
    pub fn single_line(&self) -> bool {
        self.ccd_mode == Some(CcdMode::SingleLine)
    }
}

/// How the frames get located, checked against the loaded adapter
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedPlacement {
    /// An overview pass, with the frames found in it
    Detect { frames: usize },
    /// Placed arithmetically along the feed
    Pitch {
        frames: Option<u32>,
        pitch_mm: Option<f32>,
        offsets_mm: Vec<f32>,
    },
    /// The transport's own sensed table
    Sensed { frames: Option<u32> },
}

/// Where the gain comes from, with the model's own answer already folded in
///
/// A driver no longer has to know whether it meters: asking for `Auto` on a firmware-metered
/// model resolves to [`FirmwareMetered`](Self::FirmwareMetered) here rather than every driver
/// working it out again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedExposure {
    HostMetered { lock_white_balance: bool },
    FirmwareMetered,
    Fixed { visible: [u32; 3], ir: Option<u32> },
}

/// A prepare, checked against the unit it is for
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedPrepare {
    placement: ResolvedPlacement,
    exposure: ResolvedExposure,
    wait_for_media: Duration,
}

impl ResolvedPrepare {
    pub fn placement(&self) -> &ResolvedPlacement {
        &self.placement
    }
    pub fn exposure(&self) -> ResolvedExposure {
        self.exposure
    }
    pub fn wait_for_media(&self) -> Duration {
        self.wait_for_media
    }
}

impl Capabilities {
    /// Check one frame's settings against this unit
    ///
    /// The single place a per-frame refusal is written.
    pub fn resolve_frame(&self, request: &FrameSettings) -> Result<ResolvedFrame, Unsupported> {
        if !self.allows_dpi(request.dpi) {
            return Err(Unsupported::not_one_of(
                Feature::Resolution,
                u32::from(request.dpi),
                self.resolution.ladder.iter().map(|&d| u32::from(d)),
            ));
        }

        if !self.multisample.contains(&request.multisample) {
            // A model with no hardware averaging does not have the feature at all, which is a
            // different answer from asking a model that has it for a count it does not do
            return Err(if self.multisample == [1] {
                Unsupported::not_on_model(Feature::Multisample, self.model)
            } else {
                Unsupported::not_one_of(
                    Feature::Multisample,
                    u32::from(request.multisample),
                    self.multisample.iter().map(|&n| u32::from(n)),
                )
            });
        }

        let ccd_mode = match request.single_line {
            true if !self.allows_ccd_mode(CcdMode::SingleLine) => {
                return Err(Unsupported::not_on_model(Feature::CcdMode, self.model));
            }
            true => Some(CcdMode::SingleLine),
            // Whatever the model does when given no choice, which is not a mode a caller picked
            false if self.ccd_modes.is_empty() => None,
            false => Some(CcdMode::ThreeLine),
        };

        if request.window.is_some() {
            return Err(Unsupported::not_implemented(
                Feature::FrameWindow,
                "no capture crops a frame, and the alignment a window has to keep is per model",
            ));
        }

        Ok(ResolvedFrame {
            dpi: request.dpi,
            ir: request.ir,
            focus: request.focus,
            multisample: request.multisample,
            ccd_mode,
        })
    }

    /// Check a prepare against this unit and the adapter it has loaded
    pub fn resolve_prepare(&self, request: &Prepare) -> Result<ResolvedPrepare, Unsupported> {
        let placement = match &request.placement {
            Placement::Detect { frames } => {
                if self.overview == Overview::Unavailable {
                    return Err(Unsupported::not_present(Feature::Overview, self));
                }
                ResolvedPlacement::Detect { frames: *frames }
            }
            Placement::Sensed { frames } => {
                if self.frames != FrameLocation::Reported {
                    return Err(Unsupported::not_present(Feature::FramePlacement, self));
                }
                ResolvedPlacement::Sensed { frames: *frames }
            }
            Placement::Pitch {
                frames,
                pitch_mm,
                offsets_mm,
            } => ResolvedPlacement::Pitch {
                frames: *frames,
                pitch_mm: *pitch_mm,
                offsets_mm: offsets_mm.clone(),
            },
        };

        let exposure = match (request.exposure, self.exposure) {
            (Exposure::Auto { lock_white_balance }, ExposureControl::Host { .. }) => {
                if lock_white_balance
                    && self.exposure
                        != (ExposureControl::Host {
                            lock_white_balance: true,
                        })
                {
                    return Err(Unsupported::not_present(Feature::WhiteBalanceLock, self));
                }
                ResolvedExposure::HostMetered { lock_white_balance }
            }
            // Nothing host-side decides the gain, so there is no knob to hold the channels with
            (
                Exposure::Auto {
                    lock_white_balance: true,
                },
                ExposureControl::Firmware,
            ) => {
                return Err(Unsupported::not_on_model(
                    Feature::WhiteBalanceLock,
                    self.model,
                ));
            }
            (Exposure::Auto { .. }, ExposureControl::Firmware) => ResolvedExposure::FirmwareMetered,
            // A firmware-metered model drives its infrared off a zeroed field rather than
            // metering it, so a value for it would be quietly ignored
            (Exposure::Fixed { ir: Some(_), .. }, ExposureControl::Firmware) => {
                return Err(Unsupported::not_on_model(Feature::Exposure, self.model));
            }
            (Exposure::Fixed { visible, ir }, _) => ResolvedExposure::Fixed { visible, ir },
        };

        Ok(ResolvedPrepare {
            placement,
            exposure,
            wait_for_media: request.wait_for_media,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::Adapter;
    use crate::capability::table;
    use crate::capability::unsupported::Reason;
    use crate::model::Model;

    fn frame() -> FrameSettings {
        FrameSettings::default()
    }

    fn caps(model: Model, adapter: Adapter) -> Capabilities {
        table::compute(model, adapter)
    }

    #[test]
    fn a_resolution_on_the_ladder_resolves() {
        let resolved = caps(Model::Ls9000, Adapter::Fh869S)
            .resolve_frame(&frame())
            .expect("4000 is on every ladder");
        assert_eq!(resolved.dpi(), 4000);
    }

    /// The LS-40's sensor is 2900 DPI, so the default 4000 is not a rung it has
    #[test]
    fn a_resolution_off_this_models_ladder_is_refused() {
        let refusal = caps(Model::Ls40, Adapter::StripFilm)
            .resolve_frame(&frame())
            .expect_err("4000 is above a 2900 DPI sensor");
        assert_eq!(refusal.feature, Feature::Resolution);
        assert_eq!(refusal.reason.slug(), "out_of_range");
    }

    /// The same request, two different answers, which is the point of the reason split
    #[test]
    fn multisample_is_absent_on_one_model_and_merely_the_wrong_count_on_another() {
        let mut request = frame();
        request.multisample = 4;

        // No hardware averaging at all
        let refusal = caps(Model::Ls50, Adapter::StripFilm)
            .resolve_frame(&request)
            .expect_err("the LS-50 averages nothing");
        assert_eq!(refusal.feature, Feature::Multisample);
        assert_eq!(refusal.reason.slug(), "not_present");

        // Has it, and 4 is one of the counts
        caps(Model::Ls9000, Adapter::Fh869S)
            .resolve_frame(&request)
            .expect("the LS-9000 averages four");

        // Has it, and 3 is not one of the counts
        request.multisample = 3;
        let refusal = caps(Model::Ls9000, Adapter::Fh869S)
            .resolve_frame(&request)
            .expect_err("three is not a repeat count");
        assert_eq!(refusal.reason.slug(), "out_of_range");
    }

    #[test]
    fn the_single_line_readout_is_only_offered_where_it_exists() {
        let mut request = frame();
        request.single_line = true;

        let resolved = caps(Model::Ls9000, Adapter::Fh869S)
            .resolve_frame(&request)
            .expect("the LS-9000 has both readouts");
        assert_eq!(resolved.ccd_mode(), Some(CcdMode::SingleLine));
        assert!(resolved.single_line());

        let refusal = caps(Model::Ls50, Adapter::StripFilm)
            .resolve_frame(&request)
            .expect_err("no choice of readout on a 35 mm body");
        assert_eq!(refusal.feature, Feature::CcdMode);
    }

    /// A model with no choice reports none, rather than claiming the three-line mode was picked
    #[test]
    fn a_model_with_no_readout_choice_resolves_to_no_mode() {
        let resolved = caps(Model::Ls50, Adapter::StripFilm)
            .resolve_frame(&frame())
            .expect("resolves");
        assert_eq!(resolved.ccd_mode(), None);
        assert!(!resolved.single_line());
    }

    #[test]
    fn cropping_is_refused_as_unimplemented_rather_than_as_absent() {
        let mut request = frame();
        request.window = Some((0.0, 0.0, 0.5, 0.5));
        let refusal = caps(Model::Ls9000, Adapter::Fh869S)
            .resolve_frame(&request)
            .expect_err("no model crops");
        assert_eq!(refusal.feature, Feature::FrameWindow);
        assert_eq!(refusal.reason.slug(), "not_implemented");
    }

    fn prepare(placement: Placement, exposure: Exposure) -> Prepare {
        Prepare {
            placement,
            exposure,
            wait_for_media: Duration::from_secs(0),
        }
    }

    /// An adapter with no thumbnail pass cannot have its frames found in one
    #[test]
    fn detect_is_refused_on_an_adapter_with_no_overview() {
        let request = prepare(
            Placement::Detect { frames: 4 },
            Exposure::Auto {
                lock_white_balance: false,
            },
        );
        let refusal = caps(Model::Ls50, Adapter::MountedSlide)
            .resolve_prepare(&request)
            .expect_err("a mounted slide has no overview");
        assert_eq!(refusal.feature, Feature::Overview);
        assert_eq!(refusal.reason.slug(), "not_present");
    }

    /// The adapter does have one; this library has not written the pass. A different answer, and
    /// deliberately left to the driver rather than decided here.
    #[test]
    fn detect_passes_resolution_where_the_adapter_has_an_overview() {
        let request = prepare(
            Placement::Detect { frames: 4 },
            Exposure::Auto {
                lock_white_balance: false,
            },
        );
        caps(Model::Ls50, Adapter::StripFilm)
            .resolve_prepare(&request)
            .expect("the SA-21 has a thumbnail pass, whatever this library does with it");
    }

    #[test]
    fn sensed_needs_a_transport_that_reports_a_table() {
        let request = prepare(
            Placement::Sensed { frames: None },
            Exposure::Auto {
                lock_white_balance: false,
            },
        );
        caps(Model::Ls5000, Adapter::RollFilm)
            .resolve_prepare(&request)
            .expect("the SA-30 reports one");
        let refusal = caps(Model::Ls9000, Adapter::Fh869S)
            .resolve_prepare(&request)
            .expect_err("no medium format holder reports one");
        assert_eq!(refusal.feature, Feature::FramePlacement);
    }

    /// Firmware metering is folded in here, so a driver never works it out again
    #[test]
    fn auto_exposure_resolves_to_whichever_side_meters() {
        let request = prepare(
            Placement::Pitch {
                frames: None,
                pitch_mm: None,
                offsets_mm: Vec::new(),
            },
            Exposure::Auto {
                lock_white_balance: false,
            },
        );
        assert_eq!(
            caps(Model::Ls9000, Adapter::Fh869S)
                .resolve_prepare(&request)
                .unwrap()
                .exposure(),
            ResolvedExposure::HostMetered {
                lock_white_balance: false
            }
        );
        assert_eq!(
            caps(Model::Ls50, Adapter::StripFilm)
                .resolve_prepare(&request)
                .unwrap()
                .exposure(),
            ResolvedExposure::FirmwareMetered
        );
    }

    /// Nothing host-side decides the gain, so there is no knob to hold the channels together
    #[test]
    fn the_white_balance_lock_is_refused_where_the_firmware_meters() {
        let request = prepare(
            Placement::Pitch {
                frames: None,
                pitch_mm: None,
                offsets_mm: Vec::new(),
            },
            Exposure::Auto {
                lock_white_balance: true,
            },
        );
        let refusal = caps(Model::Ls50, Adapter::StripFilm)
            .resolve_prepare(&request)
            .expect_err("the LS-50 has no lock");
        assert_eq!(refusal.feature, Feature::WhiteBalanceLock);
        caps(Model::Ls9000, Adapter::Fh869S)
            .resolve_prepare(&request)
            .expect("the LS-9000 meters host-side and can hold the ratios");
    }

    /// The infrared plane is driven off a zeroed field there, so a value for it means nothing
    #[test]
    fn a_separate_infrared_gain_is_refused_where_the_firmware_meters() {
        let request = prepare(
            Placement::Pitch {
                frames: None,
                pitch_mm: None,
                offsets_mm: Vec::new(),
            },
            Exposure::Fixed {
                visible: [1, 2, 3],
                ir: Some(4),
            },
        );
        let refusal = caps(Model::Ls50, Adapter::StripFilm)
            .resolve_prepare(&request)
            .expect_err("the LS-50 takes three gains");
        assert_eq!(refusal.feature, Feature::Exposure);
        caps(Model::Ls5000, Adapter::RollFilm)
            .resolve_prepare(&request)
            .expect("the LS-5000 meters infrared");
    }

    /// A caller cannot build one of these, which is what makes the check impossible to skip
    #[test]
    fn a_resolved_setting_carries_what_was_asked_for() {
        let mut request = frame();
        request.ir = true;
        request.focus = FocusMode::At(320);
        let resolved = caps(Model::Ls9000, Adapter::Fh869S)
            .resolve_frame(&request)
            .expect("resolves");
        assert!(resolved.ir());
        assert_eq!(resolved.focus(), FocusMode::At(320));
        assert!(matches!(
            Unsupported::not_on_model(Feature::Focus, Model::Ls9000).reason,
            Reason::NotPresent { .. }
        ));
    }
}
