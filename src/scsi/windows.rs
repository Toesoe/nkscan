//! SCSI passthrough on Windows, via SPTI/SPTD
//!
//! Not yet implemented. Windows issues SCSI commands through
//! `DeviceIoControl` with `IOCTL_SCSI_PASS_THROUGH_DIRECT`, passing a
//! `SCSI_PASS_THROUGH_DIRECT` (or `..._WITH_BUFFER`) struct that carries the
//! CDB, data buffer, direction, and sense buffer in a single call. This is
//! structurally close to the Linux `sg_io_hdr` used by [`super::linux`], so
//! the `Transport` contract should map onto it without much friction.

use super::{DataDirection, Error, Transport};

/// A SCSI device reachable through Windows' SPTI passthrough
pub struct SptiDevice {
    // TODO: HANDLE obtained from CreateFileW
}

impl SptiDevice {
    pub fn open(_path: &str) -> std::io::Result<Self> {
        todo!("open a HANDLE to the device via CreateFileW")
    }
}

impl Transport for SptiDevice {
    fn execute(
        &mut self,
        _cdb: &[u8],
        _direction: DataDirection,
        _data: &mut [u8],
        _sense: &mut [u8],
    ) -> Result<(), Error> {
        todo!("DeviceIoControl with IOCTL_SCSI_PASS_THROUGH_DIRECT")
    }
}
