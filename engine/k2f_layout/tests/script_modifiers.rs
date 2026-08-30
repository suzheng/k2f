//! Superscript / subscript: engine script size + baseline shift in lock glyphs.
use k2f_core::{collect_boxes, GeometryNode, LockFile, Pt};
use k2f_layout::LayoutEngine;
use std::path::PathBuf;

fn roboto() -> Vec<u8> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/Roboto-Regular.ttf");
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

fn theme() -> &'static str {
    r##"{
        "palette": { "ink": "#111111" },
        "roles": {
            "body": {
                "font_family": "default",
                "font_size": 12000,
                "line_height_mult": 1400,
                "color": "ink"
            }
        }
    }"##
}

fn compile(content: &str) -> LockFile {
    let json = LayoutEngine::compile_chunk(content, theme(), &roboto())
        .unwrap_or_else(|e| panic!("compile: {e}"));
    serde_json::from_str(&json).unwrap()
}

fn body_node(lock: &LockFile) -> &GeometryNode {
    let mut found = Vec::new();
    collect_boxes(&lock.geometry.pages[0].root, "body", &mut found);
    found
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing body geometry"))
}

#[test]
fn subscript_glyph_is_smaller_and_lower() {
    // "H2O" with subscript on "2" (byte range [1,2]).
    let content = r#"{
        "title": "sub",
        "canvas_mode": "paged",
        "page_config": { "width": 595000, "height": 842000, "margin": [72000, 72000, 72000, 72000] },
        "root": {
            "id": "body",
            "role": "body",
            "content": { "type": "text", "value": "H2O" },
            "modifiers": [
                { "range": [1, 2], "type": "subscript", "intent": "default" }
            ]
        }
    }"#;
    let lock = compile(content);
    let node = body_node(&lock);
    assert!(node.text_runs.len() >= 2, "expected split runs, got {}", node.text_runs.len());

    let base_size = Pt(12000);
    let script_size = Pt(base_size.0 * 7 / 10);
    let sub_run = node
        .text_runs
        .iter()
        .find(|r| r.style.font_size == script_size)
        .unwrap_or_else(|| panic!("missing script-sized run: {:?}", node.text_runs));
    let base_run = node
        .text_runs
        .iter()
        .find(|r| r.style.font_size == base_size)
        .unwrap_or_else(|| panic!("missing base-sized run"));

    let base_y = node.glyphs[base_run.glyph_range[0]].y_offset;
    let sub_y = node.glyphs[sub_run.glyph_range[0]].y_offset;
    assert!(
        sub_y > base_y,
        "subscript should sit lower (larger y): sub={sub_y:?} base={base_y:?}"
    );
    let expected_drop = Pt(base_size.0 * 250 / 1000);
    assert_eq!(sub_y - base_y, expected_drop);
}

#[test]
fn superscript_glyph_is_smaller_and_higher() {
    let content = r#"{
        "title": "sup",
        "canvas_mode": "paged",
        "page_config": { "width": 595000, "height": 842000, "margin": [72000, 72000, 72000, 72000] },
        "root": {
            "id": "body",
            "role": "body",
            "content": { "type": "text", "value": "x2" },
            "modifiers": [
                { "range": [1, 2], "type": "superscript", "intent": "default" }
            ]
        }
    }"#;
    let lock = compile(content);
    let node = body_node(&lock);
    let base_size = Pt(12000);
    let script_size = Pt(base_size.0 * 7 / 10);
    let sup_run = node
        .text_runs
        .iter()
        .find(|r| r.style.font_size == script_size)
        .expect("script run");
    let base_run = node
        .text_runs
        .iter()
        .find(|r| r.style.font_size == base_size)
        .expect("base run");
    let base_y = node.glyphs[base_run.glyph_range[0]].y_offset;
    let sup_y = node.glyphs[sup_run.glyph_range[0]].y_offset;
    assert!(
        sup_y < base_y,
        "superscript should sit higher (smaller y): sup={sup_y:?} base={base_y:?}"
    );
    let expected_raise = Pt(base_size.0 * 450 / 1000);
    assert_eq!(base_y - sup_y, expected_raise);
}
