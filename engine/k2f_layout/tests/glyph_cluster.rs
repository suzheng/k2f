use k2f_core::{collect_boxes, GeometryNode, LockFile};
use k2f_layout::LayoutEngine;
use std::path::PathBuf;

fn roboto() -> Vec<u8> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/Roboto-Regular.ttf");
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

fn compile(content: &str) -> LockFile {
    let json = LayoutEngine::compile_chunk(
        content,
        r#"{
            "palette": {},
            "roles": {
                "body": {
                    "font_family": "default",
                    "font_size": 12000,
                    "line_height_mult": 1200,
                    "color": "black"
                }
            }
        }"#,
        &roboto(),
    )
    .unwrap_or_else(|e| panic!("compile: {e}"));
    serde_json::from_str(&json).unwrap()
}

fn node_named<'a>(page_root: &'a GeometryNode, id: &str) -> &'a GeometryNode {
    let mut found = Vec::new();
    collect_boxes(page_root, id, &mut found);
    found
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing geometry node {id}"))
}

#[test]
fn text_node_glyphs_have_monotonic_clusters() {
    let lock = compile(
        r#"{
            "id": "hi",
            "role": "body",
            "content": { "type": "text", "value": "Hi" }
        }"#,
    );
    let geo = node_named(&lock.geometry.pages[0].root, "hi");
    assert!(
        geo.glyphs.len() >= 2,
        "expected a glyph per letter, got {}",
        geo.glyphs.len()
    );
    assert_eq!(geo.glyphs[0].cluster, 0);
    assert_eq!(geo.glyphs[1].cluster, 1);
}

/// CJK byte-vs-char mapping is in `k2f_text::byte_to_char_index` (vendored Noto is Latin-only).
/// Wrapping must still continue character indexes into the original node string.
#[test]
fn wrapped_line_clusters_continue_source_char_indexes() {
    let lock = compile(
        r#"{
            "title": "wrap",
            "canvas_mode": "paged",
            "page_config": { "width": 90000, "height": 200000, "margin": [10000, 10000, 10000, 10000] },
            "root": {
                "id": "wrap",
                "role": "body",
                "content": { "type": "text", "value": "Hello world" }
            }
        }"#,
    );
    let geo = node_named(&lock.geometry.pages[0].root, "wrap");
    let clusters: Vec<u32> = geo.glyphs.iter().map(|g| g.cluster).collect();
    assert!(
        clusters.contains(&0) && clusters.contains(&6),
        "expected clusters for 'H' (0) and 'w' (6) after wrap, got {clusters:?}"
    );
}
