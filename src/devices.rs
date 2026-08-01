//! Finding the scanners attached to this machine
//!
//! Enumeration must not disturb anything: it sweeps devices that have nothing to do with us, so
//! it asks each who it is and no more. Building a driver would reserve the unit and write a mode
//! page on its way up, which is why that waits for [`Session`](crate::session::Session).

use crate::model::Interface;
use crate::scsi::{Transport, TransportExt, cdbs::Inquiry, usb::UsbTransport};
use nusb::MaybeFuture;
use std::io;
use std::path::{Path, PathBuf};
use tracing::debug;

/// The SCSI transport for this platform
///
/// macOS has an unimplemented stub whose signature differs, so SCSI is not offered there.
#[cfg(target_os = "linux")]
use crate::scsi::linux::SgDevice as ScsiDevice;
#[cfg(target_os = "windows")]
use crate::scsi::windows::ScsiScanDevice as ScsiDevice;

/// What this library reports as the maker of every scanner it drives
pub const VENDOR: &str = "Nikon";

/// A scanner this library has vocabulary for
///
/// Adding a model is a variant in [`model`](crate::model), its tables below, and a driver in
/// [`session`](crate::session).
pub use crate::model::Model;

/// Where a particular scanner is
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attach {
    Usb { vendor: u16, product: u16 },
    Scsi { path: PathBuf },
}

impl Attach {
    /// The half of a device id that says where the scanner is
    pub fn location(&self) -> String {
        match self {
            Attach::Usb { vendor, product } => format!("usb:{vendor:04x}:{product:04x}"),
            Attach::Scsi { path } => path.display().to_string(),
        }
    }

    fn parse(location: &str, model: Model) -> Option<Self> {
        match model.usb_ids() {
            Some((vendor, product)) => location
                .eq_ignore_ascii_case(&format!("usb:{vendor:04x}:{product:04x}"))
                .then_some(Attach::Usb { vendor, product }),
            None => Some(Attach::Scsi {
                path: PathBuf::from(location),
            }),
        }
    }

    /// Open a transport to this scanner, claiming it
    pub fn open(&self) -> io::Result<Box<dyn Transport + Send>> {
        match self {
            Attach::Usb { vendor, product } => Ok(Box::new(UsbTransport::open(*vendor, *product)?)),
            Attach::Scsi { path } => open_scsi(path),
        }
    }
}

/// A scanner the search turned up, not yet opened
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// `<model slug>@<where it is attached>`, as [`Session::open`](crate::session::Session::open)
    /// takes it
    ///
    /// Names the attachment point, not the unit. These scanners report no serial number and an
    /// sg node is assigned in probe order, so an id holds only for as long as nothing is
    /// replugged. Re-enumerate rather than storing one.
    pub id: String,
    pub model: Model,
    pub attach: Attach,
}

impl DeviceInfo {
    fn new(model: Model, attach: Attach) -> Self {
        Self {
            id: format!("{}@{}", model.slug(), attach.location()),
            model,
            attach,
        }
    }

    /// The model and location an id names, or `None` if it is not an id at all
    ///
    /// Split at the first `@` so a Windows device path, which has no `@` of its own, survives.
    pub fn parse_id(id: &str) -> Option<(Model, Attach)> {
        let (slug, location) = id.split_once('@')?;
        let model = Model::ALL.into_iter().find(|m| m.slug() == slug)?;
        Some((model, Attach::parse(location, model)?))
    }
}

/// Every scanner attached that this library recognizes
///
/// Recognizing one is not the same as being able to drive it: a model without a driver is listed
/// so a caller learns their unit was seen, and [`Session::open`](crate::session::Session::open)
/// is where it is refused. [`Model::is_driven`] tells the two apart without opening anything.
pub fn list() -> Vec<DeviceInfo> {
    let mut found = Vec::new();

    // Presence only, since claiming the interface is what opening would do
    let usb: Vec<(u16, u16)> = nusb::list_devices()
        .wait()
        .map(|devices| devices.map(|d| (d.vendor_id(), d.product_id())).collect())
        .unwrap_or_default();

    for model in Model::ALL {
        if let Some((vendor, product)) = model.usb_ids() {
            found.extend(
                usb.iter()
                    .filter(|ids| **ids == (vendor, product))
                    .map(|_| DeviceInfo::new(model, Attach::Usb { vendor, product })),
            );
        }
    }

    // One sweep, since one INQUIRY answers for every SCSI model at once
    for path in scsi_paths() {
        let Some(product) = probe_scsi(&path) else {
            continue;
        };
        match model_named(&product) {
            Some(model) => found.push(DeviceInfo::new(model, Attach::Scsi { path })),
            None => debug!(%product, path = %path.display(), "A Nikon we do not drive"),
        }
    }
    found
}

/// The model an INQUIRY product string names, if it is one this library knows
///
/// Only the FireWire models, since those are what a SCSI node can be. A USB model is found by
/// its ids instead, and matching one here would name a scanner that is not on this bus.
fn model_named(product: &str) -> Option<Model> {
    let product = product.to_ascii_lowercase();
    Model::ALL
        .into_iter()
        .filter(|model| model.interface() == Interface::Firewire400)
        .find(|model| product.contains(&model.name().to_ascii_lowercase()))
}

