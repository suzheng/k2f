use crate::classify;
use crate::idml;
use crate::IdmlError;
use k2f_paint::OpenedDocument;
use std::io::{Cursor, Read};
use zip::ZipArchive;

pub fn export_bytes(package_bytes: &[u8]) -> Result<Vec<u8>, IdmlError> {
    if looks_like_idml(package_bytes) {
        return Err(IdmlError::NotASource);
    }
    export_opened(&OpenedDocument::open(package_bytes)?)
}

pub fn export_opened(doc: &OpenedDocument) -> Result<Vec<u8>, IdmlError> {
    let lock = doc.lock().ok_or(IdmlError::Unlocked)?;
    if lock.has_unknown_paint_ops() {
        return Err(IdmlError::UnknownOp);
    }
    if doc.fonts().is_empty() {
        return Err(IdmlError::NoFont);
    }
    let ir = classify::classify_opened(doc)?;
    let files = idml::build_package(doc, lock, &ir)?;
    idml::write_idml_zip(&files)
}

fn looks_like_idml(bytes: &[u8]) -> bool {
    let Ok(mut zip) = ZipArchive::new(Cursor::new(bytes)) else {
        return false;
    };
    let mut has_designmap = false;
    let mut mime_idx = None;
    for i in 0..zip.len() {
        let Ok(file) = zip.by_index(i) else {
            continue;
        };
        let name = file.name().replace('\\', "/");
        if name == "designmap.xml" {
            has_designmap = true;
        }
        if name == "mimetype" {
            mime_idx = Some(i);
        }
    }
    if !has_designmap {
        return false;
    }
    let Some(idx) = mime_idx else {
        return false;
    };
    let Ok(mut file) = zip.by_index(idx) else {
        return false;
    };
    let mut buf = Vec::new();
    if file.read_to_end(&mut buf).is_err() {
        return false;
    }
    std::str::from_utf8(&buf)
        .map(|s| s.trim() == crate::MIME)
        .unwrap_or(false)
}
