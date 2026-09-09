use crate::slack::{layout_slack_diags, LayoutDiagKind};
use crate::Theme;
use crate::{LayoutContext, LayoutEngine};
use k2f_core::{
    BreakBefore, CanvasMode, FixedSizeHint, GridTrack, LayoutHint, Manifest, NodeContent,
    PageConfig, Pt, SemanticNode, StackDirection,
};

fn fonts_ctx() -> (k2f_text::FontLibrary, Theme) {
    (crate::test_utils::test_fonts(), Theme::default())
}

fn page_a4() -> PageConfig {
    PageConfig {
        width: Pt(595_000),
        height: Pt(842_000),
        margin: [Pt::ZERO; 4],
    }
}

fn image(id: &str, h: i128) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "body".to_string(),
        content: NodeContent::Image {
            src: id.to_string(),
            width: Pt(100_000),
            height: Pt(h),
        },
        ..Default::default()
    }
}

fn wrap_children(children: Vec<SemanticNode>) -> Manifest {
    Manifest {
        title: "slack".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: page_a4(),
        root: SemanticNode {
            id: "root".to_string(),
            role: "document".to_string(),
            content: NodeContent::Container { children },
            ..Default::default()
        },
        running_blocks: vec![],
    }
}

fn wrap_root(child: SemanticNode) -> Manifest {
    wrap_children(vec![child])
}

fn page_shell(body: SemanticNode) -> SemanticNode {
    SemanticNode {
        id: "doc.shell".to_string(),
        role: "section".to_string(),
        content: NodeContent::Container {
            children: vec![
                image("doc.header", 40_000),
                body,
                image("doc.footer", 32_000),
            ],
        },
        layout: Some(LayoutHint::Grid {
            columns: vec![GridTrack::Fr { fr: 1 }],
            rows: vec![
                GridTrack::Pt { pt: 40_000 },
                GridTrack::Fr { fr: 1 },
                GridTrack::Pt { pt: 32_000 },
            ],
            gap: 8_000,
            row_gap: None,
            column_gap: None,
            cell_align: None,
            size: FixedSizeHint {
                width: Some(Pt(595_000)),
                height: Some(Pt(842_000)),
            },
        }),
        ..Default::default()
    }
}

#[test]
fn pinned_stack_shell_reports_bottom_slack() {
    let (fonts, theme) = fonts_ctx();
    let ctx = LayoutContext::new(&fonts, &theme);
    let shell = SemanticNode {
        id: "doc.shell".to_string(),
        role: "section".to_string(),
        content: NodeContent::Container {
            children: vec![image("doc.hero", 20_000)],
        },
        layout: Some(LayoutHint::Stack {
            direction: StackDirection::Vertical,
            gap: 0,
            align_items: Default::default(),
            justify_content: Default::default(),
            size: FixedSizeHint {
                width: Some(Pt(595_000)),
                height: Some(Pt(842_000)),
            },
        }),
        ..Default::default()
    };
    let manifest = wrap_root(shell);
    let layout = LayoutEngine::layout(&manifest, &ctx).unwrap();
    let diags = layout_slack_diags(&manifest, &layout, &theme);
    assert_eq!(diags.len(), 1, "{diags:?}");
    assert_eq!(diags[0].node_id, "doc.shell");
    assert!(diags[0].unused_below.0 > 700_000, "{}", diags[0]);
    let line = diags[0].to_string();
    assert!(line.starts_with("LAYOUT_SLACK id=doc.shell"), "{line}");
    assert!(
        line.contains("hint=nest {fr:1} in the grower; do not pack an auto-height stack"),
        "{line}"
    );
}

#[test]
fn fr_grower_image_leaf_does_not_report_slack() {
    let (fonts, theme) = fonts_ctx();
    let ctx = LayoutContext::new(&fonts, &theme);
    let manifest = wrap_root(page_shell(image("doc.body", 20_000)));
    let layout = LayoutEngine::layout(&manifest, &ctx).unwrap();
    let diags = layout_slack_diags(&manifest, &layout, &theme);
    assert!(diags.is_empty(), "{diags:?}");
}

