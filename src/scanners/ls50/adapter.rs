//! Which adapter this model has loaded
//!
//! Working out *which* adapter a page list names is shared, in
//! [`nikon::adapter`](crate::scanners::nikon::adapter). What is specific to this model is which
//! page carries the adapter's own name.

use super::Ls50;
use crate::{
    adapter::Adapter,
    scanners::{FilmHolder, Scanner, nikon::adapter::SupportedPages, nikon::page_name},
    scsi::{self, Transport, TransportExt},
};

/// Carries the adapter's name on the adapters that have this page
const ADAPTER_NAME_PAGE: u8 = 0x46;

impl<T> FilmHolder for Ls50<T>
where
    T: Transport,
{
    fn adapter(&mut self) -> Result<Adapter, scsi::Error> {
        let pages: SupportedPages = self.transport.vpd()?;
        Ok(pages.adapter())
    }
}

impl<T> Ls50<T>
where
    T: Transport,
{
    /// What the adapter calls itself, `None` without page 0x46
    ///
    /// The strip adapter answers `36SA_OBJECT`: a positive identification, unlike the reading
    /// [`adapter`](crate::scanners::FilmHolder::adapter) takes from the page list.
    pub fn adapter_name(&mut self) -> Option<String> {
        page_name(&self.vpd_page(ADAPTER_NAME_PAGE).ok()?)
    }
}

#[cfg(test)]
mod tests {
    use crate::scanners::nikon::page_name;

    /// Page 0x46 exactly as the strip adapter answers it
    #[test]
    fn reads_the_adapter_name_off_the_captured_page() {
        let captured = [
            0x0C, 0x33, 0x36, 0x53, 0x41, 0x5F, 0x4F, 0x42, 0x4A, 0x45, 0x43, 0x54, 0x00,
        ];
        assert_eq!(page_name(&captured).as_deref(), Some("36SA_OBJECT"));
    }

    /// Pages 0x60 and 0x61 carry parameter names in the same shape
    #[test]
    fn reads_a_parameter_name_in_the_same_shape() {
        let exp_time = [0x09, 0x45, 0x58, 0x50, 0x5F, 0x54, 0x49, 0x4D, 0x45, 0x00];
        assert_eq!(page_name(&exp_time).as_deref(), Some("EXP_TIME"));
    }

    #[test]
    fn a_page_with_no_name_in_it_reads_none() {
        assert_eq!(page_name(&[]), None);
        assert_eq!(page_name(&[0x00]), None);
        // A count reaching past what arrived
        assert_eq!(page_name(&[0x0C, 0x33, 0x36]), None);
    }
}
