mod common;

use std::collections::BTreeMap;
use std::io::Cursor;

use common::contract_k2f_bytes;
use k2f_paint::{OpenedDocument, OFFICIAL_PNG_SCALE};
use zip::ZipArchive;

fn invoice_bytes() -> Vec<u8> {
    std::fs::read(common::repo_root().join("examples/published/invoice.K2F"))
        .expect("examples/published/invoice.K2F")
}

fn zip_entries(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut zip = ZipArchive::new(Cursor::new(bytes)).expect("zip");
    let mut out = BTreeMap::new();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).unwrap();
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut f, &mut buf).unwrap();
        out.insert(f.name().to_string(), buf);
    }
    out
}

#[test]
fn invoice_markdown_has_heading_without_hints() {
    let doc = OpenedDocument::open(&invoice_bytes()).unwrap();
    let md = doc.document_markdown().unwrap();
    assert!(md.contains('#'), "invoice markdown must have headings:\n{md}");
    assert!(!md.contains("<!--"), "export markdown must omit k2f hints");
}

#[test]
fn invoice_png_export() {
    let doc = OpenedDocument::open(&invoice_bytes()).unwrap();
    let n = doc.page_count();
    assert!(n >= 1);
    let out = doc.export_pages_png(OFFICIAL_PNG_SCALE).unwrap();
    if n == 1 {
        assert!(out.starts_with(b"\x89PNG"));
    } else {
        assert!(out.starts_with(b"PK"));
        assert_eq!(zip_entries(&out).len(), n);
    }
}

#[test]
fn invoice_jpeg_export() {
    let doc = OpenedDocument::open(&invoice_bytes()).unwrap();
    let n = doc.page_count();
    let out = doc.export_pages_jpeg(OFFICIAL_PNG_SCALE).unwrap();
    if n == 1 {
        assert!(out.starts_with(&[0xff, 0xd8]));
    } else {
        assert!(out.starts_with(b"PK"));
        let entries = zip_entries(&out);
        assert_eq!(entries.len(), n);
        for bytes in entries.values() {
            assert!(bytes.starts_with(&[0xff, 0xd8]));
        }
    }
}

#[test]
fn contract_multi_page_png_zip() {
    let doc = OpenedDocument::open(&contract_k2f_bytes()).unwrap();
    let n = doc.page_count();
    assert!(n > 1, "contract must be multi-page");
    let zip = doc.export_pages_png(OFFICIAL_PNG_SCALE).unwrap();
    assert!(zip.starts_with(b"PK"), "multi-page png export must be zip");
    let entries = zip_entries(&zip);
    assert_eq!(entries.len(), n);
    for (name, bytes) in &entries {
        assert!(name.starts_with("page-") && name.ends_with(".png"), "{name}");
        assert!(bytes.starts_with(b"\x89PNG"), "{name}");
    }
}

#[test]
fn contract_multi_page_jpeg_zip() {
    let doc = OpenedDocument::open(&contract_k2f_bytes()).unwrap();
    let n = doc.page_count();
    let zip = doc.export_pages_jpeg(OFFICIAL_PNG_SCALE).unwrap();
    assert!(zip.starts_with(b"PK"));
    let entries = zip_entries(&zip);
    assert_eq!(entries.len(), n);
    for (name, bytes) in &entries {
        assert!(name.ends_with(".jpg"), "{name}");
        assert!(bytes.starts_with(&[0xff, 0xd8]), "{name}");
    }
}
