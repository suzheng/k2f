use crate::{
    nfc, node_text, replace_node_text, validate_semantic_tree, BreakInside, FixedSizeHint,
    FormFieldKind, FormFieldSpec, K2FError, LayoutHint, Modifier, NodeContent, NodeEditError, Pt,
    SemanticNode,
};

fn field_node(id: &str, spec: FormFieldSpec) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "form_field".to_string(),
        variant: Some("underline".to_string()),
        break_inside: BreakInside::Avoid,
        content: NodeContent::FormField(spec),
        ..Default::default()
    }
}

fn empty_text_field(id: &str) -> SemanticNode {
    field_node(
        id,
        FormFieldSpec {
            kind: FormFieldKind::Text,
            value: String::new(),
            placeholder: Some("Full legal name".to_string()),
            width: None,
            height: None,
            lines: Some(1),
            max_length: None,
            required: false,
        },
    )
}

#[test]
fn accepts_valid_empty_text_field() {
    validate_semantic_tree(&empty_text_field("app.applicant.full_name")).unwrap();
}

#[test]
fn deserializes_plan_json_and_validates() {
    let json = r#"{
        "id": "app.applicant.full_name",
        "role": "form_field",
        "variant": "underline",
        "break_inside": "avoid",
        "content": {
            "type": "form_field",
            "value": {
                "kind": "text",
                "value": "",
                "placeholder": "Full legal name",
                "width": 220000,
                "lines": 1
            }
        }
    }"#;
    let node: SemanticNode = serde_json::from_str(json).unwrap();
    validate_semantic_tree(&node).unwrap();
    match &node.content {
        NodeContent::FormField(spec) => {
            assert_eq!(spec.kind, FormFieldKind::Text);
            assert_eq!(spec.value, "");
            assert_eq!(spec.width, Some(Pt(220000)));
            assert_eq!(spec.placeholder.as_deref(), Some("Full legal name"));
        }
        other => panic!("expected form_field, got {}", other.type_name()),
    }
}

#[test]
fn rejects_body_role_with_form_field_content() {
    let node = SemanticNode {
        id: "app.name".to_string(),
        role: "body".to_string(),
        content: NodeContent::FormField(FormFieldSpec {
            kind: FormFieldKind::Text,
            value: String::new(),
            placeholder: None,
            width: None,
            height: None,
            lines: None,
            max_length: None,
            required: false,
        }),
        ..Default::default()
    };
    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(
        err,
        K2FError::FormFieldContentRequiresFormFieldRole { .. }
    ));
    assert!(err.to_string().contains("FORM_FIELD_CONTENT"));
}

#[test]
fn rejects_form_field_role_with_text_content() {
    let node = SemanticNode {
        id: "app.name".to_string(),
        role: "form_field".to_string(),
        content: NodeContent::Text("____".to_string()),
        ..Default::default()
    };
    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(
        err,
        K2FError::FormFieldRoleRequiresFormFieldContent { .. }
    ));
    assert!(err.to_string().contains("FORM_FIELD_ROLE"));
}

#[test]
fn rejects_form_field_with_layout() {
    let mut node = empty_text_field("app.name");
    node.layout = Some(LayoutHint::Overlay {
        size: FixedSizeHint::default(),
    });
    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(err, K2FError::FormFieldLayoutNotAllowed { .. }));
    assert!(err.to_string().contains("FORM_FIELD_LAYOUT"));
}

#[test]
fn rejects_form_field_with_modifiers() {
    let mut node = empty_text_field("app.name");
    node.modifiers.push(Modifier {
        range: [0, 1],
        mod_type: "emphasis".to_string(),
        intent: "strong".to_string(),
    });
    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(err, K2FError::FormFieldModifiersNotAllowed { .. }));
    assert!(err.to_string().contains("FORM_FIELD_MODIFIERS"));
}

