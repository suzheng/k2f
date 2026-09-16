mod common;

use k2f_core::{
    collect_boxes, BreakInside, FormFieldKind, FormFieldSpec, GeometryNode, NodeContent, PaintOp,
    SemanticNode,
};
use k2f_paint::{LocatedBox, OpenedDocument};
use k2f_sdk::Editor;

const LONG_NAME: &str = "Alexandria Catherine Whittaker-Jones";

fn form_field(id: &str, spec: FormFieldSpec, variant: &str) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "form_field".to_string(),
        variant: Some(variant.to_string()),
        break_inside: BreakInside::Avoid,
        content: NodeContent::FormField(spec),
        ..Default::default()
    }
}

fn text_field(id: &str) -> SemanticNode {
    form_field(
        id,
        FormFieldSpec {
            kind: FormFieldKind::Text,
            value: String::new(),
            placeholder: Some("Full legal name".into()),
            width: None,
            height: None,
            lines: None,
            max_length: None,
            required: false,
        },
        "underline",
    )
}

fn checkbox_field(id: &str) -> SemanticNode {
    form_field(
        id,
        FormFieldSpec {
            kind: FormFieldKind::Checkbox,
            value: String::new(),
            placeholder: None,
            width: None,
            height: None,
            lines: None,
            max_length: None,
            required: false,
        },
        "checkbox",
    )
}

fn build_form() -> Editor {
    let mut ed = common::open("blank");
    common::insert_heading(&mut ed, "root", "root.title", 1, "Application");
    common::insert_node(&mut ed, "root", &text_field("root.name"));
    common::insert_text(&mut ed, "root", "root.sign", "body", "Signature");
    common::insert_node(&mut ed, "root", &checkbox_field("root.agree"));
    ed
}

fn rect(b: &LocatedBox) -> (usize, i64, i64, i64, i64) {
    (b.page, b.x, b.y, b.width, b.height)
}

fn only_box(doc: &OpenedDocument, id: &str) -> LocatedBox {
    let boxes = doc.boxes_for(id);
    assert_eq!(
        boxes.len(),
        1,
        "expected one box for {id}, got {}",
        boxes.len()
    );
    boxes.into_iter().next().unwrap()
}

fn geo_nodes<'a>(doc: &'a OpenedDocument, id: &str) -> Vec<&'a GeometryNode> {
    let lock = doc.lock().expect("locked");
    let mut out = Vec::new();
    for page in &lock.geometry.pages {
        collect_boxes(&page.root, id, &mut out);
    }
    out
}

fn has_draw_text(doc: &OpenedDocument, id: &str) -> bool {
    doc.lock()
        .expect("locked")
        .render_plan
        .pages
        .iter()
        .flat_map(|p| p.ops.iter())
        .any(|op| matches!(op, PaintOp::DrawText { node_id, .. } if node_id == id))
}

#[test]
fn replace_text_on_form_field_succeeds() {
    let mut ed = build_form();
    assert_eq!(ed.node_text("root.name").unwrap(), "");
    ed.replace_text("root.name", "Alice").unwrap();
    assert_eq!(ed.node_text("root.name").unwrap(), "Alice");
}

#[test]
fn form_field_fill_and_relock_keeps_signature_geometry() {
    let mut ed = build_form();
    let bytes_a = ed.save_bytes().unwrap();
    let doc_a = OpenedDocument::open(&bytes_a).unwrap();
    let sign_a = only_box(&doc_a, "root.sign");
    let name_a = only_box(&doc_a, "root.name");
    let hash_a = doc_a.content_hash().unwrap().to_string();
    assert!(geo_nodes(&doc_a, "root.name")[0].glyphs.is_empty());
    assert!(!has_draw_text(&doc_a, "root.name"));

    ed.replace_text("root.name", "Alice").unwrap();
    let bytes_b = ed.save_bytes().unwrap();
    let doc_b = OpenedDocument::open(&bytes_b).unwrap();
    let hash_b = doc_b.content_hash().unwrap().to_string();
    assert_ne!(hash_a, hash_b, "filling must change content_hash");
    assert!(!geo_nodes(&doc_b, "root.name")[0].glyphs.is_empty());
    assert!(has_draw_text(&doc_b, "root.name"));
    assert_eq!(rect(&only_box(&doc_b, "root.sign")), rect(&sign_a));
    assert_eq!(rect(&only_box(&doc_b, "root.name")), rect(&name_a));

    ed.replace_text("root.name", LONG_NAME).unwrap();
    let bytes_c = ed.save_bytes().unwrap();
    let doc_c = OpenedDocument::open(&bytes_c).unwrap();
    assert_eq!(rect(&only_box(&doc_c, "root.sign")), rect(&sign_a));
    assert_eq!(rect(&only_box(&doc_c, "root.name")), rect(&name_a));
    let filled = doc_c
        .form_fields()
        .into_iter()
        .find(|f| f.id == "root.name")
        .unwrap();
    assert_eq!(filled.value, LONG_NAME);
}

#[test]
fn form_field_checkbox_replace_text_true_survives_relock() {
    let mut ed = build_form();
    ed.replace_text("root.agree", "true").unwrap();
    assert_eq!(ed.node_text("root.agree").unwrap(), "true");
    let bytes = ed.save_bytes().unwrap();
    let doc = OpenedDocument::open(&bytes).unwrap();
    let agree = doc
        .form_fields()
        .into_iter()
        .find(|f| f.id == "root.agree")
        .unwrap();
    assert_eq!(agree.value, "true");
    assert_eq!(agree.kind, FormFieldKind::Checkbox);
    assert!(!geo_nodes(&doc, "root.agree")[0].glyphs.is_empty());
    assert!(has_draw_text(&doc, "root.agree"));
}
