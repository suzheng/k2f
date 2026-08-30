#![allow(dead_code)]

pub mod extract;

use k2f_core::GeometryNode;
use k2f_paint::OpenedDocument;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn invoice() -> OpenedDocument {
    let bytes = std::fs::read(repo_root().join("examples/published/invoice.K2F")).unwrap();
    OpenedDocument::open(&bytes).unwrap()
}

pub fn contract() -> OpenedDocument {
    let bytes = std::fs::read(repo_root().join("examples/published/contract.K2F")).unwrap();
    OpenedDocument::open(&bytes).unwrap()
}

pub fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.02
}

pub fn index<'a>(n: &'a GeometryNode, out: &mut HashMap<&'a str, &'a GeometryNode>) {
    out.insert(n.id.as_str(), n);
    for c in &n.children {
        index(c, out);
    }
}
