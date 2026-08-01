//! Which scanners this library has vocabulary for
//!
//! The list is closed. Nikon Scan drove six Coolscans and there will never be a seventh, so a
//! model is an enum variant rather than anything open-ended.
//!
//! Knowing a model is not the same as being able to drive one: [`Model::is_driven`] says which
//! have a wire implementation behind them. The rest are here so that enumeration can name what
//! it found, and so the capability tables can answer for hardware nobody has plugged in yet.

use crate::scanners::{ls50, ls5000};

/// Nikon's USB vendor id, which every Coolscan on the bus answers to
const USB_VENDOR: u16 = 0x04B0;

/// The LS-40's product id
///
/// It has no driver module to keep this in. Reported from a live descriptor as
/// `USB\VID_04b0&PID_4000`, and it sits directly below the LS-50's 0x4001 and the LS-5000's
/// 0x4002.
const LS40_PRODUCT: u16 = 0x4000;

/// A scanner Nikon Scan drove
///
/// Nikon named these twice: an `LS-` number on the chassis and a Coolscan number in the
/// marketing. The variants use the `LS-` name because that is what the unit answers INQUIRY
/// with, and the doc comments give the other one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Model {
    /// Coolscan 8000 ED
    Ls8000,
    /// Coolscan 9000 ED
    Ls9000,
    /// Coolscan 5000 ED
    Ls5000,
    /// Coolscan 4000 ED
    Ls4000,
    /// Coolscan V ED. The owner's tables call this one "V".
    Ls50,
    /// Coolscan IV ED. The owner's tables call this one "IV".
    Ls40,
}

/// What film a body takes, which is what decides which adapters fit it
///
/// The capability table keys its adapter half on this rather than on the model, because every
/// 35 mm body takes the same five adapters and every medium format body takes the same eight
/// holders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// 120/220 on a removable holder, and 35 mm on a carrier
    MediumFormat,
    /// 35 mm through an adapter that the body drives
    ThirtyFiveMm,
}

/// The link the scanner is on
///
/// Also what decides how enumeration finds it: a FireWire unit appears as a SCSI device and is
/// found by asking each node who it is, while a USB unit is found by its vendor and product ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interface {
    Firewire400,
    Usb2,
    Usb11,
}

/// Which wire dialect a model speaks, and so which driver serves it
///
/// A different axis from [`Family`]: the LS-50 and the LS-5000 are both 35 mm bodies and take
/// the same adapters, yet their command encodings differ enough to need separate drivers. Adding
/// a model is a line here once someone has established which dialect it speaks, not a new
/// directory under `scanners`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Protocol {
    Ls9000,
    Ls50,
    Ls5000,
}

impl Model {
    /// Every model this library has vocabulary for
    pub const ALL: [Model; 6] = [
        Model::Ls8000,
        Model::Ls9000,
        Model::Ls5000,
        Model::Ls4000,
        Model::Ls50,
        Model::Ls40,
    ];

