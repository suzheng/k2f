use std::collections::BTreeMap;

use k2f_package::write_zip;

use crate::document::OpenedDocument;
use crate::error::PaintError;
use crate::export::jpeg::png_to_jpeg;
use crate::OFFICIAL_PNG_SCALE;

fn page_name(idx: usize, ext: &str) -> String {
    format!("page-{}.{}", idx + 1, ext)
}

fn pages_zip(
    doc: &OpenedDocument,
    scale: f32,
    ext: &str,
    encode: fn(&[u8]) -> Result<Vec<u8>, PaintError>,
) -> Result<Vec<u8>, PaintError> {
    let n = doc.page_count();
    if n == 0 {
        return Err(PaintError::Unlocked);
    }
    if n == 1 {
        let png = doc.render_page(0, scale)?;
        return encode(&png);
    }
    let mut files = BTreeMap::new();
    for i in 0..n {
        let png = doc.render_page(i, scale)?;
        let bytes = encode(&png)?;
        files.insert(page_name(i, ext), bytes);
    }
    write_zip(&files).map_err(PaintError::from)
}

fn pass_png(png: &[u8]) -> Result<Vec<u8>, PaintError> {
    Ok(png.to_vec())
}

/// All lock pages as PNG. Single-page docs return one PNG; multi-page return a zip.
pub fn export_pages_png(doc: &OpenedDocument, scale: f32) -> Result<Vec<u8>, PaintError> {
    pages_zip(doc, scale, "png", pass_png)
}

/// All lock pages as JPEG. Single-page docs return one JPG; multi-page return a zip.
pub fn export_pages_jpeg(doc: &OpenedDocument, scale: f32) -> Result<Vec<u8>, PaintError> {
    pages_zip(doc, scale, "jpg", png_to_jpeg)
}

pub fn export_pages_png_official(doc: &OpenedDocument) -> Result<Vec<u8>, PaintError> {
    export_pages_png(doc, OFFICIAL_PNG_SCALE)
}

pub fn export_pages_jpeg_official(doc: &OpenedDocument) -> Result<Vec<u8>, PaintError> {
    export_pages_jpeg(doc, OFFICIAL_PNG_SCALE)
}