#[test]
fn fr_grower_auto_stack_reports_slack() {
    let (fonts, theme) = fonts_ctx();
    let ctx = LayoutContext::new(&fonts, &theme);
    let body = SemanticNode {
        id: "doc.body".to_string(),
        role: "section".to_string(),
        content: NodeContent::Container {
            children: vec![image("doc.body.hero", 20_000)],
        },
        layout: Some(LayoutHint::Stack {
            direction: StackDirection::Vertical,
            gap: 0,
            align_items: Default::default(),
            justify_content: Default::default(),
            size: FixedSizeHint::default(),
        }),
        ..Default::default()
    };
    let manifest = wrap_root(page_shell(body));
    let layout = LayoutEngine::layout(&manifest, &ctx).unwrap();
    let diags = layout_slack_diags(&manifest, &layout, &theme);
    assert_eq!(diags.len(), 1, "{diags:?}");
    assert_eq!(diags[0].node_id, "doc.body");
    assert!(diags[0].unused_below.0 > 500_000, "{}", diags[0]);
}

#[test]
fn short_grid_demo_is_below_height_threshold() {
    let (fonts, theme) = fonts_ctx();
    let ctx = LayoutContext::new(&fonts, &theme);
    let grid = SemanticNode {
        id: "ex.grid".to_string(),
        role: "section".to_string(),
        content: NodeContent::Container {
            children: vec![image("a", 10_000), image("b", 10_000)],
        },
        layout: Some(LayoutHint::Grid {
            columns: vec![GridTrack::Fr { fr: 1 }],
            rows: vec![GridTrack::Fr { fr: 1 }, GridTrack::Pt { pt: 40_000 }],
            gap: 0,
            row_gap: None,
            column_gap: None,
            cell_align: None,
            size: FixedSizeHint {
                width: None,
                height: Some(Pt(120_000)),
            },
        }),
        ..Default::default()
    };
    let manifest = wrap_root(grid);
    let layout = LayoutEngine::layout(&manifest, &ctx).unwrap();
    let diags = layout_slack_diags(&manifest, &layout, &theme);
    assert!(
        diags.iter().all(|d| d.kind == LayoutDiagKind::PageUnderfill),
        "small grids must not warn LAYOUT_SLACK: {diags:?}"
    );
}

#[test]
fn one_page_short_stack_reports_page_underfill() {
    let (fonts, theme) = fonts_ctx();
    let ctx = LayoutContext::new(&fonts, &theme);
    let manifest = wrap_root(image("hero", 50_000));
    let layout = LayoutEngine::layout(&manifest, &ctx).unwrap();
    assert_eq!(layout.pages.len(), 1);
    let diags = layout_slack_diags(&manifest, &layout, &theme);
    let page = diags
        .iter()
        .find(|d| d.kind == LayoutDiagKind::PageUnderfill)
        .expect("{diags:?}");
    assert_eq!(page.page, Some(0));
    assert!(page.unused_below.0 > 700_000, "{}", page);
    let line = page.to_string();
    assert!(line.starts_with("PAGE_UNDERFILL page=0"), "{line}");
    assert!(
        line.contains("hint=one-page form: copy ex_filled_page.json"),
        "{line}"
    );
}

#[test]
fn flow_last_page_short_does_not_report_page_underfill() {
    let (fonts, theme) = fonts_ctx();
    let ctx = LayoutContext::new(&fonts, &theme);
    let manifest = wrap_children(vec![
        image("a", 400_000),
        image("b", 400_000),
        image("c", 50_000),
    ]);
    let layout = LayoutEngine::layout(&manifest, &ctx).unwrap();
    assert_eq!(layout.pages.len(), 2, "pages={}", layout.pages.len());
    let diags = layout_slack_diags(&manifest, &layout, &theme);
    assert!(
        diags
            .iter()
            .all(|d| d.kind != LayoutDiagKind::PageUnderfill),
        "full first page + short last page must be silent: {diags:?}"
    );
}

#[test]
fn break_before_underfilled_page_reports_page_underfill() {
    let (fonts, theme) = fonts_ctx();
    let ctx = LayoutContext::new(&fonts, &theme);
    let mut page_two = image("p2", 200_000);
    page_two.break_before = BreakBefore::Page;
    let manifest = wrap_children(vec![image("p1", 200_000), page_two]);
    let layout = LayoutEngine::layout(&manifest, &ctx).unwrap();
    assert_eq!(layout.pages.len(), 2);
    let diags = layout_slack_diags(&manifest, &layout, &theme);
    let pages: Vec<_> = diags
        .iter()
        .filter(|d| d.kind == LayoutDiagKind::PageUnderfill)
        .collect();
    assert_eq!(pages.len(), 1, "{diags:?}");
    assert_eq!(pages[0].page, Some(0));
    let line = pages[0].to_string();
    assert!(line.contains("hint=do not pre-paginate with break_before"), "{line}");
}
