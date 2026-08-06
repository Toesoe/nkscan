//! High-level scanning operations
//!
//! This is where we work out what a scan should do, from what the unit says it
//! can do, and then order the session calls to do it. Checks that a single
//! argument is legal stay down in `session`; picking between two mechanisms
//! happens here, once, before anything moves.

pub mod expose;
pub mod focus;
pub mod framing;
pub mod meter;
pub mod preamble;
pub mod thumbnail;

pub use expose::{Exposure, expose};
pub use focus::Focus;
pub use framing::Framing;
pub use meter::Metering;
pub use thumbnail::Thumbnail;
