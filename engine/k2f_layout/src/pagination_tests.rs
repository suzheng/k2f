use crate::Theme;
use crate::{measure_node, Size, SizeConstraint};
use crate::{LayoutContext, LayoutEngine};
use k2f_core::{
    CanvasMode, Manifest, NodeContent, PageConfig, Pt, RunningBlockNode, RunningBlockPosition,
    SemanticNode,
};

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

// Make a container with N text nodes
fn make_content(count: usize) -> SemanticNode {
    let mut children = Vec::new();
    for i in 0..count {
        children.push(make_text(&format!("t{}", i), "Line"));
    }
    SemanticNode {
        id: "root".to_string(),
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

#[test]
fn test_pagination_breaks() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    // 1. Create content: 10 lines.
    let root = make_content(10);

    // 2. Compute a deterministic single-line height, then choose a page height
    // that fits exactly 4 items per page (so the expected page count is stable).
    let margins = [Pt(10_000), Pt(7_000), Pt(15_000), Pt(11_000)]; // top, right, bottom, left
    let width = Pt(200_000);
    let tmp_page_config = PageConfig {
        width,
        height: Pt(1_000_000), // placeholder for measurement only
        margin: margins,
    };
    let content_width = crate::pagination::content_width(&tmp_page_config);
    let sample = make_text("sample", "Line");
    let sample_size = measure_node(
        &sample,
        SizeConstraint::new(Size::ZERO, Size::new(content_width, Pt(i128::MAX))),
        &ctx,
    )
    .unwrap();
    let line_h = sample_size.height;

    let items_per_page: i128 = 4;
    let page_height = margins[0] + margins[2] + (line_h * items_per_page);
    let page_config = PageConfig {
        width,
        height: page_height,
        margin: margins,
    };

    let manifest = Manifest {
        title: "Test".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: page_config.clone(),
        root,
        running_blocks: vec![],
    };

    let result = LayoutEngine::layout(&manifest, &ctx).unwrap();

    assert_eq!(
        result.pages.len(),
        3,
        "Expected 3 pages for 10 items @ 4 per page"
    );

    // Verify page/root dimensions align with PageConfig.
    for page in &result.pages {
        assert_eq!(page.width, page_config.width);
        assert_eq!(page.height, page_config.height);
        assert_eq!(page.root.width, page_config.width);
        assert_eq!(page.root.height, page_config.height);
    }

    // Verify explicit distribution (4,4,2) and margin offsets.
    assert_eq!(result.pages[0].root.children.len(), 4);
    assert_eq!(result.pages[1].root.children.len(), 4);
    assert_eq!(result.pages[2].root.children.len(), 2);

    let first = &result.pages[0].root.children[0];
    assert_eq!(first.x, margins[3]);
    assert_eq!(first.y, margins[0]);

    let second = &result.pages[0].root.children[1];
    assert_eq!(second.x, margins[3]);
    assert_eq!(second.y, margins[0] + line_h);

    let first_page_2 = &result.pages[1].root.children[0];
    assert_eq!(first_page_2.x, margins[3]);
    assert_eq!(first_page_2.y, margins[0]);
}

#[test]
fn test_infinite_canvas() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let root = make_content(10); // ~120pt total

    // Infinite mode should result in 1 page; height must expand deterministically.
    let margins = [Pt(10_000), Pt(7_000), Pt(15_000), Pt(11_000)]; // top, right, bottom, left
    let width = Pt(200_000);
    let tmp_page_config = PageConfig {
        width,
        height: Pt(1_000_000), // placeholder for measurement only
        margin: margins,
    };
    let content_width = crate::pagination::content_width(&tmp_page_config);
    let sample = make_text("sample", "Line");
    let sample_size = measure_node(
        &sample,
        SizeConstraint::new(Size::ZERO, Size::new(content_width, Pt(i128::MAX))),
        &ctx,
    )
    .unwrap();
    let line_h = sample_size.height;

    // Intentionally small initial height (fits only 2 items) to force expansion.
    let page_config = PageConfig {
        width,
        height: margins[0] + margins[2] + (line_h * 2),
        margin: margins,
    };

    let manifest = Manifest {
        title: "Test".to_string(),
        canvas_mode: CanvasMode::Infinite,
        page_config: page_config.clone(),
        root,
        running_blocks: vec![],
    };

    let result = LayoutEngine::layout(&manifest, &ctx).unwrap();

    assert_eq!(
        result.pages.len(),
        1,
        "Infinite mode should only have 1 page"
    );
    assert_eq!(result.pages[0].root.children.len(), 10);

    // Page/root dimensions must remain aligned after expansion.
    assert_eq!(result.pages[0].width, page_config.width);
    assert_eq!(result.pages[0].root.width, page_config.width);
    assert_eq!(result.pages[0].height, result.pages[0].root.height);

    let expected_height = margins[0] + (line_h * 10) + margins[2];
    assert_eq!(
        result.pages[0].height, expected_height,
        "Infinite page height should expand to exactly fit content + margins"
    );

    let first = &result.pages[0].root.children[0];
    let last = &result.pages[0].root.children[9];
    assert_eq!(first.x, margins[3]);
    assert_eq!(first.y, margins[0]);
    assert_eq!(last.y, margins[0] + (line_h * 9));
}

