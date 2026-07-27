//! Nikon vendor TRIGGER(6)
//!
//! Shared, so this is only a re-export. Commits whatever a preceding
//! [`VendorWrite`](super::VendorWrite) staged; opcode only, no data phase.

pub use crate::scanners::nikon::cdbs::VendorTrigger;
