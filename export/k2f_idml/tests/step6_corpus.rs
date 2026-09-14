mod common;

use k2f_core::{for_each_node, NodeContent};
use k2f_idml::{export_bytes, export_opened, harvestable_inline_rows, IdmlError};
use k2f_paint::OpenedDocument;
use std::path::Path;

fn spread_count(idml: &[u8]) -> usize {
    common::unzip_names(idml)
        .into_iter()
        .filter(|n| n.starts_with("Spreads/Spread_k") && n.ends_with(".xml"))
        .count()
}

fn spreads_blob(idml: &[u8]) -> String {
    common::unzip_names(idml)
        .into_iter()
        .filter(|n| n.starts_with("Spreads/") && n.ends_with(".xml"))
        .map(|n| common::xml_in(idml, &n))
        .collect::<Vec<_>>()
        .join("\n")
}

fn stories_blob(idml: &[u8]) -> String {
    common::unzip_names(idml)
        .into_iter()
        .filter(|n| n.starts_with("Stories/") && n.ends_with(".xml"))
        .map(|n| common::xml_in(idml, &n))
        .collect::<Vec<_>>()
        .join("\n")
}

fn has_harvestable_table(doc: &OpenedDocument) -> bool {
    let mut found = false;
    for_each_node(doc.semantic_root(), &mut |n| {
        if let NodeContent::Table(spec) = &n.content {
            if harvestable_inline_rows(spec).is_some() {
                found = true;
            }
        }
    });
    found
}

fn pref_page_size(idml: &[u8]) -> (String, String) {
    let xml = common::xml_in(idml, "Resources/Preferences.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let pref = parsed
        .descendants()
        .find(|n| n.has_tag_name("DocumentPreference"))
        .expect("DocumentPreference");
    (
        pref.attribute("PageWidth").expect("PageWidth").to_string(),
        pref.attribute("PageHeight")
            .expect("PageHeight")
            .to_string(),
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
    let idml = export_opened(&doc).unwrap();
    let lock = doc.lock().expect("invoice is locked");
    assert_eq!(spread_count(&idml), lock.geometry.pages.len());
    assert!(
        spreads_blob(&idml).contains("<TextFrame"),
        "invoice must keep editable TextFrame, not a full-page stamp"
    );
    if has_harvestable_table(&doc) {
        assert!(
            stories_blob(&idml).contains("<Table"),
            "invoice has a harvestable Table; export must emit <Table"
        );
    }
}

#[test]
fn contract_opens() {
    let Some(doc) = open_if_present(&common::contract_path()) else {
        return;
    };
    let idml = export_opened(&doc).expect("contract.K2F must export");
    let lock = doc.lock().expect("contract is locked");
    assert_eq!(spread_count(&idml), lock.geometry.pages.len());
}

#[test]
fn widescreen_page_size() {
    let path = common::fixture_path("blank_widescreen.K2F");
    assert!(
        path.is_file(),
        "submit tests/fixtures/blank_widescreen.K2F (one page, 960000×540000 millipt)"
    );
    let bytes = std::fs::read(&path).unwrap();
    let doc = OpenedDocument::open(&bytes).unwrap();
    let lock = doc.lock().expect("widescreen fixture must be locked");
    let page = &lock.geometry.pages[0];
    assert_eq!(page.width.0, 960_000);
    assert_eq!(page.height.0, 540_000);
    let idml = export_opened(&doc).unwrap();
    assert_eq!(spread_count(&idml), lock.geometry.pages.len());
    assert_eq!(pref_page_size(&idml), ("960.000".into(), "540.000".into()));
}

#[test]
fn reject_idml() {
    let idml = export_opened(&common::invoice()).unwrap();
    let err = export_bytes(&idml).unwrap_err();
    assert!(
        matches!(err, IdmlError::NotASource),
        "got {err}; Display must be IDML_IS_NOT_A_SOURCE"
    );
    assert_eq!(err.to_string(), "IDML_IS_NOT_A_SOURCE");
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
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_k2f-idml"))
        .arg("--help")
        .output()
        .expect("run k2f-idml --help");
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

#[test]
fn cli_export_idml_exits_one() {
    let idml = export_opened(&common::invoice()).unwrap();
    let dir = std::env::temp_dir().join("k2f_idml_step6");
    std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("roundtrip.idml");
    let dest = dir.join("should-not-write.idml");
    std::fs::write(&src, &idml).unwrap();
    let _ = std::fs::remove_file(&dest);
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_k2f-idml"))
        .args([
            "export",
            src.to_str().unwrap(),
            "-o",
            dest.to_str().unwrap(),
        ])
        .output()
        .expect("run k2f-idml export");
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("IDML_IS_NOT_A_SOURCE"),
        "stderr must print IDML_IS_NOT_A_SOURCE, got {err}"
    );
    assert!(
        !dest.is_file(),
        "failed export must not write an output file"
    );
}