#[test]
fn test_running_blocks_injection_does_not_change_pagination() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    // Baseline: 10 lines -> (4,4,2) pagination as in test_pagination_breaks.
    let root = make_content(10);

    let margins = [Pt(10_000), Pt(7_000), Pt(15_000), Pt(11_000)]; // top, right, bottom, left
    let width = Pt(200_000);
    let tmp_page_config = PageConfig {
        width,
        height: Pt(1_000_000),
        margin: margins,
    };
    let content_width = crate::pagination::content_width(&tmp_page_config);
    let sample = make_text("sample", "Line");
    let sample_size = measure_node(
        &sample,
        SizeConstraint::new(Size::ZERO, Size::new(content_width, Pt(i128::MAX))),
        &ctx,
    )
    .unwrap();
    let line_h = sample_size.height;

    let items_per_page: i128 = 4;
    let page_height = margins[0] + margins[2] + (line_h * items_per_page);
    let page_config = PageConfig {
        width,
        height: page_height,
        margin: margins,
    };

    let baseline = Manifest {
        title: "Test".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: page_config.clone(),
        root: root.clone(),
        running_blocks: vec![],
    };
    let baseline_result = LayoutEngine::layout(&baseline, &ctx).unwrap();

    let with_footer = Manifest {
        title: "Test".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: page_config.clone(),
        root,
        running_blocks: vec![RunningBlockNode {
            position: RunningBlockPosition::Footer,
            node: make_text("rb_footer", "Page {{page_current}} of {{page_total}}"),
        }],
    };

    let result = LayoutEngine::layout(&with_footer, &ctx).unwrap();

    assert_eq!(result.pages.len(), baseline_result.pages.len());

    // Content distribution should remain identical, plus one extra child per page for the footer.
    assert_eq!(baseline_result.pages[0].root.children.len(), 4);
    assert_eq!(baseline_result.pages[1].root.children.len(), 4);
    assert_eq!(baseline_result.pages[2].root.children.len(), 2);

    assert_eq!(result.pages[0].root.children.len(), 5);
    assert_eq!(result.pages[1].root.children.len(), 5);
    assert_eq!(result.pages[2].root.children.len(), 3);

    // Footer positioning must stay within the bottom margin band, anchored at the band start.
    let footer_y_start = page_config.height - page_config.margin[2];
    for page in &result.pages {
        let footer = page.root.children.last().unwrap();
        assert_eq!(footer.id, "rb_footer");
        assert_eq!(footer.y, footer_y_start);
        assert!(footer.y >= footer_y_start && footer.y <= page.height);
        assert_eq!(footer.x, page_config.margin[3]);
    }
}

fn make_text_role(id: &str, role: &str, content: &str) -> SemanticNode {
    let mut n = make_text(id, content);
    n.role = role.to_string();
    n
}

