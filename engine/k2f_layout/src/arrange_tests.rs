use crate::visual_primitives::EdgeInsetsPt;
use crate::Theme;
use crate::ThemeDecoration;
use crate::{arrange_node, measure_node, LayoutContext, Point, Size, SizeConstraint};
use k2f_core::{
    Align, FixedSizeHint, JustifyContent, LayoutHint, NodeContent, Pt, SemanticNode, StackDirection,
};
use std::collections::BTreeMap;

// Helper to create a dummy text node
fn make_text(id: &str, content: &str) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "body".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Text(content.to_string()),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    }
}

fn make_image(id: &str) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "body".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        // Deterministic leaf sizing: tests provide explicit size hints.
        content: NodeContent::Image {
            src: id.to_string(),
            width: Pt(100000),
            height: Pt(100000),
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    }
}

fn make_image_sized(id: &str, w: i128, h: i128) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "body".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Image {
            src: id.to_string(),
            width: Pt(w),
            height: Pt(h),
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    }
}

// Helper to create a container
fn make_container(id: &str, children: Vec<SemanticNode>) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "section".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container { children },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    }
}

fn make_list_item(
    id: &str,
    list_id: &str,
    marker_type: k2f_core::ListMarkerType,
    text: &str,
) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "list_item".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: Some(list_id.to_string()),
        depth: Some(0),
        marker_type: Some(marker_type),
        content: NodeContent::Text(text.to_string()),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    }
}

#[test]
fn test_container_padding_offsets_children() {
    let fonts = crate::test_utils::test_fonts();

    use crate::{RoleStyle, Theme};
    use std::collections::HashMap;

    // 10pt padding on all sides.
    let section_decoration = ThemeDecoration {
        padding_pt: Some(EdgeInsetsPt::Uniform(10_000)),
        ..ThemeDecoration::default()
    };

    let mut roles = HashMap::new();
    roles.insert(
        "section".to_string(),
        RoleStyle {
            font_family: "default".to_string(),
            font_size: Pt(12_000),
            line_height_mult: 1_200,
            color: "black".to_string(),
            text_align: crate::style::TextAlign::Start,
            self_align: None,
            box_decoration: Some(section_decoration),
            list_style: None,
            bold: false,
            italic: false,
            letter_spacing_pt: Pt::ZERO,
            first_line_indent_pt: Pt::ZERO,
            variants: HashMap::new(),
        },
    );

    let theme = Theme {
        palette: HashMap::new(),
        primitives: Default::default(),
        roles,
        modifiers: crate::ModifierTheme::default(),
        font_aliases: HashMap::new(),
    };

    let ctx = LayoutContext::new(&fonts, &theme);

    let a = make_image("a");
    let b = make_image("b");
    let container = make_container("root", vec![a, b]);

    let measured = measure_node(&container, SizeConstraint::infinite(), &ctx).unwrap();
    let start_pos = Point::new(Pt(50_000), Pt(50_000));
    let geo = arrange_node(&container, start_pos, measured, &ctx).unwrap();

    assert_eq!(geo.children.len(), 2);
    let pad = Pt(10_000);
    assert_eq!(geo.children[0].x, start_pos.x + pad);
    assert_eq!(geo.children[0].y, start_pos.y + pad);
    assert_eq!(geo.children[1].x, start_pos.x + pad);
    assert_eq!(geo.children[1].y, start_pos.y + pad + Pt(100_000));
}

#[test]
fn test_overlay_positions_children_at_same_origin_and_size_is_max_child() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let a = make_image_sized("a", 100_000, 100_000);
    let b = make_image_sized("b", 150_000, 80_000);
    let container = SemanticNode {
        id: "root".to_string(),
        role: "section".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container {
            children: vec![a, b],
        },
        modifiers: vec![],
        layout: Some(LayoutHint::Overlay {
            size: FixedSizeHint::default(),
        }),
        ..Default::default()
    };

    let measured = measure_node(&container, SizeConstraint::infinite(), &ctx).unwrap();
    assert_eq!(measured.width, Pt(150_000));
    assert_eq!(measured.height, Pt(100_000));

    let start_pos = Point::new(Pt(50_000), Pt(60_000));
    let geo = arrange_node(&container, start_pos, measured, &ctx).unwrap();
    assert_eq!(geo.children.len(), 2);

    // Both children start at the same absolute origin inside the overlay container.
    assert_eq!(geo.children[0].x, start_pos.x);
    assert_eq!(geo.children[0].y, start_pos.y);
    assert_eq!(geo.children[1].x, start_pos.x);
    assert_eq!(geo.children[1].y, start_pos.y);
}

