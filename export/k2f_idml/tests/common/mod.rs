#![allow(dead_code)]

use k2f_paint::OpenedDocument;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use zip::{CompressionMethod, ZipArchive};

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn invoice_path() -> PathBuf {
    repo_root().join("examples/published/invoice.K2F")
}

pub fn invoice() -> OpenedDocument {
    OpenedDocument::open(&std::fs::read(invoice_path()).unwrap()).unwrap()
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
