#![allow(dead_code)]

use k2f_core::{for_each_node, NodeContent, TableDataSource};
use k2f_paint::OpenedDocument;
use std::collections::HashSet;
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

pub fn contract_path() -> PathBuf {
    repo_root().join("examples/published/contract.K2F")
}

pub fn contract() -> OpenedDocument {
    OpenedDocument::open(&std::fs::read(contract_path()).unwrap()).unwrap()
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

pub fn bytes_in(pptx: &[u8], name: &str) -> Vec<u8> {
    let mut zip = ZipArchive::new(Cursor::new(pptx)).expect("pptx zip");
    let mut file = zip
        .by_name(name)
        .unwrap_or_else(|_| panic!("missing {name}"));
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}

pub fn unzip_names(pptx: &[u8]) -> Vec<String> {
    let mut zip = ZipArchive::new(Cursor::new(pptx)).expect("pptx zip");
    let mut names: Vec<String> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().replace('\\', "/"))
        .collect();
    names.sort();
    names
}

pub fn xml_in(pptx: &[u8], name: &str) -> String {
    let mut zip = ZipArchive::new(Cursor::new(pptx)).expect("pptx zip");
    let mut file = zip
        .by_name(name)
        .unwrap_or_else(|_| panic!("missing {name}"));
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    buf
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

/// Node ids belonging to a harvestable plain-text table (table + cell descendants).
pub fn native_table_member_ids(doc: &OpenedDocument) -> HashSet<String> {
    let mut ids = HashSet::new();
    let mut visit = |root: &k2f_core::SemanticNode| {
        for_each_node(root, &mut |n| {
            let NodeContent::Table(spec) = &n.content else {
                return;
            };
            let TableDataSource::Inline { rows } = &spec.data else {
                return;
            };
            let plain = rows
                .iter()
                .flatten()
                .all(|c| matches!(c.content, NodeContent::Text(_)));
            if !plain {
                return;
            }
            ids.insert(n.id.clone());
            for cell in rows.iter().flatten() {
                for_each_node(cell, &mut |c| {
                    ids.insert(c.id.clone());
                });
            }
        });
    };
    visit(doc.semantic_root());
    for rb in doc.running_blocks() {
        visit(&rb.node);
    }
    ids
}