#[test]
fn test_vertical_stacking() {
    // Parent at (100, 100)
    // Child A (Text) Should be at (100, 100)
    // Child B (Text) Should be at (100, 100 + HeightA)

    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);
    let text_a = make_text("a", "Hello");
    let text_b = make_text("b", "World");
    let container = make_container("root", vec![text_a, text_b]);

    let start_pos = Point::new(Pt(100000), Pt(100000)); // (100pt, 100pt)
    let container_size = Size::new(Pt(500000), Pt(500000)); // Arbitrary big size used for constraint

    let geo = arrange_node(&container, start_pos, container_size, &ctx).unwrap();

    assert_eq!(geo.children.len(), 2);

    let child_a = &geo.children[0];
    let child_b = &geo.children[1];

    // Check Child A
    assert_eq!(child_a.x, start_pos.x);
    assert_eq!(child_a.y, start_pos.y);
    assert!(child_a.height.0 > 0);

    // Check Child B
    assert_eq!(child_b.x, start_pos.x);
    // Absolute Y of B = Absolute Y of A + Height of A
    assert_eq!(child_b.y, child_a.y + child_a.height);
}

#[test]
fn test_horizontal_stacking_with_gap_positions_children_by_x() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    // Use images so sizes are deterministic (explicit 100pt x 100pt size hints).
    let a = make_image("a");
    let b = make_image("b");
    let c = make_image("c");

    let gap = 2000_i64; // 2pt
    let gap_pt = Pt(gap as i128);
    let container = SemanticNode {
        id: "root".to_string(),
        role: "section".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container {
            children: vec![a, b, c],
        },
        modifiers: vec![],
        layout: Some(LayoutHint::Stack {
            direction: StackDirection::Horizontal,
            gap,
            align_items: Align::default(),
            justify_content: JustifyContent::default(),
            size: FixedSizeHint::default(),
        }),
        ..Default::default()
    };

    let measured = measure_node(&container, SizeConstraint::infinite(), &ctx).unwrap();
    let start_pos = Point::new(Pt(100000), Pt(250000)); // (100pt, 250pt)

    let geo = arrange_node(&container, start_pos, measured, &ctx).unwrap();
    assert_eq!(geo.children.len(), 3);

    // Image size hint is 100pt = 100000 (in 1/1000pt units)
    let w = Pt(100000);

    assert_eq!(geo.children[0].x, start_pos.x);
    assert_eq!(geo.children[0].y, start_pos.y);

    assert_eq!(geo.children[1].x, start_pos.x + w + gap_pt);
    assert_eq!(geo.children[1].y, start_pos.y);

    assert_eq!(geo.children[2].x, start_pos.x + Pt(w.0 * 2 + gap_pt.0 * 2));
    assert_eq!(geo.children[2].y, start_pos.y);
}

#[test]
fn test_arrange_list_item_emits_marker_before_body_glyphs() {
    use crate::list_style::ListStyle;
    use std::collections::HashMap;

    let fonts = crate::test_utils::test_fonts();

    let mut roles = HashMap::new();
    roles.insert(
        "list_item".to_string(),
        crate::RoleStyle {
            font_family: "default".to_string(),
            font_size: Pt(12_000),
            line_height_mult: 1_200,
            color: "black".to_string(),
            text_align: crate::style::TextAlign::Start,
            self_align: None,
            box_decoration: Some(ThemeDecoration::default()),
            list_style: Some(ListStyle {
                marker_box_width_pt: Some(Pt(18_000)),
                marker_gap_pt: Some(Pt(4_000)),
                depth_indent_pt: Some(Pt(12_000)),
                bullet_glyph: Some("*".to_string()),
                ..ListStyle::default()
            }),
            bold: false,
            italic: false,
            letter_spacing_pt: Pt::ZERO,
            first_line_indent_pt: Pt::ZERO,
            variants: HashMap::new(),
        },
    );

    let theme = Theme {
        palette: HashMap::new(),
        primitives: Default::default(),
        roles,
        modifiers: crate::ModifierTheme::default(),
        font_aliases: HashMap::new(),
    };

    let item = make_list_item("li.1", "L", k2f_core::ListMarkerType::Bullet, "Hello");
    let root = make_container("root", vec![item.clone()]);

    let mut ctx = LayoutContext::new(&fonts, &theme);
    ctx.list_markers = crate::list_markers::derive_list_marker_map(&root, &theme).unwrap();

    let constraint = SizeConstraint::new(Size::ZERO, Size::new(Pt(200_000), Pt(i128::MAX)));
    let measured = measure_node(&item, constraint, &ctx).unwrap();
    let geo = arrange_node(&item, Point::ZERO, measured, &ctx).unwrap();

    assert!(geo.text_runs.len() >= 2);
    let marker_run = &geo.text_runs[0];
    let body_runs = &geo.text_runs[1..];

    let marker_glyphs = &geo.glyphs[marker_run.glyph_range[0]..marker_run.glyph_range[1]];
    let mut marker_max_x = Pt::ZERO;
    for g in marker_glyphs {
        if g.x_offset > marker_max_x {
            marker_max_x = g.x_offset;
        }
    }

    let mut body_min_x = Pt(i128::MAX);
    for r in body_runs {
        for g in &geo.glyphs[r.glyph_range[0]..r.glyph_range[1]] {
            if g.x_offset < body_min_x {
                body_min_x = g.x_offset;
            }
        }
    }

    assert!(marker_max_x < body_min_x);
}

