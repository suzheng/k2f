use crate::slack::layout_slack_diags;
use crate::Theme;
use crate::{LayoutContext, LayoutEngine};
use k2f_core::{
    CanvasMode, FixedSizeHint, GridTrack, LayoutHint, Manifest, NodeContent, PageConfig, Pt,
    SemanticNode, StackDirection,
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

fn wrap_root(child: SemanticNode) -> Manifest {
    Manifest {
        title: "slack".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: page_a4(),
        root: SemanticNode {
            id: "root".to_string(),
            role: "document".to_string(),
            content: NodeContent::Container {
                children: vec![child],
            },
            ..Default::default()
        },
        running_blocks: vec![],
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
    assert!(line.contains("hint=use {fr:1} body row"), "{line}");
}

#[test]
fn fr_grower_grid_does_not_report_slack() {
    let (fonts, theme) = fonts_ctx();
    let ctx = LayoutContext::new(&fonts, &theme);
    let shell = SemanticNode {
        id: "doc.shell".to_string(),
        role: "section".to_string(),
        content: NodeContent::Container {
            children: vec![
                image("doc.header", 40_000),
                image("doc.body", 20_000),
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
    };
    let manifest = wrap_root(shell);
    let layout = LayoutEngine::layout(&manifest, &ctx).unwrap();
    let diags = layout_slack_diags(&manifest, &layout, &theme);
    assert!(diags.is_empty(), "{diags:?}");
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
    assert!(diags.is_empty(), "small grids must not warn: {diags:?}");
}
