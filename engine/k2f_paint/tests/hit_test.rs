mod common;

use k2f_paint::OpenedDocument;

fn invoice() -> OpenedDocument {
    OpenedDocument::open(&std::fs::read(common::repo_root().join("examples/published/invoice.K2F")).unwrap())
        .unwrap()
}

#[test]
fn click_center_of_invoice_total_hits_that_id() {
    let doc = invoice();
    let boxes = doc.boxes_for("invoice.total");
    assert!(
        !boxes.is_empty(),
        "invoice.total must have geometry on the published lock"
    );
    let b = &boxes[0];
    let hit = doc
        .hit_test(b.page, b.x + b.width / 2, b.y + b.height / 2)
        .expect("center of invoice.total must hit");
    assert_eq!(hit.leaf(), Some("invoice.total"));
    assert!(hit.ids.contains(&"invoice.total".to_string()));
    let sel = doc.selection("invoice.total").unwrap();
    assert!(sel.text.unwrap().contains("7,047.00"));
    assert_eq!(sel.id, "invoice.total");
}

#[test]
fn click_on_text_span_returns_one_char_from_cluster() {
    let doc = invoice();
    let spans = doc.text_layer(0);
    let span = spans
        .iter()
        .find(|s| s.node_id == "invoice.header")
        .expect("header span on page 0");
    let x = ((span.x_pt + span.width_pt * 0.5) * 1000.0).round() as i64;
    let y = ((span.y_pt + span.height_pt * 0.5) * 1000.0).round() as i64;
    let hit = doc
        .hit_test(0, x, y)
        .expect("center of a text span must hit");
    assert_eq!(hit.leaf(), Some("invoice.header"));
    let range = hit.char_range.expect("clustered lock");
    assert_eq!(range[1] - range[0], 1, "v0 hit is one character, got {range:?}");
}

#[test]
fn padding_around_child_selects_parent_card_not_a_miss() {
    let doc = invoice();
    let boxes = doc.boxes_for("invoice.header");
    assert!(!boxes.is_empty());
    let b = &boxes[0];
    let hit = doc.hit_test(b.page, b.x + 1, b.y + 1).unwrap();
    assert_eq!(hit.leaf(), Some("invoice.header"));
}

#[test]
fn search_returns_ids_not_a_flat_string() {
    let doc = invoice();
    let hits = doc.search("7,047.00");
    assert_eq!(hits, vec!["invoice.total"]);
    assert_eq!(
        doc.search("Consulting Services"),
        vec!["invoice.row_1.item"]
    );
    assert!(doc.search("").is_empty());
}

#[test]
fn miss_in_margin_is_none() {
    let doc = invoice();
    assert!(doc.hit_test(0, 1000, 1000).is_none());
}

/// Frozen millipts from examples/published/invoice.K2F lock geometry. Layout drift fails this.
#[test]
fn golden_pts_on_invoice_lock_return_stable_ids() {
    let doc = invoice();
    let total = doc.hit_test(2, 297_500, 258_200).unwrap();
    assert_eq!(total.ids, vec!["invoice.total"]);
    let sel = doc.selection_from_hit(&total).unwrap();
    assert_eq!(sel.ids[0], "invoice.total");
    assert!(sel.text.unwrap().contains("7,047.00"));

    let header = doc.hit_test(0, 96_001, 150_001).unwrap();
    assert_eq!(header.ids, vec!["invoice.header"]);

    let cell = doc.hit_test(1, 466_417, 141_100).unwrap();
    assert_eq!(cell.ids, vec!["invoice.row_1.amount", "invoice.table"]);

    assert!(doc.hit_test(0, 1_000, 1_000).is_none());
}

#[test]
fn clipboard_is_semantic_json_not_ocr() {
    let doc = invoice();
    let clip = doc.clipboard("invoice.total").unwrap();
    assert_eq!(clip.id, "invoice.total");
    assert!(clip.text.unwrap().contains("7,047.00"));
    assert_eq!(clip.node["id"], "invoice.total");
    assert_eq!(clip.node["content"]["type"], "text");
    assert!(clip.node.get("x").is_none());
    assert!(clip.node.get("y").is_none());
}