#[test]
fn test_arrange_list_item_errors_when_marker_missing() {
    use crate::list_style::ListStyle;
    use std::collections::HashMap;

    let fonts = crate::test_utils::test_fonts();

    let mut roles = HashMap::new();
    roles.insert(
        "list_item".to_string(),
        crate::RoleStyle {
            font_family: "default".to_string(),
            font_size: Pt(12_000),
            line_height_mult: 1_200,
            color: "black".to_string(),
            text_align: crate::style::TextAlign::Start,
            self_align: None,
            box_decoration: Some(ThemeDecoration::default()),
            list_style: Some(ListStyle {
                marker_box_width_pt: Some(Pt(18_000)),
                marker_gap_pt: Some(Pt(4_000)),
                depth_indent_pt: Some(Pt(12_000)),
                bullet_glyph: Some("*".to_string()),
                ..ListStyle::default()
            }),
            bold: false,
            italic: false,
            letter_spacing_pt: Pt::ZERO,
            first_line_indent_pt: Pt::ZERO,
            variants: HashMap::new(),
        },
    );

    let theme = Theme {
        palette: HashMap::new(),
        primitives: Default::default(),
        roles,
        modifiers: crate::ModifierTheme::default(),
        font_aliases: HashMap::new(),
    };

    let item = make_list_item("li.1", "L", k2f_core::ListMarkerType::Bullet, "Hello");
    let mut ctx = LayoutContext::new(&fonts, &theme);

    let constraint = SizeConstraint::new(Size::ZERO, Size::new(Pt(200_000), Pt(i128::MAX)));
    let measured = measure_node(&item, constraint, &ctx).unwrap();
    let err = arrange_node(&item, Point::ZERO, measured, &ctx).unwrap_err();
    assert!(err.contains("Missing derived marker label"));
}

#[test]
fn test_not_at_origin() {
    // If we verify that child coordinates are truly absolute, not relative to parent
    // If we verify that child coordinates are truly absolute, not relative to parent
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);
    let text = make_text("t", "Text");
    let container = make_container("root", vec![text]);

    // Parent placed at (50, 50)
    let start_pos = Point::new(Pt(50000), Pt(50000));
    let size = Size::new(Pt(100000), Pt(100000));

    let geo = arrange_node(&container, start_pos, size, &ctx).unwrap();

    // Child should be at (50, 50), NOT (0,0) relative to parent
    let child = &geo.children[0];
    assert_eq!(child.x, Pt(50000));
    assert_eq!(child.y, Pt(50000));
}

#[test]
fn test_arrange_text_wrapped_glyphs_within_width_and_monotonic_x() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    // Use simple ASCII to avoid shaping edge-cases (marks/RTL) for this invariant test.
    // Small width forces wrapping into multiple lines.
    let text = "aaaa aaaa aaaa aaaa aaaa aaaa aaaa aaaa aaaa aaaa aaaa aaaa";
    let node = make_text("t", text);

    let max_width = Pt(60000); // 60pt
    let constraint = SizeConstraint::new(Size::ZERO, Size::new(max_width, Pt(i128::MAX)));
    let measured = measure_node(&node, constraint, &ctx).unwrap();

    let geo = arrange_node(&node, Point::ZERO, measured, &ctx).unwrap();
    assert!(!geo.glyphs.is_empty(), "Expected shaped glyphs");

    // Group glyphs by the line offset we applied (y_offset == line-top offset for simple horizontal text).
    let mut by_line: BTreeMap<i128, Vec<&k2f_core::GlyphPosition>> = BTreeMap::new();
    for g in &geo.glyphs {
        by_line.entry(g.y_offset.0).or_default().push(g);
    }
    assert!(
        by_line.len() > 1,
        "Expected wrapping to produce multiple lines (got {})",
        by_line.len()
    );

    for (_y, glyphs) in by_line {
        let mut last_x: Option<Pt> = None;
        for g in glyphs {
            if let Some(prev) = last_x {
                assert!(
                    g.x_offset >= prev,
                    "Glyph x_offset should be non-decreasing within a line"
                );
            }
            assert!(
                g.x_offset + g.x_advance <= measured.width,
                "Glyph should not exceed node width"
            );
            last_x = Some(g.x_offset);
        }
    }
}
