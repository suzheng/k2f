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
    json!({
        "palette": { "ink": "#111111", "paper": "#FFFFFF" },
        "roles": {
            "h1": {
                "font_family": "default",
                "font_size": 24000,
                "line_height_mult": 1250,
                "bold": true,
                "color": "ink",
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

fn compile(theme: &str) -> LockFile {
    let json = LayoutEngine::compile_chunk(content(), theme, &roboto())
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
