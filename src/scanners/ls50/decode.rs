//! Decoding the LS-50's raw scan stream into an image
//!
//! The stream format itself is shared with every other planar Coolscan and lives in
//! [`nikon::decode`](crate::scanners::nikon::decode); what belongs here is only how this
//! model's scan settings turn into the geometry that decoder needs.

use super::geometry::ScanSettings;

pub use crate::scanners::nikon::decode::{DecodeError, FrameDecoder, PlanarLines};

/// A decoder for the pass `settings` describes
pub fn frame_decoder(settings: &ScanSettings) -> FrameDecoder {
    let (width, height) = settings.output_dims();
    crate::scanners::nikon::decode::planar_decoder(
        width as usize,
        height as usize,
        settings.n_colors(),
        settings.bytes_per_line(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::StreamDecoder;

    #[test]
    fn expected_bytes_follows_the_scan_geometry() {
        let capabilities = super::super::capabilities::fixture::capabilities();
        let settings = ScanSettings {
            dpi: super::super::geometry::Dpi::_1000,
            ir: false,
            samples: 1,
            window: crate::scanners::ScanArea::frame(0, capabilities),
            capabilities,
        };
        let decoder = frame_decoder(&settings);
        assert_eq!(decoder.expected_bytes(), settings.expected_bytes());
    }
}
