use crate::visual_primitives::EdgeInsetsPt;
use crate::Theme;
use crate::ThemeDecoration;
use crate::{measure_node, LayoutContext, Size, SizeConstraint};
use k2f_core::{
    Align, FixedSizeHint, JustifyContent, LayoutHint, NodeContent, Pt, SemanticNode, StackDirection,
};

#[test]
fn test_measure_text_node() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let node = SemanticNode {
        id: "1".into(),
        role: "body".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Text("Hello".into()),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    let size = measure_node(&node, SizeConstraint::infinite(), &ctx).unwrap();

    // "Hello" with no font -> fallback logic
    assert!(size.width.0 > 0, "Width should be positive");
    assert!(size.height.0 > 0, "Height should be positive");
}

#[test]
fn test_measure_list_item_is_marker_independent() {
    let fonts = crate::test_utils::test_fonts();

    use crate::{list_style::ListStyle, RoleStyle, Theme};
    use std::collections::HashMap;

    let mut roles = HashMap::new();
    roles.insert(
        "list_item".to_string(),
        RoleStyle {
            font_family: "default".to_string(),
            font_size: Pt(12_000),
            line_height_mult: 1_200,
            color: "black".to_string(),
            text_align: crate::style::TextAlign::Start,
            self_align: None,
            box_decoration: None,
            list_style: Some(ListStyle {
                marker_box_width_pt: Some(Pt(18_000)),
                marker_gap_pt: Some(Pt(4_000)),
                depth_indent_pt: Some(Pt(12_000)),
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

    let ctx = LayoutContext::new(&fonts, &theme);

    let constraint = SizeConstraint::new(Size::ZERO, Size::new(Pt(120_000), Pt(i128::MAX)));
    let long_text = "Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau";

    let node_bullet = SemanticNode {
        id: "li.1".into(),
        role: "list_item".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: Some("list_a".into()),
        depth: Some(0),
        marker_type: Some(k2f_core::ListMarkerType::Bullet),
        content: NodeContent::Text(long_text.into()),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };
    let node_number = SemanticNode {
        marker_type: Some(k2f_core::ListMarkerType::Number),
        ..node_bullet.clone()
    };

    let s1 = measure_node(&node_bullet, constraint, &ctx).unwrap();
    let s2 = measure_node(&node_number, constraint, &ctx).unwrap();

    assert_eq!(s1, s2);
}

#[test]
fn test_measure_list_item_depth_narrows_wrap_width() {
    let fonts = crate::test_utils::test_fonts();

    use crate::{list_style::ListStyle, RoleStyle, Theme};
    use std::collections::HashMap;

    let mut roles = HashMap::new();
    roles.insert(
        "list_item".to_string(),
        RoleStyle {
            font_family: "default".to_string(),
            font_size: Pt(12_000),
            line_height_mult: 1_200,
            color: "black".to_string(),
            text_align: crate::style::TextAlign::Start,
            self_align: None,
            box_decoration: None,
            list_style: Some(ListStyle {
                marker_box_width_pt: Some(Pt(18_000)),
                marker_gap_pt: Some(Pt(4_000)),
                depth_indent_pt: Some(Pt(12_000)),
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

    let ctx = LayoutContext::new(&fonts, &theme);

    let constraint = SizeConstraint::new(Size::ZERO, Size::new(Pt(120_000), Pt(i128::MAX)));
    let long_text = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty";

    let base = SemanticNode {
        id: "li.depth".into(),
        role: "list_item".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: Some("list_a".into()),
        depth: Some(0),
        marker_type: Some(k2f_core::ListMarkerType::Number),
        content: NodeContent::Text(long_text.into()),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    let s0 = measure_node(&base, constraint, &ctx).unwrap();
    let s2 = measure_node(
        &SemanticNode {
            depth: Some(2),
            ..base
        },
        constraint,
        &ctx,
    )
    .unwrap();

    assert!(s2.height.0 >= s0.height.0);
}

#[test]
fn test_measure_container_stack() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let child1 = SemanticNode {
        id: "c1".into(),
        role: "body".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Text("A".into()),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    let child2 = SemanticNode {
        id: "c2".into(),
        role: "body".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Text("BB".into()),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    let container = SemanticNode {
        id: "root".into(),
        role: "section".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container {
            children: vec![child1, child2],
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    let size = measure_node(&container, SizeConstraint::infinite(), &ctx).unwrap();

    // Container Logic: Max width, Sum height
    // We expect size > 0.
    assert!(size.width.0 > 0);
    assert!(size.height.0 > 0);
}

#[test]
fn test_measure_container_padding_affects_size() {
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

    let c1 = SemanticNode {
        id: "i1".into(),
        role: "body".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Image {
            src: "a".into(),
            width: Pt(100_000),
            height: Pt(100_000),
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };
    let c2 = SemanticNode {
        id: "i2".into(),
        role: "body".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Image {
            src: "b".into(),
            width: Pt(100_000),
            height: Pt(100_000),
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    let container = SemanticNode {
        id: "root".into(),
        role: "section".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container {
            children: vec![c1, c2],
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    let size = measure_node(&container, SizeConstraint::infinite(), &ctx).unwrap();

    // Inner: width=max(100), height=100+100. Outer adds 10pt*2 on each axis.
    assert_eq!(size.width, Pt(120_000));
    assert_eq!(size.height, Pt(220_000));
}

#[test]
fn test_measure_container_horizontal_stack_with_gap_is_deterministic() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    // Use Image nodes so measurement is fully deterministic and doesn't depend on text shaping.
    // Images carry explicit deterministic size hints (no placeholders).
    let c1 = SemanticNode {
        id: "i1".into(),
        role: "body".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Image {
            src: "a".into(),
            width: Pt(100000),
            height: Pt(100000),
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };
    let c2 = SemanticNode {
        id: "i2".into(),
        role: "body".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Image {
            src: "b".into(),
            width: Pt(100000),
            height: Pt(100000),
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };
    let c3 = SemanticNode {
        id: "i3".into(),
        role: "body".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Image {
            src: "c".into(),
            width: Pt(100000),
            height: Pt(100000),
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    let gap = 1000_i64; // 1pt
    let gap_pt = Pt(gap as i128);
    let container = SemanticNode {
        id: "root".into(),
        role: "section".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container {
            children: vec![c1, c2, c3],
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

    let size = measure_node(&container, SizeConstraint::infinite(), &ctx).unwrap();

    // 3 images: width = 100 + 100 + 100 + 2 gaps
    assert_eq!(size.width, Pt(100000i128 * 3 + gap_pt.0 * 2));
    // height = max child height = 100
    assert_eq!(size.height, Pt(100000));
}

#[test]
fn test_measure_image_scales_down_to_fit_constraint_preserving_aspect_ratio() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    // Intrinsic: 200pt x 100pt (2:1). Constrain to 100pt x 100pt -> should become 100pt x 50pt.
    let node = SemanticNode {
        id: "img".into(),
        role: "body".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Image {
            src: "chart.png".into(),
            width: Pt(200000),
            height: Pt(100000),
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    let constraint = SizeConstraint::new(Size::ZERO, Size::new(Pt(100000), Pt(100000)));
    let size = measure_node(&node, constraint, &ctx).unwrap();
    assert_eq!(size.width, Pt(100000));
    assert_eq!(size.height, Pt(50000));
}

#[test]
fn test_measure_fixed_size_overlay_min_outer_size_even_when_empty() {
    let fonts = crate::test_utils::test_fonts();

    use crate::{RoleStyle, Theme};
    use std::collections::HashMap;

    // 10pt padding on all sides, so an empty container would normally measure to 20pt x 20pt.
    let decoration = ThemeDecoration {
        padding_pt: Some(EdgeInsetsPt::Uniform(10_000)),
        ..ThemeDecoration::default()
    };

    let mut roles = HashMap::new();
    roles.insert(
        "shape".to_string(),
        RoleStyle {
            font_family: "default".to_string(),
            font_size: Pt(12_000),
            line_height_mult: 1_200,
            color: "black".to_string(),
            text_align: crate::style::TextAlign::Start,
            self_align: None,
            box_decoration: Some(decoration),
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

    let node = SemanticNode {
        id: "shape".into(),
        role: "shape".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container { children: vec![] },
        modifiers: vec![],
        layout: Some(LayoutHint::Overlay {
            size: FixedSizeHint {
                width: Some(Pt(100_000)),
                height: Some(Pt(80_000)),
            },
        }),
        ..Default::default()
    };

    let measured = measure_node(&node, SizeConstraint::infinite(), &ctx).unwrap();
    assert_eq!(measured.width, Pt(100_000));
    assert_eq!(measured.height, Pt(80_000));
}

#[test]
fn test_measure_table_reference_uses_deterministic_size_hint_and_respects_constraint() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let node = SemanticNode {
        id: "tbl".into(),
        role: "table".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::TableReference {
            source: "asset://data/table.csv".into(),
            view_mode: "full".into(),
            width: Pt(200000),
            height: Pt(100000),
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    let constraint = SizeConstraint::new(Size::ZERO, Size::new(Pt(150000), Pt(50000)));
    let size = measure_node(&node, constraint, &ctx).unwrap();
    assert_eq!(size.width, Pt(150000));
    assert_eq!(size.height, Pt(50000));
}

#[test]
fn test_measure_style_integration() {
    // Verify that applying a style modifier affects the measured size
    let fonts = crate::test_utils::test_fonts();
    let cx_theme = Theme::default(); // default "body" is 12pt
    let ctx = LayoutContext::new(&fonts, &cx_theme);

    // 1. Measure normal body text
    let node_normal = SemanticNode {
        id: "1".into(),
        role: "body".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Text("Text".into()),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };
    let _size_normal = measure_node(&node_normal, SizeConstraint::infinite(), &ctx).unwrap();

    // 2. Measure header text (via modifier or role)
    // Using role "h1" which should be larger (32pt in tokens.json, but here using default theme in tests)
    // Wait, Theme::default() is empty or minimal?
    // In style.rs, default() returns hardcoded defaults.
    // "h1" is not in Theme::default() map unless I explicitly put it there or if I load tokens.json.
    // Tests currently use Style::default() fallback for everything unless I populate the map.

    // Let's populate a theme for this test to be sure.
    // I can't easily populate Theme struct fields as they are public but I need to construct HashMaps.
    // Or I can use a modifier that sets specific size if I support that?
    // My resolve_style implementation supports intent -> role lookup.

    // Let's rely on the fact that if I give a modifier intent "header", expecting it to fail lookup and fallback?
    // No, I want to succeed.
    // Since I cannot easily modify Theme::default() without helper, I'll verify "bold" or something if it affects size?
    // Bold doesn't affect size in my simple measure_text logic (width based on font, fallback based on font_size).

    // But font_size DOES affect size.
    // I need distinct roles in the theme for this test.
    // Since I can't construct Theme easily (no public builder in this scope without verbose HashMap construction),
    // I will construct a Theme with manually inserted roles.

    use crate::{RoleStyle, Theme};
    use k2f_core::Pt;
    use std::collections::HashMap;

    let mut roles = HashMap::new();
    roles.insert(
        "body".to_string(),
        RoleStyle {
            font_family: "default".to_string(),
            font_size: Pt(10000), // 10pt
            line_height_mult: 1000,
            color: "black".to_string(),
            text_align: crate::style::TextAlign::Start,
            self_align: None,
            box_decoration: None,
            list_style: None,
            bold: false,
            italic: false,
            letter_spacing_pt: Pt::ZERO,
            first_line_indent_pt: Pt::ZERO,
            variants: HashMap::new(),
        },
    );
    roles.insert(
        "big".to_string(),
        RoleStyle {
            font_family: "default".to_string(),
            font_size: Pt(20000), // 20pt
            line_height_mult: 1000,
            color: "black".to_string(),
            text_align: crate::style::TextAlign::Start,
            self_align: None,
            box_decoration: None,
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

    let ctx_custom = LayoutContext::new(&fonts, &theme);

    // Measure body
    let node_body = SemanticNode {
        id: "1".into(),
        role: "body".into(), // Should trigger "body" role
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Text("Test".into()),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };
    let size_body = measure_node(&node_body, SizeConstraint::infinite(), &ctx_custom).unwrap();

    // Measure big
    let node_big = SemanticNode {
        id: "2".into(),
        role: "big".into(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Text("Test".into()),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };
    let size_big = measure_node(&node_big, SizeConstraint::infinite(), &ctx_custom).unwrap();

    // Expect big > body
    assert!(
        size_big.height.0 > size_body.height.0,
        "Big role should result in larger height"
    );
    assert!(
        size_big.width.0 > size_body.width.0,
        "Big role should result in larger width"
    );
}
