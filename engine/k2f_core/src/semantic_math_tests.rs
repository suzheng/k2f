use crate::{
    validate_semantic_tree, BreakInside, FixedSizeHint, K2FError, LayoutHint, Modifier,
    NodeContent, SemanticNode,
};

fn base_math_node(id: &str, tex: &str) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "math".to_string(),
        break_inside: BreakInside::Avoid,
        content: NodeContent::Math(tex.to_string()),
        ..Default::default()
    }
}

#[test]
fn accepts_valid_math_node() {
    validate_semantic_tree(&base_math_node("eq.1", "E=mc^2")).unwrap();
}

#[test]
fn rejects_role_math_with_non_math_content() {
    let node = SemanticNode {
        id: "eq".to_string(),
        role: "math".to_string(),
        content: NodeContent::Text("x".to_string()),
        ..Default::default()
    };
    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(err, K2FError::MathRoleRequiresMathContent { .. }));
}

#[test]
fn rejects_math_content_with_non_math_role() {
    let node = SemanticNode {
        id: "eq".to_string(),
        role: "body".to_string(),
        content: NodeContent::Math("x".to_string()),
        ..Default::default()
    };
    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(err, K2FError::MathContentRequiresMathRole { .. }));
}

#[test]
fn rejects_math_with_layout_hint() {
    let mut node = base_math_node("eq", "x");
    node.layout = Some(LayoutHint::Overlay {
        size: FixedSizeHint::default(),
    });
    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(err, K2FError::MathLayoutNotAllowed { .. }));
}

#[test]
fn rejects_math_with_modifiers() {
    let mut node = base_math_node("eq", "x");
    node.modifiers.push(Modifier {
        range: [0, 1],
        mod_type: "emphasis".to_string(),
        intent: "strong".to_string(),
    });
    let err = validate_semantic_tree(&node).unwrap_err();
    assert!(matches!(err, K2FError::MathModifiersNotAllowed { .. }));
}

#[test]
fn rejects_empty_math() {
    let err = validate_semantic_tree(&base_math_node("eq", "   ")).unwrap_err();
    assert!(matches!(err, K2FError::MathEmpty { .. }));
}

#[test]
fn fill_rect_serde_roundtrip() {
    use crate::{FillRect, GeometryNode, Pt};
    let geo = GeometryNode {
        id: "eq".into(),
        x: Pt(0),
        y: Pt(0),
        width: Pt(1000),
        height: Pt(500),
        glyphs: vec![],
        text_runs: vec![],
        fill_rects: vec![FillRect {
            x: Pt(10),
            y: Pt(20),
            width: Pt(100),
            height: Pt(2),
        }],
        children: vec![],
    };
    let json = serde_json::to_string(&geo).unwrap();
    let back: GeometryNode = serde_json::from_str(&json).unwrap();
    assert_eq!(back.fill_rects.len(), 1);
    assert_eq!(back.fill_rects[0].width, Pt(100));
}

#[test]
fn geometry_without_fill_rects_deserializes() {
    use crate::GeometryNode;
    let json = r#"{"id":"n","x":0,"y":0,"width":1,"height":1,"glyphs":[],"children":[]}"#;
    let geo: GeometryNode = serde_json::from_str(json).unwrap();
    assert!(geo.fill_rects.is_empty());
}
