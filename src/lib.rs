pub mod adapter;
pub mod capability;
pub mod decode;
pub mod devices;
pub mod model;
pub mod output;
pub mod scanners;
pub mod scsi;
pub mod session;

/// The Python extension module, built only for the wheel
#[cfg(feature = "python")]
mod python;

/// Where the `stub_gen` binary reads the bindings from, so the module itself stays private
#[cfg(feature = "python")]
pub use python::stub_info as python_stub_info;
