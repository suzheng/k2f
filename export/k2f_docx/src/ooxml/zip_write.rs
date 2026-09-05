use crate::DocxError;
use std::collections::BTreeMap;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipWriter};

pub fn write_deterministic_zip(files: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>, DocxError> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let time = DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0)
            .map_err(|e| DocxError::Write(format!("zip timestamp: {e}")))?;
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .last_modified_time(time);
        for (name, bytes) in files {
            zip.start_file(name, options)
                .map_err(|e| DocxError::Write(format!("zip start {name}: {e}")))?;
            zip.write_all(bytes)
                .map_err(|e| DocxError::Write(format!("zip write {name}: {e}")))?;
        }
        zip.finish()
            .map_err(|e| DocxError::Write(format!("zip finish: {e}")))?;
    }
    Ok(cursor.into_inner())
}
