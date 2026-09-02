use crate::classify::classify_opened;
use crate::ooxml;
use crate::PptxError;
use k2f_paint::OpenedDocument;
use std::io::Cursor;
use zip::ZipArchive;

pub fn export_bytes(package_bytes: &[u8]) -> Result<Vec<u8>, PptxError> {
    if looks_like_pptx(package_bytes) {
        return Err(PptxError::NotASource);
    }
    export_opened(&OpenedDocument::open(package_bytes)?)
}

pub fn export_opened(doc: &OpenedDocument) -> Result<Vec<u8>, PptxError> {
    if doc.fonts().is_empty() {
        return Err(PptxError::NoFont);
    }
    let deck = classify_opened(doc)?;
    let files = ooxml::build_package(&deck);
    ooxml::write_deterministic_zip(&files)
}

fn looks_like_pptx(bytes: &[u8]) -> bool {
    let Ok(zip) = ZipArchive::new(Cursor::new(bytes)) else {
        return false;
    };
    let has_presentation = zip
        .file_names()
        .any(|n| n == "ppt/presentation.xml" || n == "ppt\\presentation.xml");
    has_presentation
}
