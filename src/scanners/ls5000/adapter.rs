//! Which adapter this model has loaded
//!
//! Working out *which* adapter a page list names is shared, in
//! [`nikon::adapter`](crate::scanners::nikon::adapter). What is specific to this model is which
//! page carries the adapter's own name: 0x01 here, where the LS-50 uses 0x46.

use super::Ls5000;
use crate::{
    adapter::Adapter,
    scanners::{FilmHolder, Scanner, nikon::adapter::SupportedPages, nikon::page_name},
    scsi::{self, Transport, TransportExt},
};

/// Carries the adapter's name on this model
const ADAPTER_NAME_PAGE: u8 = 0x01;

impl<T> FilmHolder for Ls5000<T>
where
    T: Transport,
{
    fn adapter(&mut self) -> Result<Adapter, scsi::Error> {
        let pages: SupportedPages = self.transport.vpd()?;
        Ok(pages.adapter())
    }
}

impl<T> Ls5000<T>
where
    T: Transport,
{
    /// What the adapter calls itself, `None` without page 0x01
    ///
    /// The roll adapter answers `36Strip`, a positive identification rather than the reading
    /// [`adapter`](crate::scanners::FilmHolder::adapter) takes from the page list.
    pub fn adapter_name(&mut self) -> Option<String> {
        page_name(&self.vpd_page(ADAPTER_NAME_PAGE).ok()?)
    }
}

#[cfg(test)]
mod tests {
    use crate::scanners::nikon::page_name;

    /// Page 0x01 as the roll adapter answers it, name and all
    #[test]
    fn reads_the_adapter_name_off_the_captured_page() {
        let captured = [0x08, 0x33, 0x36, 0x53, 0x74, 0x72, 0x69, 0x70, 0x00];
        assert_eq!(page_name(&captured).as_deref(), Some("36Strip"));
    }
}
