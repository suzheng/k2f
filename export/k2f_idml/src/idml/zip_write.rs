use crate::coord::MIME;
use crate::IdmlError;
use std::collections::BTreeMap;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipWriter};

fn zip_time() -> Result<DateTime, IdmlError> {
    DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0)
        .map_err(|e| IdmlError::Write(format!("zip timestamp: {e}")))
}

fn options(method: CompressionMethod) -> Result<SimpleFileOptions, IdmlError> {
    Ok(SimpleFileOptions::default()
        .compression_method(method)
        .last_modified_time(zip_time()?))
}

pub fn write_idml_zip(files: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>, IdmlError> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let stored = options(CompressionMethod::Stored)?;
        let deflated = options(CompressionMethod::Deflated)?;
        zip.start_file("mimetype", stored)
            .map_err(|e| IdmlError::Write(format!("zip start mimetype: {e}")))?;
        zip.write_all(MIME.as_bytes())
            .map_err(|e| IdmlError::Write(format!("zip write mimetype: {e}")))?;
        for (name, bytes) in files {
            if name == "mimetype" {
                continue;
            }
            zip.start_file(name, deflated)
                .map_err(|e| IdmlError::Write(format!("zip start {name}: {e}")))?;
            zip.write_all(bytes)
                .map_err(|e| IdmlError::Write(format!("zip write {name}: {e}")))?;
        }
        zip.finish()
            .map_err(|e| IdmlError::Write(format!("zip finish: {e}")))?;
    }
    Ok(cursor.into_inner())
}

/// Deterministic Deflate zip of `name → bytes`. No mimetype-first rule (that is IDML-only).
pub fn write_files_zip(files: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>, IdmlError> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let deflated = options(CompressionMethod::Deflated)?;
        for (name, bytes) in files {
            zip.start_file(name, deflated)
                .map_err(|e| IdmlError::Write(format!("zip start {name}: {e}")))?;
            zip.write_all(bytes)
                .map_err(|e| IdmlError::Write(format!("zip write {name}: {e}")))?;
        }
        zip.finish()
            .map_err(|e| IdmlError::Write(format!("zip finish: {e}")))?;
    }
    Ok(cursor.into_inner())
}