#[test]
fn checkbox_value_must_be_empty_or_true() {
    let yes = field_node(
        "app.read",
        FormFieldSpec {
            kind: FormFieldKind::Checkbox,
            value: "yes".to_string(),
            placeholder: None,
            width: None,
            height: None,
            lines: None,
            max_length: None,
            required: false,
        },
    );
    let err = validate_semantic_tree(&yes).unwrap_err();
    assert!(matches!(err, K2FError::FormFieldCheckboxValue { .. }));
    assert!(err.to_string().contains("FORM_FIELD_CHECKBOX_VALUE"));

    for value in ["", "true"] {
        let node = field_node(
            "app.read",
            FormFieldSpec {
                kind: FormFieldKind::Checkbox,
                value: value.to_string(),
                placeholder: None,
                width: None,
                height: None,
                lines: None,
                max_length: None,
                required: false,
            },
        );
        validate_semantic_tree(&node).unwrap();
    }
}

#[test]
fn rejects_zero_lines() {
    let node = field_node(
        "app.addr",
        FormFieldSpec {
            kind: FormFieldKind::Multiline,
            value: String::new(),
            placeholder: None,
            width: None,
            height: None,
            lines: Some(0),
            max_length: None,
            required: false,
        },
    );
    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(err, K2FError::FormFieldLines { .. }));
    assert!(err.to_string().contains("FORM_FIELD_LINES"));
}

#[test]
fn rejects_zero_width() {
    let node = field_node(
        "app.name",
        FormFieldSpec {
            kind: FormFieldKind::Text,
            value: String::new(),
            placeholder: None,
            width: Some(Pt(0)),
            height: None,
            lines: Some(1),
            max_length: None,
            required: false,
        },
    );
    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(err, K2FError::FormFieldSize { .. }));
    assert!(err.to_string().contains("FORM_FIELD_SIZE"));
}

#[test]
fn node_text_reads_value_and_replace_writes_nfc() {
    let mut root = empty_text_field("app.name");
    assert_eq!(node_text(&root), Some(""));
    replace_node_text(&mut root, &mut [], "app.name", "e\u{0301}").unwrap();
    assert_eq!(node_text(&root), Some(nfc("e\u{0301}").as_str()));
    match &root.content {
        NodeContent::FormField(spec) => assert_eq!(spec.value, "\u{00e9}"),
        _ => panic!("expected form_field"),
    }
}

#[test]
fn replace_rejects_over_max_length() {
    let mut root = field_node(
        "app.name",
        FormFieldSpec {
            kind: FormFieldKind::Text,
            value: String::new(),
            placeholder: None,
            width: None,
            height: None,
            lines: Some(1),
            max_length: Some(3),
            required: false,
        },
    );
    let err = replace_node_text(&mut root, &mut [], "app.name", "abcd").unwrap_err();
    assert!(matches!(
        err,
        NodeEditError::MaxLengthExceeded { max: 3, got: 4, .. }
    ));
    assert_eq!(node_text(&root), Some(""));
}

#[test]
fn required_empty_value_is_allowed() {
    let node = field_node(
        "app.name",
        FormFieldSpec {
            kind: FormFieldKind::Text,
            value: String::new(),
            placeholder: None,
            width: None,
            height: None,
            lines: Some(1),
            max_length: None,
            required: true,
        },
    );
    validate_semantic_tree(&node).unwrap();
}

#[test]
fn omitted_break_inside_is_allowed() {
    let mut node = empty_text_field("app.name");
    node.break_inside = BreakInside::Auto;
    validate_semantic_tree(&node).unwrap();
}

#[test]
fn rejects_zero_height() {
    let node = field_node(
        "app.name",
        FormFieldSpec {
            kind: FormFieldKind::Text,
            value: String::new(),
            placeholder: None,
            width: Some(Pt(220000)),
            height: Some(Pt(0)),
            lines: Some(1),
            max_length: None,
            required: false,
        },
    );
    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(err, K2FError::FormFieldSize { .. }));
    assert!(err.to_string().contains("FORM_FIELD_SIZE"));
}

#[test]
fn rejects_value_longer_than_max_length() {
    let node = field_node(
        "app.name",
        FormFieldSpec {
            kind: FormFieldKind::Text,
            value: "abcd".to_string(),
            placeholder: None,
            width: None,
            height: None,
            lines: Some(1),
            max_length: Some(3),
            required: false,
        },
    );
    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(
        err,
        K2FError::FormFieldMaxLength { max: 3, got: 4, .. }
    ));
    assert!(err.to_string().contains("FORM_FIELD_MAX_LENGTH"));
}

