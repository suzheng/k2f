//! Fill → dirty Map → Save relock on a real packed form.

mod common;

use k2f_core::{BreakInside, FormFieldKind, FormFieldSpec, NodeContent, SemanticNode};
use k2f_paint::OpenedDocument;
use k2f_reader::ui::Session;
use k2f_reader::AppState;
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

fn packed_form() -> Vec<u8> {
    let mut ed = Editor::open_dir(&common::repo_root().join("templates/blank")).unwrap();
    let json = |n: &SemanticNode| serde_json::to_string(n).unwrap();
    ed.insert_node(
        "root",
        0,
        &json(&SemanticNode {
            id: "root.title".into(),
            role: "h1".into(),
            content: NodeContent::Text("Application".into()),
            keep_with_next: true,
            ..Default::default()
        }),
    )
    .unwrap();
    ed.insert_node(
        "root",
        1,
        &json(&form_field(
            "root.name",
            FormFieldSpec {
                kind: FormFieldKind::Text,
                value: String::new(),
                placeholder: Some("Full legal name".into()),
                width: None,
                height: None,
                lines: None,
                max_length: Some(80),
                required: false,
            },
            "underline",
        )),
    )
    .unwrap();
    ed.insert_node(
        "root",
        2,
        &json(&form_field(
            "root.agree",
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
        )),
    )
    .unwrap();
    ed.insert_node(
        "root",
        3,
        &json(&SemanticNode {
            id: "root.sign".into(),
            role: "body".into(),
            content: NodeContent::Text("Signature".into()),
            ..Default::default()
        }),
    )
    .unwrap();
    ed.save_bytes().unwrap()
}

#[test]
fn form_fill_invoice_has_no_chrome() {
    let app = AppState::open(&common::invoice_bytes()).unwrap();
    assert!(!app.has_form_fields());
}

#[test]
fn form_fill_committed_contract_has_chrome() {
    let bytes = std::fs::read(common::repo_root().join("examples/published/contract.K2F")).unwrap();
    let app = AppState::open(&bytes).unwrap();
    assert_eq!(app.form_fields().len(), 4);
    assert!(app.has_form_fields());
}

#[test]
fn form_fill_save_relock_writes_value_and_keeps_sibling() {
    let bytes = packed_form();
    let app = AppState::open(&bytes).unwrap();
    let sign_a = app.doc().boxes_for("root.sign")[0].clone();
    assert_eq!(app.form_fields().len(), 2);
    assert!(app.form_fields().iter().all(|f| f.value.is_empty()));

    let mut session = Session::new(app).unwrap();
    session.toggle_fill();
    assert!(session.is_filling());
    assert!(session.fill_click_id("root.name"));
    session.fill_insert("Alice");
    assert!(session.fill_click_id("root.agree"));
    let next = session.save_fill().unwrap();
    assert!(!session.is_filling());

    let doc = OpenedDocument::open(&next).unwrap();
    let fields = doc.form_fields();
    let name = fields.iter().find(|f| f.id == "root.name").unwrap();
    let agree = fields.iter().find(|f| f.id == "root.agree").unwrap();
    assert_eq!(name.value, "Alice");
    assert_eq!(agree.value, "true");
    let sign_b = doc.boxes_for("root.sign")[0].clone();
    assert_eq!(
        (sign_a.x, sign_a.y, sign_a.width, sign_a.height),
        (sign_b.x, sign_b.y, sign_b.width, sign_b.height)
    );
    assert_ne!(
        OpenedDocument::open(&bytes).unwrap().content_hash(),
        doc.content_hash()
    );
}

#[test]
fn form_fill_empty_save_preserves_package() {
    let bytes = packed_form();
    let before = OpenedDocument::open(&bytes).unwrap();
    let hash_before = before.content_hash();
    let mut session = Session::new(AppState::open(&bytes).unwrap()).unwrap();
    session.toggle_fill();
    let next = session.save_fill().unwrap();
    let after = OpenedDocument::open(&next).unwrap();
    assert_eq!(
        after.content_hash(),
        hash_before,
        "Fill → Save with no edits must not relock"
    );
}

#[test]
fn form_fill_unsaved_does_not_change_package() {
    let bytes = packed_form();
    let app = AppState::open(&bytes).unwrap();
    let mut session = Session::new(app).unwrap();
    session.toggle_fill();
    session.fill_click_id("root.name");
    session.fill_insert("Bob");
    assert_eq!(
        session.app().unwrap().form_fields()[0].value,
        "",
        "dirty map must not write the package before Save"
    );
}

#[test]
fn form_fill_application_example_relock() {
    let bytes = Editor::open_dir(&common::repo_root().join("examples/form_application"))
        .unwrap()
        .save_bytes()
        .unwrap();
    let mut session = Session::new(AppState::open(&bytes).unwrap()).unwrap();
    session.toggle_fill();
    assert!(session.fill_click_id("app.name"));
    session.fill_insert("Alice");
    let next = session.save_fill().unwrap();
    let doc = OpenedDocument::open(&next).unwrap();
    let name = doc
        .form_fields()
        .into_iter()
        .find(|f| f.id == "app.name")
        .expect("app.name");
    assert_eq!(name.value, "Alice");
}
