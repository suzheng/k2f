use crate::{document_to_markdown, MarkdownEmitOptions};
use k2f_core::SemanticNode;

fn plan_doc(field_json: &str) -> SemanticNode {
    serde_json::from_str(&format!(
        r#"{{
            "id": "doc",
            "role": "document",
            "layout": {{ "type": "stack", "direction": "vertical", "gap": 8000 }},
            "content": {{
                "type": "container",
                "value": {{
                    "children": [
                        {{
                            "id": "doc.name_label",
                            "role": "body",
                            "content": {{ "type": "text", "value": "Name" }}
                        }},
                        {field_json}
                    ]
                }}
            }}
        }}"#
    ))
    .unwrap()
}

#[test]
fn hints_emit_comment_and_value_from_plan_json() {
    let root = plan_doc(
        r#"{
            "id": "doc.name",
            "role": "form_field",
            "variant": "underline",
            "break_inside": "avoid",
            "content": {
                "type": "form_field",
                "value": { "kind": "text", "value": "Alice", "placeholder": "Full name" }
            }
        }"#,
    );
    let md = document_to_markdown(&root, &[], MarkdownEmitOptions { hints: true });
    assert!(
        md.contains("<!-- k2f: form_field kind=text id=doc.name -->"),
        "got {md:?}"
    );
    assert!(md.contains("Alice"), "got {md:?}");
    assert!(
        !md.contains("Full name"),
        "placeholder must not be emitted: {md:?}"
    );
}

#[test]
fn clipboard_emits_only_value() {
    let root = plan_doc(
        r#"{
            "id": "doc.name",
            "role": "form_field",
            "variant": "underline",
            "break_inside": "avoid",
            "content": {
                "type": "form_field",
                "value": { "kind": "text", "value": "Alice" }
            }
        }"#,
    );
    let md = document_to_markdown(&root, &[], MarkdownEmitOptions::clipboard());
    assert!(!md.contains("k2f:"), "got {md:?}");
    assert!(md.contains("Alice"), "got {md:?}");
}

#[test]
fn checkbox_plain_markers() {
    let unchecked = plan_doc(
        r#"{
            "id": "doc.read",
            "role": "form_field",
            "variant": "checkbox",
            "break_inside": "avoid",
            "content": { "type": "form_field", "value": { "kind": "checkbox", "value": "" } }
        }"#,
    );
    let checked = plan_doc(
        r#"{
            "id": "doc.read",
            "role": "form_field",
            "variant": "checkbox",
            "break_inside": "avoid",
            "content": { "type": "form_field", "value": { "kind": "checkbox", "value": "true" } }
        }"#,
    );
    let empty = document_to_markdown(&unchecked, &[], MarkdownEmitOptions::clipboard());
    let filled = document_to_markdown(&checked, &[], MarkdownEmitOptions::clipboard());
    assert!(empty.contains("[ ]"), "got {empty:?}");
    assert!(filled.contains("[x]"), "got {filled:?}");
}
