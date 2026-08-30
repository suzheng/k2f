use crate::render_plan::build_render_plan;
use crate::theme::{RoleStyle, Theme};
use crate::{arrange_node, measure_node, LayoutContext, LayoutEngine, Point, Size, SizeConstraint};
use k2f_core::{
    BreakInside, CanvasMode, GlyphPosition, GridTrack, LayoutResult, Manifest, NodeContent,
    PageConfig, PaintOp, Pt, SemanticNode, TableDataSource, TableSpec,
};
use k2f_text::FontLibrary;
use std::collections::HashMap;

fn math_fonts() -> FontLibrary {
    let mut fonts = crate::test_utils::test_fonts();
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fonts/NotoSansMath-Regular.ttf");
    let data = std::fs::read(&path).expect("NotoSansMath-Regular.ttf");
    fonts.add_font("NotoSansMath-Regular", data);
    fonts
}

fn math_theme() -> Theme {
    let mut roles = HashMap::new();
    roles.insert(
        "math".to_string(),
        RoleStyle {
            font_family: "NotoSansMath-Regular".to_string(),
            font_size: Pt(12_000),
            line_height_mult: 1_200,
            color: "#111111".to_string(),
            text_align: crate::style::TextAlign::Center,
            ..Default::default()
        },
    );
    roles.insert(
        "document".to_string(),
        RoleStyle {
            font_family: "default".to_string(),
            font_size: Pt(12_000),
            line_height_mult: 1_200,
            color: "#111111".to_string(),
            ..Default::default()
        },
    );
    Theme {
        palette: HashMap::new(),
        primitives: Default::default(),
        roles,
        modifiers: Default::default(),
        font_aliases: HashMap::new(),
    }
}

fn math_node(id: &str, tex: &str) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "math".to_string(),
        break_inside: BreakInside::Avoid,
        content: NodeContent::Math(tex.to_string()),
        ..Default::default()
    }
}

fn body_theme() -> Theme {
    let mut theme = math_theme();
    theme.roles.insert(
        "body".to_string(),
        RoleStyle {
            font_family: "default".to_string(),
            font_size: Pt(12_000),
            line_height_mult: 1_200,
            color: "#111111".to_string(),
            ..Default::default()
        },
    );
    theme.roles.insert(
        "table".to_string(),
        RoleStyle {
            font_family: "default".to_string(),
            font_size: Pt(12_000),
            line_height_mult: 1_200,
            color: "#111111".to_string(),
            ..Default::default()
        },
    );
    theme
}

fn body_with_inline_math(surround: &str, tex: &str) -> SemanticNode {
    let start = surround.find('\u{FFFC}').expect("placeholder");
    SemanticNode {
        id: "p".into(),
        role: "body".into(),
        content: NodeContent::Text(surround.into()),
        modifiers: vec![k2f_core::Modifier {
            range: [start, start + '\u{FFFC}'.len_utf8()],
            mod_type: "math".into(),
            intent: tex.into(),
        }],
        ..Default::default()
    }
}

#[test]
fn inline_math_arranges_atom_in_body_text() {
    let fonts = math_fonts();
    let theme = body_theme();
    let ctx = LayoutContext::new(&fonts, &theme);
    let node = body_with_inline_math("x \u{FFFC} y", r"\frac{1}{2}");
    let size = measure_node(&node, SizeConstraint::infinite(), &ctx).unwrap();
    let geo = arrange_node(&node, Point::ZERO, size, &ctx).unwrap();
    assert!(geo.glyphs.len() >= 3);
    assert!(!geo.fill_rects.is_empty());
}

#[test]
fn unknown_inline_tex_fails_compile() {
    let fonts = math_fonts();
    let theme = body_theme();
    let ctx = LayoutContext::new(&fonts, &theme);
    let err = measure_node(
        &body_with_inline_math("\u{FFFC}", r"\unknown"),
        SizeConstraint::infinite(),
        &ctx,
    )
    .unwrap_err();
    assert!(err.starts_with("MATH_UNSUPPORTED:"), "got {err}");
}

#[test]
fn inline_math_wider_than_line_is_unsplittable() {
    let fonts = math_fonts();
    let theme = body_theme();
    let ctx = LayoutContext::new(&fonts, &theme);
    let err = measure_node(
        &body_with_inline_math("\u{FFFC}", "xxxxxxxxxx"),
        SizeConstraint::new(Size::ZERO, Size::new(Pt(1), Pt(i128::MAX))),
        &ctx,
    )
    .unwrap_err();
    assert!(err.contains("UNSPLITTABLE_OVERFLOW"), "got {err}");
}

