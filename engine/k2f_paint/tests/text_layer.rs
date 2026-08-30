mod common;

use k2f_layout::LayoutEngine;
use k2f_paint::OpenedDocument;

#[test]
fn compiled_hi_emits_one_span_with_both_chars() {
    let content = r#"{
        "id": "hi",
        "role": "body",
        "content": { "type": "text", "value": "Hi" }
    }"#;
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
        &common::font_bytes(),
    )
    .unwrap();
    let lock: k2f_core::LockFile = serde_json::from_str(&json).unwrap();
    let root: k2f_core::SemanticNode = serde_json::from_str(content).unwrap();
    let spans = k2f_paint::spans_for_page(&lock.geometry.pages[0], &root, &[]);
    assert!(!spans.is_empty());
    let hi = spans.iter().find(|s| s.node_id == "hi").expect("hi span");
    assert_eq!(hi.text, "Hi");
    assert_eq!(hi.char_start, 0);
    assert_eq!(hi.char_end, 2);
    assert!(hi.width_pt > 0.0);
    assert!(hi.height_pt > 0.0);
}

#[test]
fn invoice_page0_spans_are_nonempty() {
    let doc = OpenedDocument::open(&std::fs::read(common::repo_root().join("examples/published/invoice.K2F")).unwrap())
        .unwrap();
    let spans = doc.text_layer(0);
    assert!(
        !spans.is_empty(),
        "invoice page 0 must expose selectable text"
    );
    assert!(
        spans.iter().all(|s| !s.text.is_empty()),
        "every span must carry source text"
    );
}

fn page_with(geo: k2f_core::GeometryNode) -> k2f_core::Page {
    k2f_core::Page {
        index: 0,
        width: k2f_core::Pt(200_000),
        height: k2f_core::Pt(200_000),
        root: geo,
    }
}

fn text_root(id: &str, value: &str) -> k2f_core::SemanticNode {
    k2f_core::SemanticNode {
        id: id.into(),
        role: "body".into(),
        content: k2f_core::NodeContent::Text(value.into()),
        ..Default::default()
    }
}

fn gp(cluster: u32, x: i128, advance: i128) -> k2f_core::GlyphPosition {
    k2f_core::GlyphPosition {
        glyph_id: 1,
        cluster,
        x_offset: k2f_core::Pt(x),
        y_offset: k2f_core::Pt(0),
        x_advance: k2f_core::Pt(advance),
        y_advance: k2f_core::Pt(0),
    }
}

#[test]
fn old_lock_without_clusters_emits_whole_node_span() {
    let geo = k2f_core::GeometryNode {
        id: "old".into(),
        x: k2f_core::Pt(1000),
        y: k2f_core::Pt(2000),
        width: k2f_core::Pt(10_000),
        height: k2f_core::Pt(12_000),
        glyphs: vec![gp(0, 0, 5000), gp(0, 5000, 5000)],
        text_runs: vec![],
        fill_rects: vec![],
        children: vec![],
    };
    let spans = k2f_paint::spans_for_page(&page_with(geo), &text_root("old", "Hi"), &[]);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].text, "Hi");
    assert_eq!(spans[0].char_start, 0);
    assert_eq!(spans[0].char_end, 2);
}

#[test]
fn decorative_glyphs_are_not_in_the_span() {
    let geo = k2f_core::GeometryNode {
        id: "item".into(),
        x: k2f_core::Pt(0),
        y: k2f_core::Pt(0),
        width: k2f_core::Pt(20_000),
        height: k2f_core::Pt(12_000),
        glyphs: vec![
            gp(k2f_core::GlyphPosition::CLUSTER_NOT_SOURCE, 0, 2000),
            gp(0, 3000, 5000),
            gp(1, 8000, 5000),
        ],
        text_runs: vec![],
        fill_rects: vec![],
        children: vec![],
    };
    let spans = k2f_paint::spans_for_page(&page_with(geo), &text_root("item", "Hi"), &[]);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].text, "Hi");
    assert_eq!(spans[0].x_pt, 3.0);
    assert!(spans[0].width_pt > 9.0);
}

#[test]
fn wrapped_lines_emit_one_span_each() {
    let json = LayoutEngine::compile_chunk(
        r#"{
            "title": "wrap",
            "canvas_mode": "paged",
            "page_config": { "width": 50000, "height": 200000, "margin": [10000, 10000, 10000, 10000] },
            "root": {
                "id": "wrap",
                "role": "body",
                "content": { "type": "text", "value": "Hello world" }
            }
        }"#,
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
        &common::font_bytes(),
    )
    .unwrap();
    let lock: k2f_core::LockFile = serde_json::from_str(&json).unwrap();
    let root = k2f_core::SemanticNode {
        id: "wrap".into(),
        role: "body".into(),
        content: k2f_core::NodeContent::Text("Hello world".into()),
        ..Default::default()
    };
    let spans = k2f_paint::spans_for_page(&lock.geometry.pages[0], &root, &[]);
    let wrap: Vec<_> = spans.iter().filter(|s| s.node_id == "wrap").collect();
    assert!(
        wrap.len() >= 2,
        "narrow page must wrap into line spans, got {wrap:?}"
    );
    let joined: String = wrap.iter().map(|s| s.text.as_str()).collect();
    assert!(joined.contains("Hello"), "{joined}");
    assert!(joined.contains("world"), "{joined}");
}
