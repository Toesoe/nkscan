//! Why a request cannot be carried out, in a form a caller can branch on
//!
//! Replaces a refusal that was only a sentence. A sentence cannot be matched on, so a consumer
//! wanting to react — grey out a control, pick another resolution, file a bug — had to match
//! substrings of prose that no test pinned.
//!
//! The split that matters most is [`Reason::NotPresent`] against [`Reason::NotImplemented`]:
//! hardware that does not have a feature, versus hardware that has it and a driver that does not
//! drive it yet. The first is permanent and the second is a to-do, and the old stringly error
//! made them the same kind of thing with different wording.
//!
//! The message is generated from the data, in one place, so no driver phrases a refusal by naming
//! a model again.

use crate::adapter::Adapter;
use crate::capability::Capabilities;
use crate::model::Model;

/// A request the scanner in front of you cannot carry out
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unsupported {
    pub feature: Feature,
    pub reason: Reason,
}

/// A user-facing option, one per row of the capability table
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    Model,
    Adapter,
    Resolution,
    PixelDepth,
    Multisample,
    CcdMode,
    InfraredChannel,
    Focus,
    Exposure,
    WhiteBalanceLock,
    Eject,
    Overview,
    FramePlacement,
    BatchScan,
    StripFilmOffset,
    FrameWindow,
    KodachromeIce,
    Abort,
}

impl Feature {
    /// A stable machine-readable name, which is what a Python caller branches on
    pub fn slug(self) -> &'static str {
        match self {
            Feature::Model => "model",
            Feature::Adapter => "adapter",
            Feature::Resolution => "resolution",
            Feature::PixelDepth => "pixel_depth",
            Feature::Multisample => "multisample",
            Feature::CcdMode => "ccd_mode",
            Feature::InfraredChannel => "infrared_channel",
            Feature::Focus => "focus",
            Feature::Exposure => "exposure",
            Feature::WhiteBalanceLock => "white_balance_lock",
            Feature::Eject => "eject",
            Feature::Overview => "overview",
            Feature::FramePlacement => "frame_placement",
            Feature::BatchScan => "batch_scan",
            Feature::StripFilmOffset => "strip_film_offset",
            Feature::FrameWindow => "frame_window",
            Feature::KodachromeIce => "kodachrome_ice",
            Feature::Abort => "abort",
        }
    }

    /// How the message names it
    fn label(self) -> &'static str {
        match self {
            Feature::Model => "this model",
            Feature::Adapter => "this adapter",
            Feature::Resolution => "that resolution",
            Feature::PixelDepth => "that pixel depth",
            Feature::Multisample => "multi-sample scanning",
            Feature::CcdMode => "choosing the CCD readout mode",
            Feature::InfraredChannel => "the infrared channel",
            Feature::Focus => "that focus setpoint",
            Feature::Exposure => "that exposure control",
            Feature::WhiteBalanceLock => "the white balance lock",
            Feature::Eject => "ejecting",
            Feature::Overview => "the overview pass",
            Feature::FramePlacement => "that way of placing frames",
            Feature::BatchScan => "batch scanning",
            Feature::StripFilmOffset => "shifting the film along the strip",
            Feature::FrameWindow => "scanning part of a frame",
            Feature::KodachromeIce => "the Kodachrome infrared profile",
            Feature::Abort => "cancelling a pass",
        }
    }
}

/// Why the answer is no
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    /// The hardware does not have it, and no amount of work here would add it
    NotPresent { model: Model, adapter: Adapter },
    /// The hardware has it. This library does not drive it yet.
    ///
    /// `tracking` names where the missing work is written down, so a caller hitting this can go
    /// and read why rather than guessing whether it is worth asking for.
    NotImplemented { tracking: &'static str },
    /// The feature is there; the value asked for is outside what this unit offers
    OutOfRange { asked: u32, allowed: Allowed },
}

impl Reason {
    /// A stable machine-readable name, alongside [`Feature::slug`]
    pub fn slug(&self) -> &'static str {
        match self {
            Reason::NotPresent { .. } => "not_present",
            Reason::NotImplemented { .. } => "not_implemented",
            Reason::OutOfRange { .. } => "out_of_range",
        }
    }
}

/// What a value was allowed to be
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Allowed {
    Values(Vec<u32>),
    Range { min: u32, max: u32 },
}

impl std::fmt::Display for Allowed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Allowed::Values(values) => {
                let text: Vec<String> = values.iter().map(u32::to_string).collect();
                f.write_str(&text.join(", "))
            }
            Allowed::Range { min, max } => write!(f, "{min} to {max}"),
        }
    }
}

impl Unsupported {
    /// The hardware does not have it
    pub fn not_present(feature: Feature, capabilities: &Capabilities) -> Self {
        Self {
            feature,
            reason: Reason::NotPresent {
                model: capabilities.model,
                adapter: capabilities.adapter,
            },
        }
    }

    /// The model does not have it, whatever is loaded
    ///
    /// For the rows decided by the model rather than the adapter, where naming an adapter in the
    /// message would only suggest that a different one would help.
    pub fn not_on_model(feature: Feature, model: Model) -> Self {
        Self {
            feature,
            reason: Reason::NotPresent {
                model,
                adapter: Adapter::None,
            },
        }
    }

