//! Empty form fields export as DrawBox shapes; filled fields as text boxes.

mod common;

use k2f_core::{BreakInside, FormFieldKind, FormFieldSpec, NodeContent, SemanticNode};
use k2f_paint::OpenedDocument;
use k2f_pptx::export_opened;
use k2f_sdk::Editor;

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

fn spec(kind: FormFieldKind, value: &str, lines: Option<u32>) -> FormFieldSpec {
    FormFieldSpec {
        kind,
        value: value.into(),
        placeholder: None,
        width: None,
        height: None,
        lines,
        max_length: None,
        required: false,
    }
}

fn packed_form() -> Vec<u8> {
    let mut ed = Editor::open_dir(&common::repo_root().join("templates/blank")).unwrap();
    let json = |n: &SemanticNode| serde_json::to_string(n).unwrap();
    ed.insert_node(
        "root",
        0,
        &json(&form_field(
            "root.empty_name",
            spec(FormFieldKind::Text, "", None),
            "underline",
        )),
    )
    .unwrap();
    ed.insert_node(
        "root",
        1,
        &json(&form_field(
            "root.filled_name",
            spec(FormFieldKind::Text, "Alice", None),
            "underline",
        )),
    )
    .unwrap();
    ed.insert_node(
        "root",
        2,
        &json(&form_field(
            "root.empty_box",
            spec(FormFieldKind::Multiline, "", Some(3)),
            "box",
        )),
    )
    .unwrap();
    ed.save_bytes().unwrap()
}

fn slide1(pptx: &[u8]) -> String {
    common::xml_in(pptx, "ppt/slides/slide1.xml")
}

fn has_name(xml: &str, name: &str) -> bool {
    let parsed = roxmltree::Document::parse(xml).unwrap();
    parsed
        .descendants()
        .any(|n| n.has_tag_name("cNvPr") && n.attribute("name") == Some(name))
}

#[test]
fn form_field_empty_keeps_border_shape() {
    let doc = OpenedDocument::open(&packed_form()).unwrap();
    let pptx = export_opened(&doc).unwrap();
    let xml = slide1(&pptx);
    assert!(
        has_name(&xml, "root.empty_name") || has_name(&xml, "root.empty_name::edge_bottom"),
        "empty underline field must remain as a shape, missing cNvPr root.empty_name"
    );
    assert!(
        has_name(&xml, "root.empty_box") || xml.contains("root.empty_box"),
        "empty boxed field must remain as a shape"
    );
}

#[test]
fn form_field_filled_exports_as_textbox() {
    let doc = OpenedDocument::open(&packed_form()).unwrap();
    let pptx = export_opened(&doc).unwrap();
    let xml = slide1(&pptx);
    assert!(
        xml.contains("Alice"),
        "filled field value must appear in slide text"
    );
    assert!(
        has_name(&xml, "root.filled_name") || xml.contains("root.filled_name"),
        "filled field must keep a named shape/textbox"
    );
}