#[test]
fn keep_with_next_moves_heading_with_body() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let sample = make_text("sample", "Line");
    let margins = [Pt(10_000), Pt(7_000), Pt(15_000), Pt(11_000)];
    let width = Pt(200_000);
    let tmp = PageConfig {
        width,
        height: Pt(1_000_000),
        margin: margins,
    };
    let cw = crate::pagination::content_width(&tmp);
    let line_h = measure_node(
        &sample,
        SizeConstraint::new(Size::ZERO, Size::new(cw, Pt(i128::MAX))),
        &ctx,
    )
    .unwrap()
    .height;

    // Page fits exactly 2 lines. Heading + body is 2 lines; with a filler first,
    // heading would orphan at the bottom without keep_with_next.
    let page_config = PageConfig {
        width,
        height: margins[0] + margins[2] + (line_h * 2),
        margin: margins,
    };

    let mut heading = make_text_role("h", "header", "Title");
    heading.keep_with_next = true;
    let body = make_text("b", "Body");
    let filler = make_text("f", "Fill");

    let root = SemanticNode {
        id: "root".to_string(),
        role: "section".to_string(),
        content: NodeContent::Container {
            children: vec![filler, heading, body],
        },
        ..Default::default()
    };
    let manifest = Manifest {
        title: "kwn".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config,
        root,
        running_blocks: vec![],
    };
    let result = LayoutEngine::layout(&manifest, &ctx).unwrap();
    assert_eq!(result.pages.len(), 2);
    let p0: Vec<_> = result.pages[0]
        .root
        .children
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    let p1: Vec<_> = result.pages[1]
        .root
        .children
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(p0, vec!["f"]);
    assert_eq!(p1, vec!["h", "b"]);
}

#[test]
fn avoid_block_taller_than_page_errors() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);
    let page_config = PageConfig {
        width: Pt(200_000),
        height: Pt(50_000),
        margin: [Pt(5_000); 4],
    };
    let mut img = SemanticNode {
        id: "logo".to_string(),
        role: "body".to_string(),
        content: NodeContent::Image {
            src: "assets/images/logo.png".to_string(),
            width: Pt(80_000),
            height: Pt(80_000),
        },
        ..Default::default()
    };
    img.break_inside = k2f_core::BreakInside::Avoid;
    let root = SemanticNode {
        id: "root".to_string(),
        role: "section".to_string(),
        content: NodeContent::Container {
            children: vec![img],
        },
        ..Default::default()
    };
    let manifest = Manifest {
        title: "ov".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config,
        root,
        running_blocks: vec![],
    };
    let err = LayoutEngine::layout(&manifest, &ctx).unwrap_err();
    assert!(err.contains("UNSPLITTABLE_OVERFLOW"), "got {err}");
}

#[test]
fn nested_unpadded_stack_splits_at_child_boundaries() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let sample = make_text("sample", "Line");
    let margins = [Pt(10_000), Pt(7_000), Pt(15_000), Pt(11_000)];
    let width = Pt(200_000);
    let tmp = PageConfig {
        width,
        height: Pt(1_000_000),
        margin: margins,
    };
    let cw = crate::pagination::content_width(&tmp);
    let line_h = measure_node(
        &sample,
        SizeConstraint::new(Size::ZERO, Size::new(cw, Pt(i128::MAX))),
        &ctx,
    )
    .unwrap()
    .height;
    let page_config = PageConfig {
        width,
        height: margins[0] + margins[2] + (line_h * 2),
        margin: margins,
    };

    let article = SemanticNode {
        id: "article".to_string(),
        role: "body".to_string(),
        content: NodeContent::Container {
            children: vec![
                make_text("a", "Line"),
                make_text("b", "Line"),
                make_text("c", "Line"),
            ],
        },
        ..Default::default()
    };
    let root = SemanticNode {
        id: "root".to_string(),
        role: "section".to_string(),
        content: NodeContent::Container {
            children: vec![article],
        },
        ..Default::default()
    };
    let manifest = Manifest {
        title: "nested".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config,
        root,
        running_blocks: vec![],
    };
    let result = LayoutEngine::layout(&manifest, &ctx).unwrap();
    assert_eq!(result.pages.len(), 2);
    let p0: Vec<_> = result.pages[0]
        .root
        .children
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    let p1: Vec<_> = result.pages[1]
        .root
        .children
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(p0, vec!["a", "b"]);
    assert_eq!(p1, vec!["c"]);
}

