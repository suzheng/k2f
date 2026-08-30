use k2f_core::LockFile;
use k2f_layout::LayoutEngine;
use std::fs;
use std::path::PathBuf;

fn roboto() -> Vec<u8> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/Roboto-Regular.ttf");
    fs::read(&p).unwrap()
}

fn theme() -> String {
    r#"{"palette":{},"roles":{"body":{"font_family":"default","font_size":12000,"line_height_mult":1200,"color":"black"},"default":{"font_family":"default","font_size":12000,"line_height_mult":1200,"color":"black"}}}"#.into()
}

fn manifest(text: &str) -> String {
    format!(
        r#"{{"title":"t","canvas_mode":"paged","page_config":{{"width":595000,"height":842000,"margin":[72000,72000,72000,72000]}},"root":{{"id":"root","role":"body","content":{{"type":"text","value":{text}}}}},"running_blocks":[]}}"#
    )
}

#[test]
fn nfc_and_nfd_compile_to_the_same_hashes() {
    let font = roboto();
    let nfd = serde_json::to_string("e\u{0301}").unwrap();
    let nfc = serde_json::to_string("\u{00e9}").unwrap();
    let a: LockFile = serde_json::from_str(
        &LayoutEngine::compile_chunk(&manifest(&nfd), &theme(), &font).unwrap(),
    )
    .unwrap();
    let b: LockFile = serde_json::from_str(
        &LayoutEngine::compile_chunk(&manifest(&nfc), &theme(), &font).unwrap(),
    )
    .unwrap();
    assert_eq!(a.content_hash, b.content_hash);
    assert_eq!(a.appearance_hash, b.appearance_hash);
}

#[test]
fn generic_sans_serif_font_fails_compile() {
    let font = roboto();
    let theme = r#"{"palette":{},"roles":{"body":{"font_family":"sans-serif","font_size":12000,"line_height_mult":1200,"color":"black"}}}"#;
    let err = LayoutEngine::compile_chunk(&manifest("\"Hi\""), theme, &font).unwrap_err();
    assert!(err.contains("FONT_MISSING"), "got {err}");
}

#[test]
fn cjk_without_cjk_font_fails() {
    let font = roboto();
    let err = LayoutEngine::compile_chunk(&manifest("\"合同\""), &theme(), &font).unwrap_err();
    assert!(err.contains("FONT_MISSING_GLYPH"), "got {err}");
}

#[test]
fn cjk_compiles_with_embedded_noto_sans_sc() {
    let noto =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/NotoSansSC-Regular.otf");
    let font = fs::read(&noto).unwrap();
    let theme = r#"{"palette":{},"roles":{"body":{"font_family":"NotoSansSC-Regular","font_size":12000,"line_height_mult":1200,"color":"black"},"default":{"font_family":"NotoSansSC-Regular","font_size":12000,"line_height_mult":1200,"color":"black"}}}"#;
    let mut fonts = std::collections::BTreeMap::new();
    fonts.insert("assets/fonts/NotoSansSC-Regular.otf".into(), font);
    let json =
        k2f_layout::compile_chunk_with_fonts(&manifest("\"合同\""), theme, &fonts, None).unwrap();
    let lock: LockFile = serde_json::from_str(&json).unwrap();
    assert_eq!(lock.geometry.pages.len(), 1);
    assert!(
        !lock.geometry.pages[0].root.children.is_empty()
            || !lock.geometry.pages[0].root.glyphs.is_empty()
    );
}
