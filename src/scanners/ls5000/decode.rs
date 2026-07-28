//! Decoding the LS-5000's raw scan stream into an image
//!
//! The stream format is shared and lives in [`nikon::decode`](crate::scanners::nikon::decode).
//! This model reads it in 512-aligned bulk chunks that straddle lines, which the block decoder
//! already handles through its staging buffer.

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
    use crate::scanners::ls5000::geometry::{Dpi, Samples, whole_frame};

    fn settings(ir: bool) -> ScanSettings {
        let capabilities = super::super::capabilities::fixture::capabilities();
        ScanSettings {
            resolution: Dpi::_1000.to_dpi(),
            ir,
            samples: Samples::default(),
            window: whole_frame(0, capabilities),
            capabilities,
        }
    }

    #[test]
    fn expected_bytes_follows_the_scan_geometry() {
        for ir in [false, true] {
            let settings = settings(ir);
            assert_eq!(
                frame_decoder(&settings).expected_bytes(),
                settings.expected_bytes()
            );
        }
    }
}