/// Device paths worth asking who they are
#[cfg(target_os = "linux")]
fn scsi_paths() -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir("/dev") else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let name = path.file_name()?.to_str()?.to_owned();
            let index = name.strip_prefix("sg")?;
            (!index.is_empty() && index.chars().all(|c| c.is_ascii_digit())).then_some(path)
        })
        .collect();
    paths.sort();
    paths
}

#[cfg(target_os = "windows")]
fn scsi_paths() -> Vec<PathBuf> {
    (0..10)
        .map(|n| PathBuf::from(format!(r"\\.\Scanner{n}")))
        .collect()
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn scsi_paths() -> Vec<PathBuf> {
    Vec::new()
}

/// How the device at `path` introduces itself, if it is a Nikon at all
///
/// An INQUIRY and nothing else. This sweeps devices that have nothing to do with us, so it must
/// not change any of them: notably it does not build a driver, which would reserve the unit and
/// write a mode page on its way up. Anything that fails to open or answers as something else is
/// not a match rather than an error, since being refused by an unrelated device is normal.
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub fn probe_scsi(path: &Path) -> Option<String> {
    let mut device = ScsiDevice::open(path).ok()?;
    let identity = device.send(&Inquiry::new()).ok()?;
    identity
        .vendor
        .trim()
        .eq_ignore_ascii_case("nikon")
        .then(|| identity.product.trim().to_owned())
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn probe_scsi(_path: &Path) -> Option<String> {
    None
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn open_scsi(path: &Path) -> io::Result<Box<dyn Transport + Send>> {
    Ok(Box::new(ScsiDevice::open(path)?))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn open_scsi(_path: &Path) -> io::Result<Box<dyn Transport + Send>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "SCSI is not implemented on this platform, so only a USB scanner will work here",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An id has to survive a round trip, including a Windows path, which is why the split is at
    /// the first `@` rather than the last
    #[test]
    fn ids_round_trip() {
        let cases = [
            (
                Model::Ls9000,
                Attach::Scsi {
                    path: PathBuf::from("/dev/sg3"),
                },
                "ls9000@/dev/sg3",
            ),
            (
                Model::Ls9000,
                Attach::Scsi {
                    path: PathBuf::from(r"\\.\Scanner0"),
                },
                r"ls9000@\\.\Scanner0",
            ),
            (
                Model::Ls50,
                Attach::Usb {
                    vendor: 0x04B0,
                    product: 0x4001,
                },
                "ls50@usb:04b0:4001",
            ),
            (
                Model::Ls5000,
                Attach::Usb {
                    vendor: 0x04B0,
                    product: 0x4002,
                },
                "ls5000@usb:04b0:4002",
            ),
        ];
        for (model, attach, id) in cases {
            let info = DeviceInfo::new(model, attach.clone());
            assert_eq!(info.id, id);
            assert_eq!(DeviceInfo::parse_id(id), Some((model, attach)), "{id}");
        }
    }

    #[test]
    fn a_string_that_is_not_an_id_is_refused() {
        for id in [
            "",
            "ls9000",
            "/dev/sg3",
            "nope@/dev/sg3",
            "ls50@usb:0000:0000",
        ] {
            assert_eq!(DeviceInfo::parse_id(id), None, "{id}");
        }
    }

    /// The product string a unit answers with is matched loosely, since it carries padding and
    /// the revision alongside the model
    #[test]
    fn the_inquiry_product_string_names_a_model() {
        assert_eq!(model_named("LS-9000 ED"), Some(Model::Ls9000));
        assert_eq!(model_named("ls-9000 ed   "), Some(Model::Ls9000));
        // Recognized without being driveable: enumeration names it, `Session::open` refuses it
        assert_eq!(model_named("LS-8000 ED"), Some(Model::Ls8000));
        assert!(!Model::Ls8000.is_driven());
        // The USB models are never found this way, so they never match here
        assert_eq!(model_named("LS-50 ED"), None);
        assert_eq!(model_named("LS-40 ED"), None);
    }

    /// The two four-digit names must not shadow the two-digit ones, since the match is a substring
    #[test]
    fn a_four_thousand_is_not_a_forty() {
        assert_eq!(model_named("LS-4000 ED"), Some(Model::Ls4000));
        assert_ne!(model_named("LS-4000 ED"), Some(Model::Ls40));
    }

    /// The floor the LS-9000 reports across its sensor bar keeps 333 DPI off its ladder, even
    /// though 12 divides the sensor evenly
    #[test]
    fn the_ls9000_ladder_stops_at_the_reported_floor() {
        let dpi = crate::capability::Capabilities::of(Model::Ls9000)
            .resolution
            .ladder;
        assert_eq!(dpi, [4000, 2000, 1333, 666]);
        assert!(!dpi.contains(&333));
    }
}
