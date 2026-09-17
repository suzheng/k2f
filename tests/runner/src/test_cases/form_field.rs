use k2f_core::{
    BreakInside, CanvasMode, FormFieldKind, FormFieldSpec, Manifest, NodeContent, SemanticNode,
    StackDirection,
};

use super::super::node_builders::{default_page_config, stack_container, text_node_role};

fn field(id: &str, spec: FormFieldSpec, variant: &str) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "form_field".to_string(),
        variant: Some(variant.to_string()),
        break_inside: BreakInside::Avoid,
        content: NodeContent::FormField(spec),
        ..Default::default()
    }
}

fn empty_text() -> FormFieldSpec {
    FormFieldSpec {
        kind: FormFieldKind::Text,
        value: String::new(),
        placeholder: Some("Name".into()),
        width: None,
        height: None,
        lines: None,
        max_length: None,
        required: false,
    }
}

fn empty_multiline(id_lines: u32) -> FormFieldSpec {
    FormFieldSpec {
        kind: FormFieldKind::Multiline,
        value: String::new(),
        placeholder: Some("Address".into()),
        width: None,
        height: None,
        lines: Some(id_lines),
        max_length: None,
        required: false,
    }
}

fn empty_checkbox() -> FormFieldSpec {
    FormFieldSpec {
        kind: FormFieldKind::Checkbox,
        value: String::new(),
        placeholder: None,
        width: None,
        height: None,
        lines: None,
        max_length: None,
        required: false,
    }
}

/// One-page empty form. Filling is not part of the golden (values change appearance).
pub fn form_field_empty_page() -> Manifest {
    Manifest {
        title: "Form field empty page".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: stack_container(
            "root",
            "document",
            StackDirection::Vertical,
            12000,
            vec![
                text_node_role("title", "body", "Application"),
                text_node_role("name_label", "body", "Name"),
                field("name", empty_text(), "underline"),
                text_node_role("addr_label", "body", "Address"),
                field("address", empty_multiline(3), "box"),
                field("agree", empty_checkbox(), "checkbox"),
                text_node_role("sign", "body", "Signature"),
            ],
        ),
        running_blocks: vec![],
    }
}
