#![allow(dead_code)]

use k2f_paint::OpenedDocument;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn invoice_path() -> PathBuf {
    repo_root().join("examples/published/invoice.K2F")
}

pub fn invoice_bytes() -> Vec<u8> {
    std::fs::read(invoice_path()).unwrap()
}

pub fn invoice() -> OpenedDocument {
    OpenedDocument::open(&invoice_bytes()).unwrap()
}

pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub fn fixture_path(name: &str) -> PathBuf {
    fixtures_dir().join(name)
}

pub fn open_fixture(name: &str) -> Option<OpenedDocument> {
    let path = fixture_path(name);
    if !path.is_file() {
        return None;
    }
    Some(OpenedDocument::open(&std::fs::read(path).unwrap()).unwrap())
}

pub fn contract_path() -> PathBuf {
    repo_root().join("examples/published/contract.K2F")
}

pub fn contract() -> OpenedDocument {
    OpenedDocument::open(&std::fs::read(contract_path()).unwrap()).unwrap()
}

pub fn unzip_names(idml: &[u8]) -> Vec<String> {
    let mut zip = ZipArchive::new(Cursor::new(idml)).expect("idml zip");
    let mut names: Vec<String> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().replace('\\', "/"))
        .collect();
    names.sort();
    names
}

pub fn xml_in(idml: &[u8], name: &str) -> String {
    String::from_utf8(bytes_in(idml, name)).unwrap_or_else(|_| panic!("{name} not utf-8"))
}

pub fn bytes_in(idml: &[u8], name: &str) -> Vec<u8> {
    let mut zip = ZipArchive::new(Cursor::new(idml)).expect("idml zip");
    let mut file = zip
        .by_name(name)
        .unwrap_or_else(|_| panic!("missing {name}"));
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}

pub fn zip_index0_name_and_method(idml: &[u8]) -> (String, CompressionMethod) {
    let mut zip = ZipArchive::new(Cursor::new(idml)).expect("idml zip");
    let file = zip.by_index(0).expect("zip index 0");
    (file.name().replace('\\', "/"), file.compression())
}

/// Copy a zip, optionally dropping or replacing named entries. Directories skipped.
pub fn rewrite_zip(src: &[u8], mut map: impl FnMut(&str, Vec<u8>) -> Option<Vec<u8>>) -> Vec<u8> {
    let mut input = ZipArchive::new(Cursor::new(src)).expect("src zip");
    let mut entries = Vec::new();
    for i in 0..input.len() {
        let mut f = input.by_index(i).unwrap();
        let name = f.name().replace('\\', "/");
        if name.ends_with('/') {
            continue;
        }
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        if let Some(next) = map(&name, buf) {
            entries.push((name, next));
        }
    }
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (name, bytes) in &entries {
            zip.start_file(name, opts).unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap();
    }
    cursor.into_inner()
}
