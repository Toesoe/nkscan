//! Coolscan IV ED — recognized, not driven
//!
//! A placeholder so that the model has somewhere to live and so what is known about it is
//! written down rather than rediscovered. The USB ids below are the only thing here that is
//! wired up: they let enumeration name an attached unit, which
//! [`Session::open`](crate::session::Session::open) then refuses.
//!
//! # What it is believed to need
//!
//! `Protocol::Ls50`, since the LS-50 is its direct successor and the
//! closest relative. Unverified.
//!
//! # What differs from every other model
//!
//! The LS-40 is the outlier of the six, and each difference is a place where a constant that
//! looks universal is not:
//!
//! - **2900 DPI optical**, not 4000. Every other Coolscan here is a 4000 DPI sensor, and the
//!   library currently converts dots to millimeters against a hardcoded 4000 and derives its
//!   resolution ladders by dividing 4000. Both are wrong for this body by 38%, which is the one
//!   real code change LS-40 support needs beyond a driver.
//! - **12-bit samples**, the narrowest of the six.
//! - **No multi-sample.** The LS-50 shares this; every other model has 1/2/4/8/16.
//! - **USB 1.1.** A full-resolution 35 mm frame over a 12 Mbit link is slow enough that the
//!   read timeouts written for USB 2.0 may not be generous enough.
//! - **No Kodachrome infrared profile.**
//!
//! # Adapters
//!
//! The same five objects as the rest of the 35 mm range, under Nikon's older part numbers:
//! SA-20, MA-20, SF-200 and the IX240 adapter, plus the SA-30, which kept one number everywhere.
//! [`Adapter::part_number`](crate::adapter::Adapter::part_number) does that translation.

/// For [`UsbTransport::open`](crate::scsi::usb::UsbTransport::open)
pub const VENDOR_ID: u16 = 0x04B0;
/// Below the LS-50's 0x4001 and the LS-5000's 0x4002. From a live descriptor,
/// `USB\VID_04b0&PID_4000`.
pub const PRODUCT_ID: u16 = 0x4000;