#[test]
fn table_cell_math_node_arranges() {
    let fonts = math_fonts();
    let theme = body_theme();
    let ctx = LayoutContext::new(&fonts, &theme);
    let table = SemanticNode {
        id: "tbl".into(),
        role: "table".into(),
        content: NodeContent::Table(TableSpec {
            column_widths: vec![GridTrack::Pt { pt: 200_000 }],
            header_rows: 0,
            gap: 0,
            data: TableDataSource::Inline {
                rows: vec![vec![math_node("eq.cell", r"\frac{1}{2}")]],
            },
        }),
        ..Default::default()
    };
    let size = measure_node(&table, SizeConstraint::infinite(), &ctx).unwrap();
    assert!(size.width.0 > 0);
    assert!(size.height.0 > 0);
    let geo = arrange_node(&table, Point::ZERO, size, &ctx).unwrap();
    let has_rule = geo.fill_rects.iter().any(|r| r.width.0 > 0)
        || geo.children.iter().any(|c| {
            c.fill_rects.iter().any(|r| r.width.0 > 0)
                || c.children.iter().any(|g| !g.fill_rects.is_empty())
        });
    assert!(has_rule, "expected fraction rule in table cell geometry");
}

#[test]
fn measure_simple_math_has_positive_size() {
    let fonts = math_fonts();
    let theme = math_theme();
    let ctx = LayoutContext::new(&fonts, &theme);
    let size = measure_node(&math_node("eq.x", "x"), SizeConstraint::infinite(), &ctx).unwrap();
    assert!(size.width.0 > 0);
    assert!(size.height.0 > 0);
}

#[test]
fn measure_frac_is_taller_than_atom() {
    let fonts = math_fonts();
    let theme = math_theme();
    let ctx = LayoutContext::new(&fonts, &theme);
    let atom = measure_node(&math_node("eq.x", "x"), SizeConstraint::infinite(), &ctx).unwrap();
    let frac = measure_node(
        &math_node("eq.frac", r"\frac{1}{2}"),
        SizeConstraint::infinite(),
        &ctx,
    )
    .unwrap();
    assert!(frac.height.0 > atom.height.0, "frac={frac:?} atom={atom:?}");
}

#[test]
fn measure_does_not_clamp_wide_formula() {
    let fonts = math_fonts();
    let theme = math_theme();
    let ctx = LayoutContext::new(&fonts, &theme);
    let node = math_node("eq.wide", "xxxxxxxxxx");
    let intrinsic = measure_node(&node, SizeConstraint::infinite(), &ctx).unwrap();
    let narrow = SizeConstraint::new(Size::ZERO, Size::new(Pt(1_000), Pt(i128::MAX)));
    let overflowed = measure_node(&node, narrow, &ctx).unwrap();
    assert_eq!(overflowed.width, intrinsic.width);
    assert!(overflowed.width.0 > 1_000);
}

#[test]
fn unknown_command_fails_compile() {
    let fonts = math_fonts();
    let theme = math_theme();
    let ctx = LayoutContext::new(&fonts, &theme);
    let err = measure_node(
        &math_node("eq.bad", r"\unknown"),
        SizeConstraint::infinite(),
        &ctx,
    )
    .unwrap_err();
    assert!(err.starts_with("MATH_UNSUPPORTED:"), "got {err}");
}

#[test]
fn arrange_left_right_frac_has_parens_and_rule() {
    let fonts = math_fonts();
    let theme = math_theme();
    let ctx = LayoutContext::new(&fonts, &theme);
    let node = math_node("eq.lr", r"\left(\frac{1}{2}\right)");
    let size = measure_node(&node, SizeConstraint::infinite(), &ctx).unwrap();
    let geo = arrange_node(&node, Point::ZERO, size, &ctx).unwrap();
    assert!(!geo.fill_rects.is_empty());
    assert!(geo.glyphs.len() >= 4, "got {} glyphs", geo.glyphs.len());
}

#[test]
fn arrange_pmatrix_has_delimiters() {
    let fonts = math_fonts();
    let theme = math_theme();
    let ctx = LayoutContext::new(&fonts, &theme);
    let inner = math_node("eq.m", r"\begin{matrix} a \\ b \end{matrix}");
    let wrapped = math_node("eq.p", r"\begin{pmatrix} a \\ b \end{pmatrix}");
    let inner_size = measure_node(&inner, SizeConstraint::infinite(), &ctx).unwrap();
    let wrapped_size = measure_node(&wrapped, SizeConstraint::infinite(), &ctx).unwrap();
    assert!(wrapped_size.width.0 > inner_size.width.0);
    let geo = arrange_node(&wrapped, Point::ZERO, wrapped_size, &ctx).unwrap();
    assert!(geo.glyphs.len() >= 4);
}

