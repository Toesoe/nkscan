//! Which holder this model has loaded
//!
//! Parsing page 0xC8 is shared, in [`nikon::adapter`](crate::scanners::nikon::adapter). This
//! model is the reason that page is read at all: the medium format bodies report a holder class
//! rather than advertising an adapter-specific page the way the 35 mm bodies do.

use super::Ls9000;
use crate::{
    adapter::Adapter,
    scanners::{FilmHolder, nikon::adapter::HolderReading},
    scsi::{self, Transport, TransportExt},
};

impl<T> FilmHolder for Ls9000<T>
where
    T: Transport,
{
    fn adapter(&mut self) -> Result<Adapter, scsi::Error> {
        Ok(self.holder_reading()?.adapter())
    }
}

impl<T> Ls9000<T>
where
    T: Transport,
{
    /// Everything page 0xC8 says about the loaded holder
    ///
    /// [`adapter`](crate::scanners::FilmHolder::adapter) narrows this to the shared vocabulary
    /// and throws the rest away. The rest is worth having: the class alone does not name a part,
    /// but the class together with the aperture count and the holder width very nearly does, and
    /// this is where a caller can see all three.
    pub fn holder_reading(&mut self) -> Result<HolderReading, scsi::Error> {
        self.transport.vpd()
    }
}
