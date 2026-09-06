//! Real compile: role `letter_spacing_pt` changes glyph advances in the lock.
use k2f_core::{collect_boxes, GeometryNode, LockFile, Pt};
use k2f_layout::LayoutEngine;
use serde_json::json;
use std::path::PathBuf;

fn roboto() -> Vec<u8> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/Roboto-Regular.ttf");
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

fn theme_with_tracking(letter_spacing_pt: i64) -> String {
    theme_with_tracking_align(letter_spacing_pt, "start")
}

fn theme_with_tracking_align(letter_spacing_pt: i64, text_align: &str) -> String {
    json!({
        "palette": { "ink": "#111111", "paper": "#FFFFFF" },
        "roles": {
            "h1": {
                "font_family": "default",
                "font_size": 24000,
                "line_height_mult": 1250,
                "bold": true,
                "color": "ink",
                "text_align": text_align,
                "letter_spacing_pt": letter_spacing_pt
            }
        }
    })
    .to_string()
}

fn content() -> &'static str {
    r#"{
        "title": "tracking",
        "canvas_mode": "paged",
        "page_config": { "width": 595000, "height": 842000, "margin": [72000, 72000, 72000, 72000] },
        "root": {
            "id": "title",
            "role": "h1",
            "content": { "type": "text", "value": "Title" }
        }
    }"#
}

fn content_words() -> &'static str {
    r#"{
        "title": "tracking",
        "canvas_mode": "paged",
        "page_config": { "width": 595000, "height": 842000, "margin": [72000, 72000, 72000, 72000] },
        "root": {
            "id": "title",
            "role": "h1",
            "content": { "type": "text", "value": "A B" }
        }
    }"#
}

fn compile(theme: &str) -> LockFile {
    let json = LayoutEngine::compile_chunk(content(), theme, &roboto())
        .unwrap_or_else(|e| panic!("compile: {e}"));
    serde_json::from_str(&json).unwrap()
}

fn compile_words(theme: &str) -> LockFile {
    let json = LayoutEngine::compile_chunk(content_words(), theme, &roboto())
        .unwrap_or_else(|e| panic!("compile: {e}"));
    serde_json::from_str(&json).unwrap()
}

fn title_node(lock: &LockFile) -> &GeometryNode {
    let mut found = Vec::new();
    collect_boxes(&lock.geometry.pages[0].root, "title", &mut found);
    found
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing title geometry"))
}

fn run_advance(node: &GeometryNode) -> Pt {
    node.glyphs.iter().fold(Pt::ZERO, |acc, g| acc + g.x_advance)
}

#[test]
fn official_h1_negative_tracking_tightens_glyph_advances() {
    let tracked = compile(&theme_with_tracking(-500));
    let zero = compile(&theme_with_tracking(0));
    let t = title_node(&tracked);
    let z = title_node(&zero);

    assert!(
        t.glyphs.len() >= 2,
        "expected multiple glyphs, got {}",
        t.glyphs.len()
    );
    assert_eq!(t.glyphs.len(), z.glyphs.len());

    let last = t.glyphs.len() - 1;
    assert_eq!(
        t.glyphs[last].x_advance, z.glyphs[last].x_advance,
        "last glyph must not receive tracking"
    );
    assert!(
        t.glyphs[0].x_advance < z.glyphs[0].x_advance,
        "first glyph advance should shrink with -500 tracking: {:?} vs {:?}",
        t.glyphs[0].x_advance,
        z.glyphs[0].x_advance
    );
    assert!(
        run_advance(t) < run_advance(z),
        "total run advance with tracking {:?} must be < zero tracking {:?}",
        run_advance(t),
        run_advance(z)
    );
    assert!(
        t.width <= z.width,
        "wrap/layout width should not grow under negative tracking"
    );
}

fn ink_left_right(node: &GeometryNode) -> (Pt, Pt) {
    let mut left = Pt(i128::MAX);
    let mut right = Pt(i128::MIN);
    for g in &node.glyphs {
        if g.x_offset < left {
            left = g.x_offset;
        }
        let r = g.x_offset + g.x_advance;
        if r > right {
            right = r;
        }
    }
    (left, right)
}

#[test]
fn centered_tracking_keeps_ink_centered_across_word_gaps() {
    let lock = compile_words(&theme_with_tracking_align(2500, "center"));
    let node = title_node(&lock);
    assert!(
        node.glyphs.len() >= 3,
        "expected glyphs for 'A B', got {}",
        node.glyphs.len()
    );
    let (left, right) = ink_left_right(node);
    let leftover_left = left;
    let leftover_right = node.width - right;
    let drift = leftover_left.0 - leftover_right.0;
    assert!(
        drift.abs() <= 1,
        "tracking must be in the centered width: leftover_left={left:?} leftover_right={leftover_right:?} drift={drift} box={:?}",
        node.width
    );
}
