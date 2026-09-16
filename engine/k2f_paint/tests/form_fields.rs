mod common;

use k2f_core::{collect_boxes, GeometryNode, NodeContent, PaintOp};
use k2f_layout::compile_manifest;
use k2f_package::{apply_coverage_subset, load_dir, pack_bytes};
use k2f_paint::{FormFieldLoc, OpenedDocument};

fn pack_form_doc() -> Vec<u8> {
    let mut pkg = load_dir(&common::repo_root().join("templates/blank")).expect("templates/blank");
    pkg.root = serde_json::from_str(FORM_ROOT_JSON).expect("form root json");
    apply_coverage_subset(&mut pkg.fonts).expect("subset fonts");
    let assets: std::collections::HashMap<String, Vec<u8>> =
        pkg.assets.clone().into_iter().collect();
    let lock = compile_manifest(
        pkg.engine_manifest(),
        &pkg.theme_json,
        &pkg.fonts,
        if assets.is_empty() {
            None
        } else {
            Some(&assets)
        },
    )
    .unwrap_or_else(|e| panic!("compile form pack: {e}"));
    pkg.set_lock(&lock).expect("set_lock");
    pack_bytes(&pkg).expect("pack")
}

const FORM_ROOT_JSON: &str = r#"{
    "id": "root",
    "role": "document",
    "layout": { "type": "stack", "direction": "vertical", "gap": 8000 },
    "content": {
        "type": "container",
        "value": {
            "children": [
                {
                    "id": "root.title",
                    "role": "h1",
                    "content": { "type": "text", "value": "Application" }
                },
                {
                    "id": "root.name",
                    "role": "form_field",
                    "variant": "underline",
                    "break_inside": "avoid",
                    "content": {
                        "type": "form_field",
                        "value": {
                            "kind": "text",
                            "value": "",
                            "placeholder": "Full legal name"
                        }
                    }
                },
                {
                    "id": "root.sign",
                    "role": "body",
                    "content": { "type": "text", "value": "Signature" }
                },
                {
                    "id": "root.agree",
                    "role": "form_field",
                    "variant": "checkbox",
                    "break_inside": "avoid",
                    "content": {
                        "type": "form_field",
                        "value": { "kind": "checkbox", "value": "" }
                    }
                }
            ]
        }
    }
}"#;

fn geo_for<'a>(doc: &'a OpenedDocument, id: &str) -> Vec<&'a GeometryNode> {
    let lock = doc.lock().expect("locked");
    let mut out = Vec::new();
    for page in &lock.geometry.pages {
        collect_boxes(&page.root, id, &mut out);
    }
    out
}

fn field_ops<'a>(doc: &'a OpenedDocument, id: &str) -> Vec<&'a PaintOp> {
    doc.lock()
        .expect("locked")
        .render_plan
        .pages
        .iter()
        .flat_map(|p| p.ops.iter())
        .filter(|op| match op {
            PaintOp::DrawBox { node_id, .. } | PaintOp::DrawText { node_id, .. } => node_id == id,
            _ => false,
        })
        .collect()
}

#[test]
fn form_fields_lists_id_kind_and_rect_from_real_pack() {
    let doc = OpenedDocument::open(&pack_form_doc()).unwrap();
    let fields = doc.form_fields();
    assert_eq!(fields.len(), 2, "name + checkbox");

    let name = fields.iter().find(|f| f.id == "root.name").unwrap();
    assert_eq!(name.kind, k2f_core::FormFieldKind::Text);
    assert_eq!(name.value, "");
    assert_eq!(name.placeholder.as_deref(), Some("Full legal name"));
    assert!(
        name.width > 0 && name.height > 0,
        "rect must be non-empty {name:?}"
    );
    assert_eq!(doc.boxes_for("root.name").len(), 1, "v1 field is one box");

    let agree = fields.iter().find(|f| f.id == "root.agree").unwrap();
    assert_eq!(agree.kind, k2f_core::FormFieldKind::Checkbox);
    assert_eq!(agree.value, "");

    let json: Vec<FormFieldLoc> =
        serde_json::from_str(&serde_json::to_string(&fields).unwrap()).unwrap();
    assert_eq!(json, fields);

    let name_ops = field_ops(&doc, "root.name");
    assert!(
        name_ops
            .iter()
            .any(|op| matches!(op, PaintOp::DrawBox { node_id, .. } if node_id == "root.name")),
        "empty field must still paint DrawBox"
    );
    assert!(
        !name_ops
            .iter()
            .any(|op| matches!(op, PaintOp::DrawText { node_id, .. } if node_id == "root.name")),
        "empty field must not paint DrawText"
    );
}

#[test]
fn invoice_without_form_fields_returns_empty() {
    let doc = OpenedDocument::open(
        &std::fs::read(common::repo_root().join("examples/published/invoice.K2F")).unwrap(),
    )
    .unwrap();
    assert!(doc.form_fields().is_empty());
}

#[test]
fn hit_test_form_field_center_returns_field_id() {
    let doc = OpenedDocument::open(&pack_form_doc()).unwrap();
    let boxes = doc.boxes_for("root.name");
    assert_eq!(boxes.len(), 1);
    let b = &boxes[0];
    let hit = doc
        .hit_test(b.page, b.x + b.width / 2, b.y + b.height / 2)
        .expect("center of form field must hit");
    assert_eq!(hit.leaf(), Some("root.name"));

    let agree = &doc.boxes_for("root.agree")[0];
    let hit = doc
        .hit_test(
            agree.page,
            agree.x + agree.width / 2,
            agree.y + agree.height / 2,
        )
        .expect("center of checkbox must hit");
    assert_eq!(hit.leaf(), Some("root.agree"));
}

#[test]
fn empty_form_field_has_geometry_but_no_glyphs() {
    let doc = OpenedDocument::open(&pack_form_doc()).unwrap();
    let geo = geo_for(&doc, "root.name");
    assert_eq!(geo.len(), 1);
    assert!(geo[0].glyphs.is_empty());
    match &doc.semantic_root().content {
        NodeContent::Container { children } => {
            assert!(children.iter().any(|c| c.id == "root.name"));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn form_fields_json_kind_is_snake_case_for_viewer() {
    let doc = OpenedDocument::open(&pack_form_doc()).unwrap();
    let json = serde_json::to_string(&doc.form_fields()).unwrap();
    assert!(json.contains(r#""kind":"text""#));
    assert!(json.contains(r#""kind":"checkbox""#));
    assert!(
        !json.contains("Multiline") && !json.contains("Checkbox") && !json.contains("Text"),
        "WASM/JS expects snake_case kind strings: {json}"
    );
}

#[test]
fn form_fields_preserves_document_order() {
    let doc = OpenedDocument::open(&pack_form_doc()).unwrap();
    let ids: Vec<_> = doc.form_fields().into_iter().map(|f| f.id).collect();
    assert_eq!(ids, vec!["root.name", "root.agree"]);
}

#[test]
fn unlocked_pack_omits_form_fields_without_boxes() {
    let bytes = pack_form_doc();
    let mut pkg = k2f_package::unpack_bytes(&bytes).unwrap();
    pkg.lock_json = None;
    let doc = OpenedDocument::from_package(pkg).unwrap();
    assert!(
        doc.form_fields().is_empty(),
        "query needs lock boxes; unlocked draft has none"
    );
}
