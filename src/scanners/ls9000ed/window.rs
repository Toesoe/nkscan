//! Vendor-specific bytes for the window payloads

use super::Multisample;

pub struct WindowParams {
    multisample:Multisample
}

impl WindowParams {
    fn pack_to_vendor_bytes(&self) -> Vec<u8> {
        todo!()
    }
}