    /// The stable half of a device id, and what a caller names a model by
    pub fn slug(self) -> &'static str {
        match self {
            Model::Ls8000 => "ls8000",
            Model::Ls9000 => "ls9000",
            Model::Ls5000 => "ls5000",
            Model::Ls4000 => "ls4000",
            Model::Ls50 => "ls50",
            Model::Ls40 => "ls40",
        }
    }

    /// How the unit introduces itself in the INQUIRY product string
    ///
    /// Matched as a substring, so the exact trailing text does not have to be right. The three
    /// driven models are transcribed from real captures; the other three follow the same pattern
    /// and have not been seen on a wire.
    pub fn name(self) -> &'static str {
        match self {
            Model::Ls8000 => "LS-8000 ED",
            Model::Ls9000 => "LS-9000 ED",
            Model::Ls5000 => "LS-5000 ED",
            Model::Ls4000 => "LS-4000 ED",
            Model::Ls50 => "LS-50 ED",
            Model::Ls40 => "LS-40 ED",
        }
    }

    pub fn family(self) -> Family {
        match self {
            Model::Ls8000 | Model::Ls9000 => Family::MediumFormat,
            Model::Ls5000 | Model::Ls4000 | Model::Ls50 | Model::Ls40 => Family::ThirtyFiveMm,
        }
    }

    pub fn interface(self) -> Interface {
        match self {
            Model::Ls8000 | Model::Ls9000 | Model::Ls4000 => Interface::Firewire400,
            Model::Ls5000 | Model::Ls50 => Interface::Usb2,
            Model::Ls40 => Interface::Usb11,
        }
    }

    /// The USB ids this model answers to
    ///
    /// `None` on a FireWire model, which is found by sweeping SCSI nodes instead.
    pub fn usb_ids(self) -> Option<(u16, u16)> {
        match self {
            Model::Ls50 => Some((ls50::VENDOR_ID, ls50::PRODUCT_ID)),
            Model::Ls5000 => Some((ls5000::VENDOR_ID, ls5000::PRODUCT_ID)),
            Model::Ls40 => Some((USB_VENDOR, LS40_PRODUCT)),
            Model::Ls8000 | Model::Ls9000 | Model::Ls4000 => None,
        }
    }

    /// Which driver serves this model, or `None` where none has been written
    ///
    /// The single source of truth for whether a unit can be opened. Giving a model a driver is
    /// flipping one arm of this match.
    pub(crate) fn protocol(self) -> Option<Protocol> {
        match self {
            Model::Ls9000 => Some(Protocol::Ls9000),
            Model::Ls50 => Some(Protocol::Ls50),
            Model::Ls5000 => Some(Protocol::Ls5000),
            // On the strength of the family and the interface these are most likely
            // `Ls9000`, `Ls5000` and `Ls50` in turn. Nobody has put one on a wire to
            // check, and a wrong guess here drives the wrong command set, so they stay undriven.
            Model::Ls8000 | Model::Ls4000 | Model::Ls40 => None,
        }
    }

    /// Whether this library has a driver for it
    ///
    /// A model that is not driven can still be enumerated and can still answer what it would be
    /// capable of; it just cannot be opened.
    pub fn is_driven(self) -> bool {
        self.protocol().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_are_unique_and_name_one_model_each() {
        for model in Model::ALL {
            let matches: Vec<_> = Model::ALL
                .into_iter()
                .filter(|m| m.slug() == model.slug())
                .collect();
            assert_eq!(matches, vec![model], "{} names more than one", model.slug());
        }
    }

    /// A slug is half of a device id, so it must not contain the separator
    #[test]
    fn a_slug_has_no_at_sign_in_it() {
        for model in Model::ALL {
            assert!(!model.slug().contains('@'), "{}", model.slug());
        }
    }

    /// The USB models are exactly the ones enumeration can find by id, and the FireWire ones are
    /// exactly those it has to sweep SCSI nodes for. Nothing may fall between the two.
    #[test]
    fn a_model_carries_usb_ids_exactly_when_it_is_on_usb() {
        for model in Model::ALL {
            let usb = matches!(model.interface(), Interface::Usb2 | Interface::Usb11);
            assert_eq!(
                model.usb_ids().is_some(),
                usb,
                "{} is on a {:?} link",
                model.slug(),
                model.interface()
            );
        }
    }

    /// Every Coolscan is one vendor's, and no two answer to the same product id
    #[test]
    fn the_usb_ids_are_one_vendor_and_all_distinct() {
        let ids: Vec<_> = Model::ALL.into_iter().filter_map(Model::usb_ids).collect();
        assert!(ids.iter().all(|(vendor, _)| *vendor == USB_VENDOR));
        let mut products: Vec<_> = ids.iter().map(|(_, product)| *product).collect();
        products.sort_unstable();
        products.dedup();
        assert_eq!(products.len(), ids.len(), "two models share a product id");
    }

    #[test]
    fn the_driven_models_are_the_three_with_drivers() {
        let driven: Vec<_> = Model::ALL.into_iter().filter(|m| m.is_driven()).collect();
        assert_eq!(driven, vec![Model::Ls9000, Model::Ls5000, Model::Ls50]);
    }
}
