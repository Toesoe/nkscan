//! SCSI-over-USB transport for USB-attached scanners, via `nusb`
//!
//! Not yet implemented. USB-attached Nikon scanners appear to carry SCSI
//! CDBs directly over bulk endpoints rather than going through any platform
//! SCSI passthrough layer, so this backend talks to the device purely
//! through USB bulk transfers.
//!
//! `nusb`'s API is async; bridging that into this crate's synchronous
//! [`Transport`] (block on each transfer, or make `Transport` itself async)
//! is still an open question and left for when this gets implemented.

use super::{DataDirection, Error, Transport};

/// A SCSI device reachable over USB bulk endpoints
pub struct UsbTransport {
    // TODO: nusb::Interface + bulk endpoint addresses
}

impl UsbTransport {
    pub fn open(_vendor_id: u16, _product_id: u16) -> std::io::Result<Self> {
        todo!("find the device via nusb::list_devices(), open it, claim the bulk interface")
    }
}

impl Transport for UsbTransport {
    fn execute(
        &mut self,
        _cdb: &[u8],
        _direction: DataDirection,
        _data: &mut [u8],
        _sense: &mut [u8],
    ) -> Result<(), Error> {
        todo!("write the CDB + data out, read status/sense back over bulk endpoints")
    }
}
