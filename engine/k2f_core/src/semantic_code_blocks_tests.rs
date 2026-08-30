use crate::{
    validate_semantic_tree, CodeBlockValue, FixedSizeHint, K2FError, LayoutHint, Modifier,
    NodeContent, SemanticNode,
};

fn base_code_block_node(id: &str, value: CodeBlockValue) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "code_block".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::CodeBlock(value),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    }
}

#[test]
fn rejects_role_code_block_with_non_code_block_content() {
    let node = SemanticNode {
        id: "cb".to_string(),
        role: "code_block".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Text("x".to_string()),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(
        err,
        K2FError::CodeBlockRoleRequiresCodeBlockContent { .. }
    ));
}

#[test]
fn rejects_code_block_content_with_non_code_block_role() {
    let node = SemanticNode {
        id: "cb".to_string(),
        role: "body".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::CodeBlock(CodeBlockValue::Text("x".to_string())),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(
        err,
        K2FError::CodeBlockContentRequiresCodeBlockRole { .. }
    ));
}

#[test]
fn rejects_code_block_preserve_whitespace_false() {
    let mut node = base_code_block_node("cb", CodeBlockValue::Text("x".to_string()));
    node.preserve_whitespace = Some(false);

    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(
        err,
        K2FError::CodeBlockPreserveWhitespaceMustBeTrue { .. }
    ));
}

#[test]
fn rejects_code_block_with_layout_hint() {
    let mut node = base_code_block_node("cb", CodeBlockValue::Text("x".to_string()));
    node.layout = Some(LayoutHint::Overlay {
        size: FixedSizeHint::default(),
    });

    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(err, K2FError::CodeBlockLayoutNotAllowed { .. }));
}

#[test]
fn rejects_code_block_with_disallowed_modifier_type() {
    let mut node = base_code_block_node("cb", CodeBlockValue::Text("x".to_string()));
    node.modifiers.push(Modifier {
        range: [0, 1],
        mod_type: "bold".to_string(),
        intent: "on".to_string(),
    });

    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(
        err,
        K2FError::CodeBlockDisallowedModifierType { .. }
    ));
}

#[test]
fn validates_modifier_ranges_against_canonicalized_code_text_lines_variant() {
    // Canonical text is "abc\ndef" (7 bytes).
    let node = base_code_block_node(
        "cb",
        CodeBlockValue::Lines(vec!["abc".to_string(), "def".to_string()]),
    );

    // Highlight "abc" (0..3) in the canonical text.
    let mut ok = node.clone();
    ok.modifiers.push(Modifier {
        range: [0, 3],
        mod_type: "syntax_highlight".to_string(),
        intent: "keyword".to_string(),
    });
    validate_semantic_tree(&ok).unwrap();

    // Invalid: end goes past canonical length (7).
    let mut bad = node;
    bad.modifiers.push(Modifier {
        range: [0, 8],
        mod_type: "syntax_highlight".to_string(),
        intent: "keyword".to_string(),
    });
    let err = validate_semantic_tree(&bad).unwrap_err();
    assert!(matches!(err, K2FError::InvalidModifierRange { .. }));
}
