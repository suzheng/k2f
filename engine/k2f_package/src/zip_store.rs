use crate::error::PackageError;
use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

fn zip_options() -> Result<SimpleFileOptions, PackageError> {
    let time = DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0)
        .map_err(|e| PackageError::Other(format!("zip timestamp: {e}")))?;
    Ok(SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(time)
        .unix_permissions(0o644))
}

pub fn write_zip(files: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>, PackageError> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let options = zip_options()?;
        for (name, bytes) in files {
            zip.start_file(name, options)
                .map_err(|e| PackageError::Other(format!("zip start {name}: {e}")))?;
            zip.write_all(bytes)?;
        }
        zip.finish()
            .map_err(|e| PackageError::Other(format!("zip finish: {e}")))?;
    }
    Ok(cursor.into_inner())
}

pub fn read_zip(bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, PackageError> {
    let mut zip = ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| PackageError::Other(format!("zip open: {e}")))?;
    let mut files = BTreeMap::new();
    for i in 0..zip.len() {
        let mut file = zip
            .by_index(i)
            .map_err(|e| PackageError::Other(format!("zip entry {i}: {e}")))?;
        if file.is_dir() {
            continue;
        }
        let name = file.name().replace('\\', "/");
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        files.insert(name, buf);
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_zip_accepts_deflated_entries() {
        let mut files = BTreeMap::new();
        files.insert("hello.txt".to_string(), b"lock-executor".to_vec());
        let stored = write_zip(&files).unwrap();

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            let time = DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).unwrap();
            let options = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .last_modified_time(time);
            zip.start_file("hello.txt", options).unwrap();
            zip.write_all(b"lock-executor").unwrap();
            zip.finish().unwrap();
        }
        let deflated = cursor.into_inner();
        assert_ne!(deflated, stored);
        assert_eq!(read_zip(&deflated).unwrap(), files);
        assert_eq!(read_zip(&stored).unwrap(), files);
    }
}
