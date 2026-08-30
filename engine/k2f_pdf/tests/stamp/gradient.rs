use k2f_layout::compile_manifest;
use k2f_package::{pack_bytes, Package};
use k2f_paint::OpenedDocument;
use k2f_pdf::{export_opened, PdfScale};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn compile_elegant_backgrounds() -> Vec<u8> {
    let dir = repo_root().join("tests/fixtures/cases/elegant_backgrounds");
    let content = fs::read_to_string(dir.join("content.json")).unwrap();
    let theme = fs::read_to_string(dir.join("theme.json")).unwrap();
    let font = fs::read(repo_root().join("assets/fonts/NotoSansSC-Regular.otf")).unwrap();
    let mut fonts = BTreeMap::new();
    fonts.insert("assets/fonts/NotoSansSC-Regular.otf".to_string(), font);
    let manifest: k2f_core::Manifest = serde_json::from_str(&content).unwrap();
    let lock = compile_manifest(manifest.clone(), &theme, &fonts, None).unwrap();
    let mut pkg = Package::from_engine_parts(
        manifest.title.clone(),
        None,
        Some(0),
        manifest,
        theme,
        fonts,
        BTreeMap::new(),
    )
    .unwrap();
    pkg.set_lock(&lock).unwrap();
    pack_bytes(&pkg).unwrap()
}

#[test]
fn linear_gradient_fixture_exports() {
    let bytes = compile_elegant_backgrounds();
    let doc = OpenedDocument::open(&bytes).unwrap();
    let pdf = export_opened(&doc, PdfScale::DEFAULT).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
}
