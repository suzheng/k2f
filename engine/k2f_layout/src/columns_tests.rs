use crate::grid::resolve_tracks;
use crate::Theme;
use crate::{LayoutContext, LayoutEngine};
use k2f_core::{
    CanvasMode, ColumnSpan, GridTrack, LayoutHint, Manifest, NodeContent, PageConfig, Pt,
    SemanticNode,
};

fn make_text(id: &str, content: &str) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "body".to_string(),
        content: NodeContent::Text(content.to_string()),
        ..Default::default()
    }
}

fn columns_root(children: Vec<SemanticNode>) -> SemanticNode {
    SemanticNode {
        id: "root".to_string(),
        role: "document".to_string(),
        content: NodeContent::Container {
            children: vec![SemanticNode {
                id: "body".to_string(),
                role: "section".to_string(),
                layout: Some(LayoutHint::Columns {
                    count: 2,
                    gap: 10_000,
                }),
                content: NodeContent::Container { children },
                ..Default::default()
            }],
        },
        ..Default::default()
    }
}

fn page(width: Pt, height: Pt) -> PageConfig {
    PageConfig {
        width,
        height,
        margin: [Pt(10_000), Pt(10_000), Pt(10_000), Pt(10_000)],
    }
}

#[test]
fn column_widths_sum_to_content_width() {
    let gap = Pt(10_000);
    let available = Pt(180_000);
    let tracks = vec![GridTrack::Fr { fr: 1 }, GridTrack::Fr { fr: 1 }];
    let widths = resolve_tracks(&tracks, gap, available).unwrap();
    assert_eq!(widths.len(), 2);
    assert_eq!(widths[0] + widths[1] + gap, available);
}

#[test]
fn long_body_fills_left_then_right_column() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    // Enough short lines that they must spill into column 1 on a short page.
    let mut kids = Vec::new();
    for i in 0..12 {
        kids.push(make_text(&format!("p{i}"), "Line of body text."));
    }
    let root = columns_root(kids);
    let page_config = page(Pt(200_000), Pt(80_000));
    let manifest = Manifest {
        title: "cols".into(),
        canvas_mode: CanvasMode::Paged,
        page_config: page_config.clone(),
        root,
        running_blocks: vec![],
    };
    let result = LayoutEngine::layout(&manifest, &ctx).unwrap();
    assert!(!result.pages.is_empty());
    let page0 = &result.pages[0];
    let left_x = page_config.margin[3];
    let gap = Pt(10_000);
    let content_w = page_config.width - page_config.margin[1] - page_config.margin[3];
    let widths = resolve_tracks(
        &[GridTrack::Fr { fr: 1 }, GridTrack::Fr { fr: 1 }],
        gap,
        content_w,
    )
    .unwrap();
    let right_x = left_x + widths[0] + gap;

    let mut saw_left = false;
    let mut saw_right = false;
    for child in &page0.root.children {
        if child.x == left_x {
            saw_left = true;
        }
        if child.x == right_x {
            saw_right = true;
        }
        // Column text must be narrower than full content width.
        assert!(
            child.width <= widths[0] + Pt(1),
            "child {} width {} exceeds column {}",
            child.id,
            child.width.0,
            widths[0].0
        );
    }
    assert!(saw_left, "expected items in left column");
    assert!(saw_right, "expected items in right column");
}

#[test]
fn title_outside_columns_is_full_width() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let title = SemanticNode {
        id: "title".into(),
        role: "h1".into(),
        content: NodeContent::Text("Paper Title".into()),
        keep_with_next: true,
        ..Default::default()
    };
    let body = SemanticNode {
        id: "body".into(),
        role: "section".into(),
        layout: Some(LayoutHint::Columns {
            count: 2,
            gap: 10_000,
        }),
        content: NodeContent::Container {
            children: vec![
                make_text("p1", "First paragraph in columns."),
                make_text("p2", "Second paragraph in columns."),
            ],
        },
        ..Default::default()
    };
    let root = SemanticNode {
        id: "root".into(),
        role: "document".into(),
        content: NodeContent::Container {
            children: vec![title, body],
        },
        ..Default::default()
    };
    let page_config = page(Pt(200_000), Pt(300_000));
    let content_w = page_config.width - page_config.margin[1] - page_config.margin[3];
    let manifest = Manifest {
        title: "paper".into(),
        canvas_mode: CanvasMode::Paged,
        page_config: page_config.clone(),
        root,
        running_blocks: vec![],
    };
    let result = LayoutEngine::layout(&manifest, &ctx).unwrap();
    let kids = &result.pages[0].root.children;
    let title_geo = kids.iter().find(|c| c.id == "title").expect("title");
    assert_eq!(title_geo.x, page_config.margin[3]);
    // Title uses full content width (stretch / natural at full measure width).
    assert!(
        title_geo.width >= content_w / 2,
        "title should be near full width, got {}",
        title_geo.width.0
    );
    let col_item = kids.iter().find(|c| c.id == "p1").expect("p1");
    assert!(col_item.width < content_w);
}

