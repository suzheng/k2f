use crate::style::StylePatch;
use crate::{LayoutContext, Size, SizeConstraint, Theme};
use k2f_core::{Modifier, Pt};
use std::collections::HashMap;

use super::metrics::measure_text_run_width;

fn emphasis_color_theme() -> Theme {
    let mut theme = Theme::default();
    let mut by_intent = HashMap::new();
    by_intent.insert(
        "red".to_string(),
        StylePatch {
            color: Some("red".to_string()),
            ..StylePatch::default()
        },
    );
    theme
        .modifiers
        .styles
        .insert("emphasis".to_string(), by_intent);
    theme
}

#[test]
fn hard_newlines_force_line_breaks_even_with_infinite_width() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let constraint = SizeConstraint::infinite();
    let modifiers: Vec<Modifier> = vec![];

    let layout =
        crate::text_layout::layout_text("Hello\nworld", "body", None, &modifiers, constraint, &ctx)
            .unwrap();
    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].runs.len(), 1);
    assert_eq!(layout.lines[0].runs[0].text, "Hello");
    assert_eq!(layout.lines[1].runs.len(), 1);
    assert_eq!(layout.lines[1].runs[0].text, "world");
}

#[test]
fn wraps_on_word_boundaries_with_fixed_width() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let style = crate::style::resolve_base_style("body", &theme);
    let hello_w = measure_text_run_width("Hello", &style, &ctx).unwrap();
    let space_w = measure_text_run_width(" ", &style, &ctx).unwrap();

    // Allow "Hello " but not "Hello world".
    let max_w = hello_w + space_w + Pt(1);
    let constraint = SizeConstraint::new(Size::ZERO, Size::new(max_w, Pt(i128::MAX)));

    let modifiers: Vec<Modifier> = vec![];
    let layout =
        crate::text_layout::layout_text("Hello world", "body", None, &modifiers, constraint, &ctx)
            .unwrap();
    assert_eq!(layout.lines.len(), 2);
    assert_eq!(
        layout.lines[0]
            .runs
            .iter()
            .map(|r| r.text.as_str())
            .collect::<String>(),
        "Hello"
    );
    assert_eq!(
        layout.lines[1]
            .runs
            .iter()
            .map(|r| r.text.as_str())
            .collect::<String>(),
        "world"
    );
}

#[test]
fn modifier_runs_are_preserved_across_line_breaks() {
    let fonts = crate::test_utils::test_fonts();
    let theme = emphasis_color_theme();
    let ctx = LayoutContext::new(&fonts, &theme);

    let style = crate::style::resolve_base_style("body", &theme);
    let hello_w = measure_text_run_width("Hello", &style, &ctx).unwrap();
    let space_w = measure_text_run_width(" ", &style, &ctx).unwrap();
    let max_w = hello_w + space_w + Pt(1);

    let constraint = SizeConstraint::new(Size::ZERO, Size::new(max_w, Pt(i128::MAX)));
    let modifiers = vec![Modifier {
        range: [6, 11], // "world" in "Hello world"
        mod_type: "emphasis".to_string(),
        intent: "red".to_string(),
    }];

    let layout =
        crate::text_layout::layout_text("Hello world", "body", None, &modifiers, constraint, &ctx)
            .unwrap();
    assert_eq!(layout.lines.len(), 2);

    let line1_text = layout.lines[0]
        .runs
        .iter()
        .map(|r| r.text.as_str())
        .collect::<String>();
    let line2_text = layout.lines[1]
        .runs
        .iter()
        .map(|r| r.text.as_str())
        .collect::<String>();
    assert_eq!(line1_text, "Hello");
    assert_eq!(line2_text, "world");

    // Style should be applied only on "world".
    assert_eq!(layout.lines[0].runs.len(), 1);
    assert_eq!(layout.lines[0].runs[0].style.color, "black");
    assert_eq!(layout.lines[1].runs.len(), 1);
    assert_eq!(layout.lines[1].runs[0].style.color, "red");
}

fn tracking_body_theme(letter_spacing_pt: i128) -> Theme {
    use crate::theme::RoleStyle;
    let mut theme = Theme::default();
    theme.roles.insert(
        "body".to_string(),
        RoleStyle {
            font_family: "default".to_string(),
            font_size: Pt(12000),
            line_height_mult: 1200,
            color: "black".to_string(),
            letter_spacing_pt: Pt(letter_spacing_pt),
            ..RoleStyle::default()
        },
    );
    theme
}

#[test]
fn line_width_includes_tracking_between_wrapped_fragments() {
    let fonts = crate::test_utils::test_fonts();
    let theme = tracking_body_theme(2000);
    let ctx = LayoutContext::new(&fonts, &theme);
    let style = crate::style::resolve_base_style("body", &theme);

    let constraint = SizeConstraint::infinite();
    let modifiers: Vec<Modifier> = vec![];
    let layout =
        crate::text_layout::layout_text("A B", "body", None, &modifiers, constraint, &ctx)
            .unwrap();
    assert_eq!(layout.lines.len(), 1);
    let shaped = measure_text_run_width("A B", &style, &ctx).unwrap();
    assert_eq!(
        layout.lines[0].width, shaped,
        "center/end alignment uses line.width; it must match a single shaped run"
    );
    let a = measure_text_run_width("A", &style, &ctx).unwrap();
    let sp = measure_text_run_width(" ", &style, &ctx).unwrap();
    let b = measure_text_run_width("B", &style, &ctx).unwrap();
    assert_eq!(layout.lines[0].width, a + sp + b + style.letter_spacing * 2);
}

#[test]
fn wrapping_is_deterministic_across_runs() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let style = crate::style::resolve_base_style("body", &theme);
    let hello_w = measure_text_run_width("Hello", &style, &ctx).unwrap();
    let space_w = measure_text_run_width(" ", &style, &ctx).unwrap();
    let max_w = hello_w + space_w + Pt(1);
    let constraint = SizeConstraint::new(Size::ZERO, Size::new(max_w, Pt(i128::MAX)));

    let modifiers: Vec<Modifier> = vec![];
    let reference =
        crate::text_layout::layout_text("Hello world", "body", None, &modifiers, constraint, &ctx)
            .unwrap();
    for _ in 0..25 {
        let current = crate::text_layout::layout_text(
            "Hello world",
            "body",
            None,
            &modifiers,
            constraint,
            &ctx,
        )
        .unwrap();
        assert_eq!(reference, current);
    }
}