    /// The hardware has it and this library does not drive it yet
    pub fn not_implemented(feature: Feature, tracking: &'static str) -> Self {
        Self {
            feature,
            reason: Reason::NotImplemented { tracking },
        }
    }

    /// The value is outside what this unit offers
    pub fn out_of_range(feature: Feature, asked: u32, allowed: Allowed) -> Self {
        Self {
            feature,
            reason: Reason::OutOfRange { asked, allowed },
        }
    }

    /// One of a fixed set of values, which is the common shape of [`Self::out_of_range`]
    pub fn not_one_of(feature: Feature, asked: u32, values: impl IntoIterator<Item = u32>) -> Self {
        Self::out_of_range(
            feature,
            asked,
            Allowed::Values(values.into_iter().collect()),
        )
    }
}

impl std::fmt::Display for Unsupported {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.reason {
            Reason::NotPresent { model, adapter } => {
                write!(
                    f,
                    "the {} does not offer {}",
                    model.name(),
                    self.feature.label()
                )?;
                match adapter {
                    Adapter::None => Ok(()),
                    loaded => {
                        let name = loaded
                            .part_number(*model)
                            .map(str::to_owned)
                            .unwrap_or_else(|| loaded.to_string());
                        write!(f, " with {name} loaded")
                    }
                }
            }
            Reason::NotImplemented { tracking } => write!(
                f,
                "the scanner supports {}, but this library does not drive it yet ({tracking})",
                self.feature.label()
            ),
            Reason::OutOfRange { asked, allowed } => write!(
                f,
                "{} is not available: asked for {asked}, and this unit offers {allowed}",
                self.feature.label()
            ),
        }
    }
}

impl std::error::Error for Unsupported {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::table;

    #[test]
    fn a_missing_feature_names_the_model_and_the_adapter() {
        let caps = table::compute(Model::Ls50, Adapter::MountedSlide);
        let refusal = Unsupported::not_present(Feature::Overview, &caps);
        assert_eq!(
            refusal.to_string(),
            "the LS-50 ED does not offer the overview pass with MA-21 loaded"
        );
    }

    /// With nothing loaded there is no adapter to blame, so the message does not invent one
    #[test]
    fn a_missing_feature_with_no_adapter_names_only_the_model() {
        let caps = table::compute(Model::Ls50, Adapter::None);
        let refusal = Unsupported::not_present(Feature::Multisample, &caps);
        assert_eq!(
            refusal.to_string(),
            "the LS-50 ED does not offer multi-sample scanning"
        );
    }

    /// The distinction the old stringly error could not make
    #[test]
    fn absent_hardware_and_undriven_hardware_are_different_reasons() {
        let absent = Unsupported::not_present(
            Feature::Multisample,
            &table::compute(Model::Ls50, Adapter::None),
        );
        let undriven = Unsupported::not_implemented(Feature::Multisample, "docs/OPEN_QUESTIONS.md");
        assert_eq!(absent.reason.slug(), "not_present");
        assert_eq!(undriven.reason.slug(), "not_implemented");
        assert_ne!(absent.reason, undriven.reason);
        // and the same feature, so a caller branches on the reason rather than on the wording
        assert_eq!(absent.feature, undriven.feature);
    }

    #[test]
    fn an_out_of_range_value_reports_what_was_allowed() {
        let refusal = Unsupported::not_one_of(Feature::Multisample, 3, [1, 2, 4, 8, 16]);
        assert_eq!(
            refusal.to_string(),
            "multi-sample scanning is not available: asked for 3, and this unit offers 1, 2, 4, 8, 16"
        );
    }

    #[test]
    fn a_range_reads_as_a_range() {
        let refusal =
            Unsupported::out_of_range(Feature::Focus, 900, Allowed::Range { min: 0, max: 323 });
        assert_eq!(
            refusal.to_string(),
            "that focus setpoint is not available: asked for 900, and this unit offers 0 to 323"
        );
    }

    /// The slugs are the API, so they may not drift into prose
    #[test]
    fn every_feature_has_a_distinct_slug() {
        let features = [
            Feature::Model,
            Feature::Adapter,
            Feature::Resolution,
            Feature::PixelDepth,
            Feature::Multisample,
            Feature::CcdMode,
            Feature::InfraredChannel,
            Feature::Focus,
            Feature::Exposure,
            Feature::WhiteBalanceLock,
            Feature::Eject,
            Feature::Overview,
            Feature::FramePlacement,
            Feature::BatchScan,
            Feature::StripFilmOffset,
            Feature::FrameWindow,
            Feature::KodachromeIce,
            Feature::Abort,
        ];
        let mut slugs: Vec<_> = features.iter().map(|f| f.slug()).collect();
        slugs.sort_unstable();
        slugs.dedup();
        assert_eq!(slugs.len(), features.len());
        assert!(
            slugs
                .iter()
                .all(|s| s.chars().all(|c| c.is_ascii_lowercase() || c == '_'))
        );
    }
}
