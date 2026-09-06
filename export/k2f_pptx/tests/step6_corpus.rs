mod common;

use k2f_paint::OpenedDocument;
use k2f_pptx::{export_bytes, export_opened, PptxError};
use std::path::Path;

fn slide_xml_names(pptx: &[u8]) -> Vec<String> {
    common::unzip_names(pptx)
        .into_iter()
        .filter(|n| {
            n.starts_with("ppt/slides/slide") && n.ends_with(".xml") && !n.contains("_rels")
        })
        .collect()
}

fn all_slide_xml(pptx: &[u8]) -> String {
    slide_xml_names(pptx)
        .iter()
        .map(|n| common::xml_in(pptx, n))
        .collect::<Vec<_>>()
        .join("\n")
}

fn slide_count(pptx: &[u8]) -> usize {
    slide_xml_names(pptx).len()
}

fn sld_sz(pptx: &[u8]) -> (i64, i64) {
    let xml = common::xml_in(pptx, "ppt/presentation.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let sz = parsed
        .descendants()
        .find(|n| n.has_tag_name("sldSz"))
        .expect("sldSz");
    (
        sz.attribute("cx").unwrap().parse().unwrap(),
        sz.attribute("cy").unwrap().parse().unwrap(),
    )
}

fn open_if_present(path: &Path) -> Option<OpenedDocument> {
    if !path.is_file() {
        return None;
    }
    Some(OpenedDocument::open(&std::fs::read(path).unwrap()).unwrap())
}

#[test]
fn invoice_round_structure() {
    let doc = common::invoice();
    let pptx = export_opened(&doc).unwrap();
    let lock = doc.lock().expect("invoice is locked");
    assert_eq!(slide_count(&pptx), lock.geometry.pages.len());
    let xml = all_slide_xml(&pptx);
    assert!(
        xml.contains("<p:txBody>"),
        "invoice must keep editable txBody"
    );
    if !common::native_table_member_ids(&doc).is_empty() {
        assert!(
            xml.contains("<a:tbl>"),
            "invoice has a plain-text Table; export must emit a:tbl (not a stamp PNG)"
        );
    }
}

#[test]
fn contract_opens() {
    let Some(doc) = open_if_present(&common::contract_path()) else {
        return;
    };
    let pptx = export_opened(&doc).expect("contract.K2F must export");
    let lock = doc.lock().expect("contract is locked");
    assert_eq!(slide_count(&pptx), lock.geometry.pages.len());
    assert!(
        all_slide_xml(&pptx).contains("<p:txBody>"),
        "contract corpus must keep native text boxes"
    );
}

#[test]
fn widescreen_slide_size() {
    let path = common::fixture_path("blank_widescreen.K2F");
    assert!(
        path.is_file(),
        "submit tests/fixtures/blank_widescreen.K2F (one page, one line, 960000×540000 millipt)"
    );
    let bytes = std::fs::read(&path).unwrap();
    let doc = OpenedDocument::open(&bytes).unwrap();
    let lock = doc.lock().expect("widescreen fixture must be locked");
    let page = &lock.geometry.pages[0];
    assert_eq!(page.width.0, 960_000);
    assert_eq!(page.height.0, 540_000);
    let pptx = export_opened(&doc).unwrap();
    assert_eq!(slide_count(&pptx), 1);
    assert_eq!(sld_sz(&pptx), (12_192_000, 6_858_000));
    let slide1 = common::xml_in(&pptx, "ppt/slides/slide1.xml");
    assert!(
        slide1.contains("<p:txBody>"),
        "widescreen fixture must export editable text, not a stamp"
    );
    assert!(
        slide1.contains("Widescreen title"),
        "fixture is one page with one visible line"
    );
}

#[test]
fn reject_pptx() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let err = export_bytes(&pptx).unwrap_err();
    assert!(
        matches!(err, PptxError::NotASource),
        "got {err}; Display must be PPTX_IS_NOT_A_SOURCE"
    );
    assert_eq!(err.to_string(), "PPTX_IS_NOT_A_SOURCE");
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
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_k2f-pptx"))
        .arg("--help")
        .output()
        .expect("run k2f-pptx --help");
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
        lower.contains("not a second layout engine") || lower.contains("not a second typesetting"),
        "help must say this is not a second layout engine:\n{text}"
    );
}

#[test]
fn cli_export_pptx_exits_one() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let dir = std::env::temp_dir().join("k2f_pptx_step6");
    std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("roundtrip.pptx");
    let dest = dir.join("should-not-write.pptx");
    std::fs::write(&src, &pptx).unwrap();
    let _ = std::fs::remove_file(&dest);
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_k2f-pptx"))
        .args([
            "export",
            src.to_str().unwrap(),
            "-o",
            dest.to_str().unwrap(),
        ])
        .output()
        .expect("run k2f-pptx export");
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("PPTX_IS_NOT_A_SOURCE"),
        "stderr must print PPTX_IS_NOT_A_SOURCE, got {err}"
    );
    assert!(
        !dest.is_file(),
        "failed export must not write an output file"
    );
}
