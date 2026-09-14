use crate::coord::MIME;
use crate::IdmlError;
use std::collections::BTreeMap;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipWriter};

pub fn write_idml_zip(files: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>, IdmlError> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let time = DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0)
            .map_err(|e| IdmlError::Write(format!("zip timestamp: {e}")))?;
        let stored = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .last_modified_time(time);
        let deflated = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .last_modified_time(time);
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
