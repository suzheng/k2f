mod common;

use k2f_core::{for_each_node, NodeContent};
use k2f_docx::{export_bytes, export_opened, pt_to_twips, DocxError};
use k2f_paint::OpenedDocument;
use std::path::Path;

fn page_break_count(document_xml: &str) -> usize {
    let parsed = roxmltree::Document::parse(document_xml).unwrap();
    parsed
        .descendants()
        .filter(|n| n.has_tag_name("br") && common::local_attr(n, "type") == Some("page"))
        .count()
}

fn pg_sz_twips(document_xml: &str) -> (i64, i64) {
    let parsed = roxmltree::Document::parse(document_xml).unwrap();
    let sz = parsed
        .descendants()
        .find(|n| n.has_tag_name("pgSz"))
        .expect("pgSz");
    (
        common::local_attr(&sz, "w").unwrap().parse().unwrap(),
        common::local_attr(&sz, "h").unwrap().parse().unwrap(),
    )
}

fn semantic_has_table(doc: &OpenedDocument) -> bool {
    let mut found = false;
    for_each_node(doc.semantic_root(), &mut |n| {
        if matches!(n.content, NodeContent::Table(_)) {
            found = true;
        }
    });
    found
}

fn open_if_present(path: &Path) -> Option<OpenedDocument> {
    if !path.is_file() {
        return None;
    }
    Some(OpenedDocument::open(&std::fs::read(path).unwrap()).unwrap())
}

#[test]
fn invoice_structure() {
    let doc = common::invoice();
    let docx = export_opened(&doc).unwrap();
    let lock = doc.lock().expect("invoice is locked");
    let xml = common::xml_in(&docx, "word/document.xml");
    assert_eq!(
        page_break_count(&xml) + 1,
        lock.geometry.pages.len(),
        "Word pages (page-breaks + 1) must match lock page count"
    );
    assert!(
        xml.contains("txbxContent"),
        "invoice must keep editable w:txbxContent, not a full-page stamp"
    );
    if semantic_has_table(&doc) {
        assert!(
            !xml.contains("<w:tbl>"),
            "Word Dark Mode inverts w:tbl shading; invoice tables must be boxes + text"
        );
    }
}

#[test]
fn contract_opens() {
    let Some(doc) = open_if_present(&common::contract_path()) else {
        return;
    };
    let docx = export_opened(&doc).expect("contract.K2F must export");
    let lock = doc.lock().expect("contract is locked");
    let xml = common::xml_in(&docx, "word/document.xml");
    assert_eq!(page_break_count(&xml) + 1, lock.geometry.pages.len());
    assert!(
        xml.contains("txbxContent"),
        "contract must export editable text boxes, not a full-page stamp"
    );
    assert!(
        !xml.contains("k2f-raster:"),
        "contract corpus must not rely on effect-slice rasters"
    );
}

#[test]
#[allow(non_snake_case)]
fn pgSz_a4_or_custom() {
    let doc = common::invoice();
    let docx = export_opened(&doc).unwrap();
    let lock = doc.lock().expect("invoice is locked");
    let page = &lock.geometry.pages[0];
    let xml = common::xml_in(&docx, "word/document.xml");
    assert_eq!(
        pg_sz_twips(&xml),
        (pt_to_twips(page.width), pt_to_twips(page.height))
    );
}

#[test]
fn reject_docx() {
    let docx = export_opened(&common::invoice()).unwrap();
    let err = export_bytes(&docx).unwrap_err();
    assert!(
        matches!(err, DocxError::NotASource),
        "got {err}; Display must be DOCX_IS_NOT_A_SOURCE"
    );
    assert_eq!(err.to_string(), "DOCX_IS_NOT_A_SOURCE");
}

#[test]
fn determinism() {
    let doc = common::invoice();
    let a = export_opened(&doc).unwrap();
    let b = export_opened(&doc).unwrap();
    assert_eq!(a, b);
}

#[test]
fn cli_help_mentions_experimental_and_not_a_layout_engine() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_k2f-docx"))
        .arg("--help")
        .output()
        .expect("run k2f-docx --help");
    assert!(out.status.success());
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let lower = text.to_ascii_lowercase();
    assert!(
        lower.contains("experimental"),
        "help must say experimental:\n{text}"
    );
    assert!(
        lower.contains("does not modify") || lower.contains("not modify"),
        "help must say the source package is not modified:\n{text}"
    );
    assert!(
        lower.contains("not a second layout engine"),
        "help must say this is not a second layout engine:\n{text}"
    );
}
