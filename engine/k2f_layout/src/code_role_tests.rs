use crate::{measure_node, LayoutContext, Size, SizeConstraint, Theme};
use k2f_core::{NodeContent, Pt, SemanticNode};

fn text_node(role: &str, text: &str) -> SemanticNode {
    SemanticNode {
        id: format!("n.{role}"),
        role: role.into(),
        content: NodeContent::Text(text.into()),
        ..Default::default()
    }
}

#[test]
fn code_role_text_does_not_soft_wrap() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);
    let text = "this_is_a_single_very_long_identifier_without_spaces";
    let constraint = SizeConstraint::new(Size::ZERO, Size::new(Pt(20_000), Pt(i128::MAX)));
    let body = measure_node(&text_node("body", text), constraint, &ctx).unwrap();
    let code = measure_node(&text_node("code", text), constraint, &ctx).unwrap();
    assert!(
        code.height < body.height,
        "code role should stay one line, body wraps: code={:?} body={:?}",
        code.height,
        body.height
    );
}
