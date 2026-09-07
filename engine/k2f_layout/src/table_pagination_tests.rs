use crate::{LayoutContext, LayoutEngine, Theme};
use k2f_core::{
    CanvasMode, GridTrack, Manifest, NodeContent, PageConfig, Pt, SemanticNode, TableDataSource,
    TableSpec,
};

fn make_image(id: &str, w: i128, h: i128) -> SemanticNode {
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

fn make_root(children: Vec<SemanticNode>) -> SemanticNode {
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
fn test_strict_table_paged_splits_on_row_boundaries() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    // 1 column, 5 rows, each row is a 30pt-tall image.
    let rows = vec![
        vec![make_image("r0", 10_000, 30_000)],
        vec![make_image("r1", 10_000, 30_000)],
        vec![make_image("r2", 10_000, 30_000)],
        vec![make_image("r3", 10_000, 30_000)],
        vec![make_image("r4", 10_000, 30_000)],
    ];

    let table = SemanticNode {
        id: "tbl".to_string(),
        role: "table".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Table(TableSpec {
            column_widths: vec![GridTrack::Pt { pt: 50_000 }],
            header_rows: 0,
            gap: 0,
            row_gap: None,
            column_gap: None,
            data: TableDataSource::Inline { rows },
        }),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    // Content height fits exactly 2 rows per page: 2 * 30pt = 60pt.
    let page_config = PageConfig {
        width: Pt(100_000),
        height: Pt(60_000),
        margin: [Pt(0), Pt(0), Pt(0), Pt(0)],
    };
    let manifest = Manifest {
        title: "Test".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: page_config.clone(),
        root: make_root(vec![table]),
        running_blocks: vec![],
    };

    let result = LayoutEngine::layout(&manifest, &ctx).unwrap();
    assert_eq!(result.pages.len(), 3);
    assert_eq!(result.pages[0].root.children.len(), 1);
    assert_eq!(result.pages[1].root.children.len(), 1);
    assert_eq!(result.pages[2].root.children.len(), 1);

    let frag0 = &result.pages[0].root.children[0];
    let frag1 = &result.pages[1].root.children[0];
    let frag2 = &result.pages[2].root.children[0];
    assert_eq!(frag0.id, "tbl");
    assert_eq!(frag1.id, "tbl");
    assert_eq!(frag2.id, "tbl");

    // Row distribution: 2, 2, 1
    assert_eq!(frag0.children.len(), 2);
    assert_eq!(frag1.children.len(), 2);
    assert_eq!(frag2.children.len(), 1);

    // Verify row-major order + y positions on the page.
    assert_eq!(frag0.children[0].id, "r0");
    assert_eq!(frag0.children[1].id, "r1");
    assert_eq!(frag0.children[0].y, Pt(0));
    assert_eq!(frag0.children[1].y, Pt(30_000));

    assert_eq!(frag1.children[0].id, "r2");
    assert_eq!(frag1.children[1].id, "r3");
    assert_eq!(frag1.children[0].y, Pt(0));
    assert_eq!(frag1.children[1].y, Pt(30_000));

    assert_eq!(frag2.children[0].id, "r4");
    assert_eq!(frag2.children[0].y, Pt(0));
}

#[test]
fn test_strict_table_paged_repeats_header_rows() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    // 1 header row + 4 body rows, each 30pt tall.
    let rows = vec![
        vec![make_image("h0", 10_000, 30_000)],
        vec![make_image("r1", 10_000, 30_000)],
        vec![make_image("r2", 10_000, 30_000)],
        vec![make_image("r3", 10_000, 30_000)],
        vec![make_image("r4", 10_000, 30_000)],
    ];

    let table = SemanticNode {
        id: "tbl".to_string(),
        role: "table".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Table(TableSpec {
            column_widths: vec![GridTrack::Pt { pt: 50_000 }],
            header_rows: 1,
            gap: 0,
            row_gap: None,
            column_gap: None,
            data: TableDataSource::Inline { rows },
        }),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    // Content height fits exactly 2 rows per page. With header repetition, each continuation
    // page holds 1 header row + 1 body row.
    let page_config = PageConfig {
        width: Pt(100_000),
        height: Pt(60_000),
        margin: [Pt(0), Pt(0), Pt(0), Pt(0)],
    };
    let manifest = Manifest {
        title: "Test".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: page_config.clone(),
        root: make_root(vec![table]),
        running_blocks: vec![],
    };

    let result = LayoutEngine::layout(&manifest, &ctx).unwrap();
    assert_eq!(result.pages.len(), 4);

    for (page_idx, page) in result.pages.iter().enumerate() {
        assert_eq!(page.root.children.len(), 1);
        let frag = &page.root.children[0];
        assert_eq!(frag.id, "tbl");
        assert_eq!(
            frag.children.len(),
            2,
            "page {} should contain 2 rows (header + 1 body)",
            page_idx
        );

        // First row is always the header row (repeated after the first page too).
        assert_eq!(frag.children[0].id, "h0");
    }

    // Verify the continued body rows on each page.
    assert_eq!(result.pages[0].root.children[0].children[1].id, "r1");
    assert_eq!(result.pages[1].root.children[0].children[1].id, "r2");
    assert_eq!(result.pages[2].root.children[0].children[1].id, "r3");
    assert_eq!(result.pages[3].root.children[0].children[1].id, "r4");
}