#[test]
fn arrange_frac_has_rule_and_not_source_clusters() {
    let fonts = math_fonts();
    let theme = math_theme();
    let ctx = LayoutContext::new(&fonts, &theme);
    let node = math_node("eq.frac", r"\frac{1}{2}");
    let size = measure_node(&node, SizeConstraint::infinite(), &ctx).unwrap();
    let geo = arrange_node(&node, Point::new(Pt(5_000), Pt(7_000)), size, &ctx).unwrap();
    assert!(!geo.fill_rects.is_empty());
    assert!(geo.glyphs.len() >= 2);
    assert!(geo
        .glyphs
        .iter()
        .all(|g| g.cluster == GlyphPosition::CLUSTER_NOT_SOURCE));
    for r in &geo.fill_rects {
        assert!(r.x.0 >= geo.x.0);
        assert!(r.y.0 >= geo.y.0);
        assert!(r.width.0 > 0);
        assert!(r.height.0 > 0);
    }
}

#[test]
fn arrange_math_is_deterministic() {
    let fonts = math_fonts();
    let theme = math_theme();
    let ctx = LayoutContext::new(&fonts, &theme);
    let node = math_node("eq.e", "E=mc^2");
    let size = measure_node(&node, SizeConstraint::infinite(), &ctx).unwrap();
    let a = arrange_node(&node, Point::ZERO, size, &ctx).unwrap();
    let b = arrange_node(&node, Point::ZERO, size, &ctx).unwrap();
    assert_eq!(a, b);
}

#[test]
fn render_plan_emits_rule_boxes_before_text() {
    let fonts = math_fonts();
    let theme = math_theme();
    let ctx = LayoutContext::new(&fonts, &theme);
    let node = math_node("eq.frac", r"\frac{1}{2}");
    let size = measure_node(&node, SizeConstraint::infinite(), &ctx).unwrap();
    let geo = arrange_node(&node, Point::ZERO, size, &ctx).unwrap();
    assert!(!geo.fill_rects.is_empty());

    let manifest = Manifest {
        title: "math".into(),
        canvas_mode: CanvasMode::Paged,
        page_config: PageConfig {
            width: Pt(595_000),
            height: Pt(842_000),
            margin: [Pt(72_000); 4],
        },
        root: node,
        running_blocks: vec![],
    };
    let layout = LayoutResult {
        pages: vec![k2f_core::Page {
            index: 0,
            width: Pt(595_000),
            height: Pt(842_000),
            root: k2f_core::GeometryNode {
                id: "page".into(),
                x: Pt::ZERO,
                y: Pt::ZERO,
                width: Pt(595_000),
                height: Pt(842_000),
                glyphs: vec![],
                text_runs: vec![],
                fill_rects: vec![],
                children: vec![geo],
            },
        }],
    };
    let plan = build_render_plan(&manifest, &layout, &theme).unwrap();
    let ops = &plan.pages[0].ops;
    let rule_idx = ops.iter().position(|op| match op {
        PaintOp::DrawBox { node_id, .. } => node_id.starts_with("eq.frac::rule_"),
        _ => false,
    });
    let text_idx = ops.iter().position(|op| match op {
        PaintOp::DrawText { node_id, .. } => node_id == "eq.frac",
        _ => false,
    });
    assert!(rule_idx.is_some(), "missing rule DrawBox in {ops:?}");
    assert!(text_idx.is_some(), "missing DrawText in {ops:?}");
    assert!(rule_idx.unwrap() < text_idx.unwrap());
}

#[test]
fn compile_chunk_rejects_unknown_tex() {
    let font = std::fs::read(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts/NotoSansMath-Regular.ttf"),
    )
    .unwrap();
    let content = r#"{
        "title": "bad math",
        "canvas_mode": "paged",
        "page_config": { "width": 595000, "height": 842000, "margin": [72000, 72000, 72000, 72000] },
        "root": {
            "id": "eq.bad",
            "role": "math",
            "break_inside": "avoid",
            "content": { "type": "math", "value": "\\unknown" }
        }
    }"#;
    let theme = r##"{
        "palette": { "black": "#000000" },
        "roles": {
            "default": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
            "document": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
            "math": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black", "text_align": "center" }
        }
    }"##;
    let err = LayoutEngine::compile_chunk(content, theme, &font).unwrap_err();
    assert!(err.contains("MATH_UNSUPPORTED"), "got {err}");
}
