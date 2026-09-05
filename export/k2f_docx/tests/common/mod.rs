#![allow(dead_code)]

use k2f_paint::OpenedDocument;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use zip::ZipArchive;

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

pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub fn open_fixture(name: &str) -> Option<OpenedDocument> {
    let path = fixtures_dir().join(name);
    if !path.is_file() {
        return None;
    }
    Some(OpenedDocument::open(&std::fs::read(path).unwrap()).unwrap())
}

pub fn unzip_names(docx: &[u8]) -> Vec<String> {
    let mut zip = ZipArchive::new(Cursor::new(docx)).expect("docx zip");
    let mut names: Vec<String> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().replace('\\', "/"))
        .collect();
    names.sort();
    names
}

pub fn xml_in(docx: &[u8], name: &str) -> String {
    let mut zip = ZipArchive::new(Cursor::new(docx)).expect("docx zip");
    let mut file = zip
        .by_name(name)
        .unwrap_or_else(|_| panic!("missing {name}"));
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    buf
}

pub fn bytes_in(docx: &[u8], name: &str) -> Vec<u8> {
    let mut zip = ZipArchive::new(Cursor::new(docx)).expect("docx zip");
    let mut file = zip
        .by_name(name)
        .unwrap_or_else(|_| panic!("missing {name}"));
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}

pub fn local_attr<'a>(node: &roxmltree::Node<'a, '_>, name: &str) -> Option<&'a str> {
    node.attributes()
        .find(|a| a.name() == name)
        .map(|a| a.value())
}

pub fn anchor_named<'a, 'input>(
    doc: &'a roxmltree::Document<'input>,
    name: &str,
) -> Option<roxmltree::Node<'a, 'input>> {
    doc.descendants()
        .find(|n| n.has_tag_name("docPr") && local_attr(n, "name") == Some(name))
        .and_then(|pr| pr.ancestors().find(|n| n.has_tag_name("anchor")))
}

pub fn pos_xy(anchor: roxmltree::Node<'_, '_>) -> (i64, i64) {
    let x = anchor
        .descendants()
        .find(|n| n.has_tag_name("positionH"))
        .and_then(|h| h.descendants().find(|n| n.has_tag_name("posOffset")))
        .and_then(|n| n.text())
        .expect("positionH posOffset")
        .parse()
        .unwrap();
    let y = anchor
        .descendants()
        .find(|n| n.has_tag_name("positionV"))
        .and_then(|v| v.descendants().find(|n| n.has_tag_name("posOffset")))
        .and_then(|n| n.text())
        .expect("positionV posOffset")
        .parse()
        .unwrap();
    (x, y)
}
