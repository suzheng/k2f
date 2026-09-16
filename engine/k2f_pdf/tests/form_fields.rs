//! Step 6: default PDF export writes AcroForm widgets; `--flatten` paints lock glyphs.

mod common;

use k2f_core::{BreakInside, FormFieldKind, FormFieldSpec, NodeContent, SemanticNode};
use k2f_paint::OpenedDocument;
use k2f_pdf::{export_opened, PdfExportOptions, PdfScale};
use k2f_sdk::{export_pdf, export_pdf_with, Editor};

fn opts() -> PdfExportOptions {
    PdfExportOptions::new(PdfScale::DEFAULT)
}

fn pack_dir(rel: &str) -> OpenedDocument {
    let mut ed = Editor::open_dir(&common::repo_root().join(rel)).unwrap();
    let bytes = ed.save_bytes().unwrap();
    OpenedDocument::open(&bytes).unwrap()
}

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

fn blank_form() -> Editor {
    let mut ed = Editor::open_dir(&common::repo_root().join("templates/blank")).unwrap();
    let n = |id: &str, role: &str, text: &str| SemanticNode {
        id: id.to_string(),
        role: role.to_string(),
        content: NodeContent::Text(text.into()),
        ..Default::default()
    };
    let json = |node: &SemanticNode| serde_json::to_string(node).unwrap();
    ed.insert_node("root", 0, &json(&n("root.title", "h1", "Application")))
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
                required: true,
            },
            "underline",
        )),
    )
    .unwrap();
    ed.insert_node(
        "root",
        2,
        &json(&form_field(
            "root.address",
            FormFieldSpec {
                kind: FormFieldKind::Multiline,
                value: String::new(),
                placeholder: Some("Street address".into()),
                width: None,
                height: None,
                lines: Some(3),
                max_length: None,
                required: false,
            },
            "box",
        )),
    )
    .unwrap();
    ed.insert_node("root", 3, &json(&n("root.sign", "body", "Signature")))
        .unwrap();
    ed.insert_node(
        "root",
        4,
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
    ed
}

fn save_open(ed: &mut Editor) -> OpenedDocument {
    OpenedDocument::open(&ed.save_bytes().unwrap()).unwrap()
}

#[test]
fn form_field_default_export_writes_acroform_matching_query() {
    let doc = pack_dir("examples/contract");
    let fields = doc.form_fields();
    assert!(
        !fields.is_empty(),
        "compiled contract must have form fields"
    );
    let pdf = export_opened(&doc, opts()).unwrap();
    let dicts = common::acroform::field_dicts(&pdf);
    assert_eq!(dicts.len(), fields.len());
    let tus = common::acroform::field_alt_names(&pdf);
    for f in &fields {
        assert!(
            tus.iter().any(|tu| tu == &f.id),
            "missing /TU for {} in {tus:?}",
            f.id
        );
    }
}

#[test]
fn form_field_filled_value_is_widget_v() {
    let mut ed = blank_form();
    ed.replace_text("root.name", "Alice").unwrap();
    ed.replace_text("root.agree", "true").unwrap();
    let doc = save_open(&mut ed);
    let pdf = export_opened(&doc, opts()).unwrap();
    let values = common::acroform::field_values(&pdf);
    assert!(
        values.iter().any(|v| v == "Alice"),
        "text /V must be Alice, got {values:?}"
    );
    assert!(
        values.iter().any(|v| v == "Yes"),
        "checked checkbox /V must be Yes, got {values:?}"
    );
}

#[test]
fn form_field_flatten_omits_acroform_and_paints_glyphs() {
    let mut ed = blank_form();
    ed.replace_text("root.name", "Alice").unwrap();
    let doc = save_open(&mut ed);
    let fillable = export_opened(&doc, opts()).unwrap();
    let flat = export_opened(&doc, opts().with_flatten()).unwrap();
    assert!(common::acroform::has_acroform(&fillable));
    assert!(
        !common::acroform::has_acroform(&flat),
        "flatten must not write /AcroForm"
    );
    assert!(
        common::acroform::glyph_note_count(&flat) > common::acroform::glyph_note_count(&fillable),
        "flatten must paint field DrawText that fillable export skips"
    );
    let extracted = common::extract::extract_pdf_text(&flat);
    assert!(
        extracted.contains("Alice"),
        "flattened content must include Alice, got {extracted:?}"
    );
}

#[test]
fn form_field_invoice_default_export_has_no_acroform() {
    let pdf = export_opened(&common::invoice(), opts()).unwrap();
    assert!(!common::acroform::has_acroform(&pdf));
    assert!(common::acroform::field_dicts(&pdf).is_empty());
}

#[test]
fn form_field_trust_pack_without_flatten_is_fillable_exclusive() {
    let doc = pack_dir("examples/contract");
    let err = export_opened(&doc, opts().with_trust_pack()).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("FILLABLE_EXCLUSIVE"), "got {msg}");
}

#[test]
fn form_field_trust_pack_flatten_exports() {
    let doc = pack_dir("examples/contract");
    let pdf = export_opened(&doc, opts().with_trust_pack().with_flatten()).unwrap();
    assert!(!common::acroform::has_acroform(&pdf));
    let parsed = lopdf::Document::load_mem(&pdf).unwrap();
    assert_eq!(
        parsed.get_pages().len(),
        doc.lock().unwrap().geometry.pages.len() + 1,
        "flatten + trust-pack still appends the verify page"
    );
}

#[test]
fn form_field_sdk_export_pdf_matches_default_fillable() {
    let mut ed = blank_form();
    ed.replace_text("root.name", "Alice").unwrap();
    let bytes = ed.save_bytes().unwrap();
    let via_sdk = export_pdf(&bytes).unwrap();
    let via_opts = export_pdf_with(&bytes, opts()).unwrap();
    assert_eq!(via_sdk, via_opts);
    assert!(common::acroform::has_acroform(&via_sdk));
    let flat = export_pdf_with(&bytes, opts().with_flatten()).unwrap();
    assert!(!common::acroform::has_acroform(&flat));
}