#[test]
fn column_span_all_is_full_width_between_columns() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let mut fig = SemanticNode {
        id: "fig".into(),
        role: "body".into(),
        break_inside: k2f_core::BreakInside::Avoid,
        content: NodeContent::Image {
            src: "assets/images/x.png".into(),
            width: Pt(160_000),
            height: Pt(20_000),
        },
        ..Default::default()
    };
    fig.column_span = ColumnSpan::All;

    let root = columns_root(vec![
        make_text("intro", "Short intro before the figure."),
        fig,
        make_text("after", "Text after the figure continues."),
    ]);
    let page_config = page(Pt(200_000), Pt(400_000));
    let content_w = page_config.width - page_config.margin[1] - page_config.margin[3];
    let manifest = Manifest {
        title: "span".into(),
        canvas_mode: CanvasMode::Paged,
        page_config: page_config.clone(),
        root,
        running_blocks: vec![],
    };
    let result = LayoutEngine::layout(&manifest, &ctx).unwrap();
    let fig_geo = result
        .pages
        .iter()
        .flat_map(|p| p.root.children.iter())
        .find(|c| c.id == "fig")
        .expect("fig");
    assert_eq!(fig_geo.x, page_config.margin[3]);
    // Stretch may expand the image to the content width.
    assert!(fig_geo.width >= Pt(160_000) && fig_geo.width <= content_w);

    let intro = result
        .pages
        .iter()
        .flat_map(|p| p.root.children.iter())
        .find(|c| c.id == "intro")
        .expect("intro");
    let after = result
        .pages
        .iter()
        .flat_map(|p| p.root.children.iter())
        .find(|c| c.id == "after")
        .expect("after");
    assert!(intro.y <= fig_geo.y);
    assert!(after.y >= fig_geo.y);
    assert!(intro.width < content_w);
    // Figure and surrounding column text share a page when content is short.
    let fig_page = result
        .pages
        .iter()
        .position(|p| p.root.children.iter().any(|c| c.id == "fig"))
        .unwrap();
    let intro_page = result
        .pages
        .iter()
        .position(|p| p.root.children.iter().any(|c| c.id == "intro"))
        .unwrap();
    assert_eq!(fig_page, intro_page);
}

#[test]
fn columns_layout_is_deterministic() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);
    let root = columns_root(vec![
        make_text("a", "Alpha text for columns."),
        make_text("b", "Bravo text for columns."),
        make_text("c", "Charlie text for columns."),
    ]);
    let page_config = page(Pt(200_000), Pt(300_000));
    let manifest = Manifest {
        title: "det".into(),
        canvas_mode: CanvasMode::Paged,
        page_config,
        root,
        running_blocks: vec![],
    };
    let a = LayoutEngine::layout(&manifest, &ctx).unwrap();
    let b = LayoutEngine::layout(&manifest, &ctx).unwrap();
    assert_eq!(a, b);
}

#[test]
fn nested_columns_rejected_by_validate() {
    let inner = SemanticNode {
        id: "inner".into(),
        role: "section".into(),
        layout: Some(LayoutHint::Columns { count: 2, gap: 0 }),
        content: NodeContent::Container {
            children: vec![make_text("p", "x")],
        },
        ..Default::default()
    };
    let outer = SemanticNode {
        id: "outer".into(),
        role: "section".into(),
        layout: Some(LayoutHint::Columns { count: 2, gap: 0 }),
        content: NodeContent::Container {
            children: vec![inner],
        },
        ..Default::default()
    };
    let err = k2f_core::validate_semantic_tree(&outer).unwrap_err();
    assert!(matches!(err, k2f_core::K2FError::ColumnsNested { .. }));
}
