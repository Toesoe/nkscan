//! Writing scans out, tagged with the transfer function they were captured with
//!
//! The scanners return linear ADC counts. Nothing in a bare TIFF says so, and a viewer handed
//! untagged 16-bit RGB assumes something sRGB shaped, which renders linear midtones far darker
//! than they are. Every file written here carries a profile saying the data is linear, and the
//! samples themselves are left exactly as they came off the scanner.

use crate::scanners::ls9000ed::decode::{Luma16, Rgb16};
use image::{
    ExtendedColorType, ImageEncoder, ImageError, ImageFormat, ImageResult,
    codecs::tiff::TiffEncoder, error::EncodingError,
};
use moxcms::{
    CicpColorPrimaries, CicpProfile, ColorProfile, LocalizableString, MatrixCoefficients,
    ProfileText, ToneReprCurve, TransferCharacteristics,
};
use std::io::{Seek, Write};

/// An ICC profile describing linear light on sRGB primaries
///
/// The transfer curve is the part that is true and the part that matters: gamma 1.0, so a
/// viewer stops applying a decode the data never had.
///
/// The primaries are a placeholder. The real ones are three narrow LED bands and nothing has
/// measured them, which takes a 3x3 off an IT8 target. sRGB primaries are the conventional
/// stand-in and no worse than the nothing an untagged file says, but they are not a
/// characterization of the scanner.
pub fn linear_rgb_profile() -> Result<Vec<u8>, moxcms::CmsError> {
    let mut profile = ColorProfile::new_srgb();

    // A curve with no points is the ICC identity, which is gamma 1.0
    let linear = ToneReprCurve::Lut(Vec::new());
    profile.red_trc = Some(linear.clone());
    profile.green_trc = Some(linear.clone());
    profile.blue_trc = Some(linear);

    // Left as sRGB this would contradict the curves above, and a reader trusting it would undo
    // the whole point of the profile
    profile.cicp = Some(CicpProfile {
        color_primaries: CicpColorPrimaries::Bt709,
        transfer_characteristics: TransferCharacteristics::Linear,
        matrix_coefficients: MatrixCoefficients::Bt709,
        full_range: false,
    });

    profile.description = Some(ProfileText::Localizable(vec![LocalizableString::new(
        "en".to_string(),
        "US".to_string(),
        "nkscan linear RGB".to_string(),
    )]));

    profile.encode()
}

/// Samples as the encoder wants them
///
/// 16-bit TIFF is written in the host's byte order and tagged accordingly, so this hands over
/// native-endian bytes rather than converting.
fn as_bytes(samples: &[u16]) -> &[u8] {
    // SAFETY: u16 has no padding and no invalid bit patterns, so any u16 slice is a valid byte
    // slice of twice the length, borrowed for the same lifetime.
    unsafe {
        std::slice::from_raw_parts(
            samples.as_ptr().cast::<u8>(),
            std::mem::size_of_val(samples),
        )
    }
}

fn encoding_error<E>(e: E) -> ImageError
where
    E: std::error::Error + Send + Sync + 'static,
{
    ImageError::Encoding(EncodingError::new(ImageFormat::Tiff.into(), e))
}

/// Write a decoded frame as a 16-bit TIFF carrying [`linear_rgb_profile`]
pub fn write_rgb16_tiff<W>(writer: W, image: &Rgb16) -> ImageResult<()>
where
    W: Write + Seek,
{
    let profile = linear_rgb_profile().map_err(encoding_error)?;
    let mut encoder = TiffEncoder::new(writer);
    encoder.set_icc_profile(profile).map_err(encoding_error)?;
    encoder.write_image(
        as_bytes(image.as_raw()),
        image.width(),
        image.height(),
        ExtendedColorType::Rgb16,
    )
}

/// Write an infrared mask as a 16-bit grayscale TIFF
///
/// No profile. This is a transmission measurement for finding dust rather than something to
/// render, and an RGB profile would be a lie on a single channel.
pub fn write_luma16_tiff<W>(writer: W, image: &Luma16) -> ImageResult<()>
where
    W: Write + Seek,
{
    TiffEncoder::new(writer).write_image(
        as_bytes(image.as_raw()),
        image.width(),
        image.height(),
        ExtendedColorType::L16,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Whatever else changes, the curves have to stay linear, since a viewer applying an sRGB
    /// decode to linear data is the thing this exists to prevent
    #[test]
    fn the_profile_round_trips_as_linear() {
        let encoded = linear_rgb_profile().expect("profile encodes");
        let parsed = ColorProfile::new_from_slice(&encoded).expect("profile parses");

        for curve in [&parsed.red_trc, &parsed.green_trc, &parsed.blue_trc] {
            match curve.as_ref().expect("every channel has a curve") {
                // No points is the identity
                ToneReprCurve::Lut(points) => {
                    assert!(points.is_empty(), "{points:?} is not linear")
                }
                // A single u8Fixed8 point is a plain gamma, and 1.0 encodes as 256
                ToneReprCurve::Parametric(_) => panic!("expected a curve, got a parametric"),
            }
        }
    }

    /// Encoding a valid profile is worth nothing if it does not reach the file
    #[test]
    fn a_written_tiff_carries_the_profile() {
        use image::ImageDecoder;

        let image = Rgb16::from_pixel(4, 4, image::Rgb([1000, 2000, 3000]));
        let mut file = std::io::Cursor::new(Vec::new());
        write_rgb16_tiff(&mut file, &image).expect("writes");

        file.set_position(0);
        let mut decoder = image::codecs::tiff::TiffDecoder::new(file).expect("reads back");
        let embedded = decoder
            .icc_profile()
            .expect("profile is readable")
            .expect("a profile is present");

        assert_eq!(embedded, linear_rgb_profile().unwrap());
    }

    /// The CICP block travels alongside the curves and has to agree with them
    #[test]
    fn the_profile_does_not_claim_an_srgb_transfer() {
        let encoded = linear_rgb_profile().expect("profile encodes");
        let parsed = ColorProfile::new_from_slice(&encoded).expect("profile parses");
        if let Some(cicp) = parsed.cicp {
            assert_eq!(
                cicp.transfer_characteristics,
                TransferCharacteristics::Linear
            );
        }
    }
}
