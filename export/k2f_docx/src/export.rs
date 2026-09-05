use crate::classify::classify_opened;
use crate::ooxml;
use crate::DocxError;
use k2f_paint::OpenedDocument;
use std::io::Cursor;
use zip::ZipArchive;

pub fn export_bytes(package_bytes: &[u8]) -> Result<Vec<u8>, DocxError> {
    if looks_like_docx(package_bytes) {
        return Err(DocxError::NotASource);
    }
    export_opened(&OpenedDocument::open(package_bytes)?)
}

pub fn export_opened(doc: &OpenedDocument) -> Result<Vec<u8>, DocxError> {
    if doc.fonts().is_empty() {
        return Err(DocxError::NoFont);
    }
    let ir = classify_opened(doc)?;
    let files = ooxml::build_package(&ir);
    ooxml::write_deterministic_zip(&files)
}

fn looks_like_docx(bytes: &[u8]) -> bool {
    let Ok(zip) = ZipArchive::new(Cursor::new(bytes)) else {
        return false;
    };
    let has_document = zip
        .file_names()
        .any(|n| n == "word/document.xml" || n == "word\\document.xml");
    has_document
}