#[test]
fn replace_rejects_invalid_checkbox_value() {
    let mut root = field_node(
        "app.read",
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
    );
    let err = replace_node_text(&mut root, &mut [], "app.read", "yes").unwrap_err();
    assert!(matches!(err, NodeEditError::InvalidCheckboxValue(_)));
    assert_eq!(node_text(&root), Some(""));
    replace_node_text(&mut root, &mut [], "app.read", "true").unwrap();
    assert_eq!(node_text(&root), Some("true"));
}

#[test]
fn deserializes_multiline_and_checkbox_plan_json() {
    let multiline: SemanticNode = serde_json::from_str(
        r#"{
            "id": "app.address",
            "role": "form_field",
            "variant": "box",
            "break_inside": "avoid",
            "content": {
                "type": "form_field",
                "value": {
                    "kind": "multiline",
                    "value": "",
                    "placeholder": "Street address",
                    "lines": 4
                }
            }
        }"#,
    )
    .unwrap();
    validate_semantic_tree(&multiline).unwrap();
    match &multiline.content {
        NodeContent::FormField(spec) => {
            assert_eq!(spec.kind, FormFieldKind::Multiline);
            assert_eq!(spec.lines, Some(4));
        }
        other => panic!("expected form_field, got {}", other.type_name()),
    }

    let checkbox: SemanticNode = serde_json::from_str(
        r#"{
            "id": "app.read",
            "role": "form_field",
            "variant": "checkbox",
            "break_inside": "avoid",
            "content": {
                "type": "form_field",
                "value": { "kind": "checkbox", "value": "" }
            }
        }"#,
    )
    .unwrap();
    validate_semantic_tree(&checkbox).unwrap();
    match &checkbox.content {
        NodeContent::FormField(spec) => assert_eq!(spec.kind, FormFieldKind::Checkbox),
        other => panic!("expected form_field, got {}", other.type_name()),
    }
}

#[test]
fn plan_section7_document_deserializes_and_validates() {
    let json = r#"{
        "id": "doc",
        "role": "document",
        "layout": { "type": "stack", "direction": "vertical", "gap": 8000 },
        "content": {
            "type": "container",
            "value": {
                "children": [
                    {
                        "id": "doc.name_label",
                        "role": "body",
                        "content": { "type": "text", "value": "Name" }
                    },
                    {
                        "id": "doc.name",
                        "role": "form_field",
                        "variant": "underline",
                        "break_inside": "avoid",
                        "content": {
                            "type": "form_field",
                            "value": { "kind": "text", "value": "", "placeholder": "Full name" }
                        }
                    }
                ]
            }
        }
    }"#;
    let node: SemanticNode = serde_json::from_str(json).unwrap();
    validate_semantic_tree(&node).unwrap();
    let NodeContent::Container { children } = &node.content else {
        panic!("expected container");
    };
    assert_eq!(node_text(&children[1]), Some(""));
}

#[test]
fn for_each_form_field_walks_in_document_order() {
    let node: SemanticNode = serde_json::from_str(
        r#"{
        "id": "doc",
        "role": "document",
        "content": {
            "type": "container",
            "value": {
                "children": [
                    {
                        "id": "doc.name",
                        "role": "form_field",
                        "break_inside": "avoid",
                        "content": {
                            "type": "form_field",
                            "value": { "kind": "text", "value": "" }
                        }
                    },
                    {
                        "id": "doc.body",
                        "role": "body",
                        "content": { "type": "text", "value": "after" }
                    },
                    {
                        "id": "doc.agree",
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
    }"#,
    )
    .unwrap();
    let mut ids = Vec::new();
    crate::for_each_form_field(&node, &mut |n, spec| {
        ids.push((n.id.clone(), spec.kind.as_str().to_string()));
    });
    assert_eq!(
        ids,
        vec![
            ("doc.name".into(), "text".into()),
            ("doc.agree".into(), "checkbox".into()),
        ]
    );
}