#[test]
fn avoid_signature_moves_as_one_block() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let sample = make_text("sample", "Line");
    let margins = [Pt(10_000), Pt(7_000), Pt(15_000), Pt(11_000)];
    let width = Pt(200_000);
    let tmp = PageConfig {
        width,
        height: Pt(1_000_000),
        margin: margins,
    };
    let cw = crate::pagination::content_width(&tmp);
    let line_h = measure_node(
        &sample,
        SizeConstraint::new(Size::ZERO, Size::new(cw, Pt(i128::MAX))),
        &ctx,
    )
    .unwrap()
    .height;
    let page_config = PageConfig {
        width,
        height: margins[0] + margins[2] + (line_h * 2),
        margin: margins,
    };

    let mut signature = SemanticNode {
        id: "signatures".to_string(),
        role: "body".to_string(),
        content: NodeContent::Container {
            children: vec![make_text("sig_a", "Line"), make_text("sig_b", "Line")],
        },
        ..Default::default()
    };
    signature.break_inside = k2f_core::BreakInside::Avoid;
    let root = SemanticNode {
        id: "root".to_string(),
        role: "section".to_string(),
        content: NodeContent::Container {
            children: vec![make_text("f", "Line"), signature],
        },
        ..Default::default()
    };
    let manifest = Manifest {
        title: "sig".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config,
        root,
        running_blocks: vec![],
    };
    let result = LayoutEngine::layout(&manifest, &ctx).unwrap();
    assert_eq!(result.pages.len(), 2);
    let p0: Vec<_> = result.pages[0]
        .root
        .children
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    let p1: Vec<_> = result.pages[1]
        .root
        .children
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(p0, vec!["f"]);
    assert_eq!(p1, vec!["signatures"]);
    assert_eq!(result.pages[1].root.children[0].children.len(), 2);
}

#[test]
fn splittable_body_text_uses_remaining_page_space() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let sample = make_text("sample", "Line");
    let margins = [Pt(10_000), Pt(7_000), Pt(15_000), Pt(11_000)];
    let width = Pt(200_000);
    let tmp = PageConfig {
        width,
        height: Pt(1_000_000),
        margin: margins,
    };
    let cw = crate::pagination::content_width(&tmp);
    let line_h = measure_node(
        &sample,
        SizeConstraint::new(Size::ZERO, Size::new(cw, Pt(i128::MAX))),
        &ctx,
    )
    .unwrap()
    .height;

    // Five body lines fit on one page; filler consumes four.
    let page_config = PageConfig {
        width,
        height: margins[0] + margins[2] + (line_h * 5),
        margin: margins,
    };

    let filler = make_text("f", "A\nB\nC\nD");
    let long_body = make_text(
        "para",
        "word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word",
    );
    let root = SemanticNode {
        id: "root".to_string(),
        role: "section".to_string(),
        content: NodeContent::Container {
            children: vec![filler, long_body],
        },
        ..Default::default()
    };
    let manifest = Manifest {
        title: "split".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config,
        root,
        running_blocks: vec![],
    };
    let result = LayoutEngine::layout(&manifest, &ctx).unwrap();
    assert_eq!(result.pages.len(), 2, "paragraph should split across pages");
    assert!(
        !result.pages[0].root.children.is_empty(),
        "page 0 should contain filler and/or paragraph fragment"
    );
    assert!(
        !result.pages[1].root.children.is_empty(),
        "page 1 should contain paragraph continuation"
    );
}

#[test]
fn break_before_page_starts_on_fresh_page() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);
    let page_config = PageConfig {
        width: Pt(200_000),
        height: Pt(400_000),
        margin: [Pt(10_000); 4],
    };
    let mut second = make_text("b", "Second");
    second.break_before = k2f_core::BreakBefore::Page;
    let root = SemanticNode {
        id: "root".to_string(),
        role: "section".to_string(),
        content: NodeContent::Container {
            children: vec![make_text("a", "First"), second],
        },
        ..Default::default()
    };
    let manifest = Manifest {
        title: "bb".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config,
        root,
        running_blocks: vec![],
    };
    let result = LayoutEngine::layout(&manifest, &ctx).unwrap();
    assert_eq!(result.pages.len(), 2);
    assert_eq!(result.pages[0].root.children.len(), 1);
    assert_eq!(result.pages[0].root.children[0].id, "a");
    assert_eq!(result.pages[1].root.children[0].id, "b");
}
