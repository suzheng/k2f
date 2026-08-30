use image::codecs::jpeg::JpegEncoder;
use image::ImageReader;
use std::io::Cursor;

use crate::error::PaintError;

const JPEG_QUALITY: u8 = 90;

/// Encode a PNG page raster as JPEG.
pub fn png_to_jpeg(png: &[u8]) -> Result<Vec<u8>, PaintError> {
    let reader = ImageReader::new(Cursor::new(png))
        .with_guessed_format()
        .map_err(|e| PaintError::Image(e.to_string()))?;
    let img = reader
        .decode()
        .map_err(|e| PaintError::Image(e.to_string()))?;
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    let mut out = Vec::new();
    let mut enc = JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY);
    enc.encode(rgb.as_raw(), w, h, image::ExtendedColorType::Rgb8)
        .map_err(|e| PaintError::Image(e.to_string()))?;
    Ok(out)
}
