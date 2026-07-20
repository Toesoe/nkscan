//! SCSI passthrough on macOS, via IOKit's SCSI Architecture Model
//!
//! Not yet implemented. Unlike Linux's `SG_IO` (a single blocking ioctl) or
//! Windows' SPTI (a single `DeviceIoControl` call), macOS SCSI passthrough
//! goes through IOKit's `SCSITaskDeviceInterface`, a COM-style plugin
//! interface obtained via `IOCreatePlugInInterfaceForService`, where a task
//! is created, configured with the CDB/data/sense/timeout, and submitted
//! asynchronously. Bridging that into this crate's synchronous [`Transport`]
//! will need more scaffolding than the ioctl-style backends.

use super::{DataDirection, Error, Transport};

/// A SCSI device reachable through IOKit's SCSI Architecture Model.
pub struct ScsiTaskDevice {
    // TODO: IOKit plugin interface / SCSITaskDeviceInterface
}

impl ScsiTaskDevice {
    pub fn open(_bsd_path: &str) -> std::io::Result<Self> {
        todo!("look up the IOKit service and obtain a SCSITaskDeviceInterface")
    }
}

impl Transport for ScsiTaskDevice {
    fn execute(
        &mut self,
        _cdb: &[u8],
        _direction: DataDirection,
        _data: &mut [u8],
        _sense: &mut [u8],
    ) -> Result<(), Error> {
        todo!("build and submit an SCSITask, block for completion")
    }
}
