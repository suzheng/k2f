use crate::grid::resolve_tracks;
use crate::Theme;
use crate::{arrange_node, measure_node, LayoutContext, Point, Size, SizeConstraint};
use k2f_core::{CellAlign, FixedSizeHint, GridTrack, LayoutHint, NodeContent, Pt, SemanticNode};

fn make_image(id: &str) -> SemanticNode {
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
            width: Pt(100000),
            height: Pt(100000),
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    }
}

#[test]
fn test_grid_fr_remainder_distribution_is_stable() {
    let tracks = vec![
        GridTrack::Fr { fr: 1 },
        GridTrack::Fr { fr: 1 },
        GridTrack::Fr { fr: 1 },
    ];

    // 10 units, 3 tracks -> 3,3,3 remainder 1 -> earliest fr gets +1
    let out = resolve_tracks(&tracks, Pt::ZERO, Pt(10)).unwrap();
    assert_eq!(out, vec![Pt(4), Pt(3), Pt(3)]);

    // Same, but with gaps: available includes gaps, so only (10 - 2*1)=8 to distribute.
    let out_with_gaps = resolve_tracks(&tracks, Pt(1), Pt(10)).unwrap();
    assert_eq!(out_with_gaps, vec![Pt(3), Pt(3), Pt(2)]);
}

#[test]
fn test_grid_known_sizes_produce_expected_child_positions_and_cell_sizes() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let gap = 1000_i64; // 1pt
    let gap_pt = Pt(gap as i128);
    let columns = vec![GridTrack::Pt { pt: 10000 }, GridTrack::Pt { pt: 20000 }]; // 10pt, 20pt
    let rows = vec![GridTrack::Pt { pt: 30000 }, GridTrack::Pt { pt: 40000 }]; // 30pt, 40pt

    let container = SemanticNode {
        id: "grid".to_string(),
        role: "section".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container {
            children: vec![
                make_image("a"),
                make_image("b"),
                make_image("c"),
                make_image("d"),
            ],
        },
        modifiers: vec![],
        layout: Some(LayoutHint::Grid {
            columns,
            rows,
            gap,
            row_gap: None,
            column_gap: None,
            cell_align: Some(CellAlign::default()),
            size: Default::default(),
        }),
        ..Default::default()
    };

    // Pt-only tracks don't require finite constraints.
    let measured = measure_node(&container, SizeConstraint::infinite(), &ctx).unwrap();
    let start = Point::new(Pt(50000), Pt(60000)); // (50pt, 60pt)
    let geo = arrange_node(&container, start, measured, &ctx).unwrap();
    assert_eq!(geo.children.len(), 4);

    // Cell (0,0)
    assert_eq!(geo.children[0].x, start.x);
    assert_eq!(geo.children[0].y, start.y);
    assert_eq!(geo.children[0].width, Pt(10000));
    assert_eq!(geo.children[0].height, Pt(30000));

    // Cell (1,0)
    assert_eq!(geo.children[1].x, start.x + Pt(10000) + gap_pt);
    assert_eq!(geo.children[1].y, start.y);
    assert_eq!(geo.children[1].width, Pt(20000));
    assert_eq!(geo.children[1].height, Pt(30000));

    // Cell (0,1)
    assert_eq!(geo.children[2].x, start.x);
    assert_eq!(geo.children[2].y, start.y + Pt(30000) + gap_pt);
    assert_eq!(geo.children[2].width, Pt(10000));
    assert_eq!(geo.children[2].height, Pt(40000));

    // Cell (1,1)
    assert_eq!(geo.children[3].x, start.x + Pt(10000) + gap_pt);
    assert_eq!(geo.children[3].y, start.y + Pt(30000) + gap_pt);
    assert_eq!(geo.children[3].width, Pt(20000));
    assert_eq!(geo.children[3].height, Pt(40000));
}

#[test]
fn test_grid_fr_tracks_account_for_gaps_and_constraints() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    // 2 columns fr(1), fr(2) with 1 gap. Total available width is 10.
    // Available for tracks = 10 - gap(1) = 9 => col0=3 col1=6.
    let gap = 1_i64;
    let gap_pt = Pt(gap as i128);
    let columns = vec![GridTrack::Fr { fr: 1 }, GridTrack::Fr { fr: 2 }];
    let rows = vec![GridTrack::Fr { fr: 1 }];

    let container = SemanticNode {
        id: "grid".to_string(),
        role: "section".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container {
            children: vec![make_image("a"), make_image("b")],
        },
        modifiers: vec![],
        layout: Some(LayoutHint::Grid {
            columns,
            rows,
            gap,
            row_gap: None,
            column_gap: None,
            cell_align: Some(CellAlign::default()),
            size: Default::default(),
        }),
        ..Default::default()
    };

    let constraint = SizeConstraint::new(Size::ZERO, Size::new(Pt(10), Pt(5)));
    let measured = measure_node(&container, constraint, &ctx).unwrap();
    assert_eq!(measured.width, Pt(10));
    assert_eq!(measured.height, Pt(5));

    let geo = arrange_node(&container, Point::ZERO, measured, &ctx).unwrap();
    assert_eq!(geo.children.len(), 2);

    assert_eq!(geo.children[0].x, Pt(0));
    assert_eq!(geo.children[0].width, Pt(3));

    assert_eq!(geo.children[1].x, Pt(3) + gap_pt);
    assert_eq!(geo.children[1].width, Pt(6));
}

#[test]
fn test_grid_own_height_resolves_fr_rows_in_unbounded_flow() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    // Without fixed height, fr rows fail in unbounded vertical flow.
    let unbounded = LayoutHint::Grid {
        columns: vec![GridTrack::Pt { pt: 100_000 }],
        rows: vec![GridTrack::Fr { fr: 1 }, GridTrack::Pt { pt: 40_000 }],
        gap: 0,
        row_gap: None,
        column_gap: None,
        cell_align: None,
        size: FixedSizeHint::default(),
    };
    let fail_node = SemanticNode {
        id: "g_fail".into(),
        role: "body".into(),
        content: NodeContent::Container {
            children: vec![make_image("a"), make_image("b")],
        },
        layout: Some(unbounded),
        ..Default::default()
    };
    let err = measure_node(&fail_node, SizeConstraint::infinite(), &ctx).unwrap_err();
    assert!(
        err.contains("Cannot resolve fr tracks"),
        "expected fr failure, got {err}"
    );

    // With grid.height set, fr rows resolve against that outer height.
    let fixed = LayoutHint::Grid {
        columns: vec![GridTrack::Pt { pt: 100_000 }],
        rows: vec![GridTrack::Fr { fr: 1 }, GridTrack::Pt { pt: 40_000 }],
        gap: 0,
        row_gap: None,
        column_gap: None,
        cell_align: Some(CellAlign::default()),
        size: FixedSizeHint {
            width: None,
            height: Some(Pt(200_000)),
        },
    };
    let ok_node = SemanticNode {
        id: "g_ok".into(),
        role: "body".into(),
        content: NodeContent::Container {
            children: vec![make_image("a"), make_image("b")],
        },
        layout: Some(fixed),
        ..Default::default()
    };
    let measured = measure_node(&ok_node, SizeConstraint::infinite(), &ctx).unwrap();
    assert_eq!(measured.width, Pt(100_000));
    assert_eq!(measured.height, Pt(200_000));

    let geo = arrange_node(&ok_node, Point::ZERO, measured, &ctx).unwrap();
    assert_eq!(geo.children.len(), 2);
    // fr row gets remaining 160000; pt footer is 40000
    assert_eq!(geo.children[0].height, Pt(160_000));
    assert_eq!(geo.children[1].y, Pt(160_000));
    assert_eq!(geo.children[1].height, Pt(40_000));
}

#[test]
fn test_grid_auto_fr_auto_rows_fill_fixed_height() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let header = make_image("h");
    let body = make_image("b");
    let footer = make_image("f");
    let container = SemanticNode {
        id: "g".to_string(),
        role: "section".to_string(),
        content: NodeContent::Container {
            children: vec![header, body, footer],
        },
        layout: Some(LayoutHint::Grid {
            columns: vec![GridTrack::Fr { fr: 1 }],
            rows: vec![
                GridTrack::Auto { auto: true },
                GridTrack::Fr { fr: 1 },
                GridTrack::Auto { auto: true },
            ],
            gap: 0,
            row_gap: None,
            column_gap: None,
            cell_align: Some(CellAlign::default()),
            size: FixedSizeHint {
                width: Some(Pt(100_000)),
                height: Some(Pt(400_000)),
            },
        }),
        ..Default::default()
    };

    let measured = measure_node(&container, SizeConstraint::infinite(), &ctx).unwrap();
    assert_eq!(measured.height, Pt(400_000));
    let geo = arrange_node(&container, Point::ZERO, measured, &ctx).unwrap();
    assert_eq!(geo.children.len(), 3);
    // images are 100000 tall; auto rows take that; fr gets 200000
    assert_eq!(geo.children[0].height, Pt(100_000));
    assert_eq!(geo.children[1].height, Pt(200_000));
    assert_eq!(geo.children[2].height, Pt(100_000));
    assert_eq!(geo.children[2].y, Pt(300_000));
}

#[test]
fn test_resolve_tracks_rejects_auto_without_intrinsics() {
    let err = resolve_tracks(&[GridTrack::Auto { auto: true }], Pt::ZERO, Pt(100)).unwrap_err();
    assert!(err.contains("auto"));
}

#[test]
fn test_grid_auto_rows_sum_when_height_unbounded() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);
    let container = SemanticNode {
        id: "g".to_string(),
        role: "section".to_string(),
        content: NodeContent::Container {
            children: vec![make_image("a"), make_image("b")],
        },
        layout: Some(LayoutHint::Grid {
            columns: vec![GridTrack::Fr { fr: 1 }],
            rows: vec![
                GridTrack::Auto { auto: true },
                GridTrack::Auto { auto: true },
            ],
            gap: 0,
            row_gap: None,
            column_gap: None,
            cell_align: Some(CellAlign::default()),
            size: FixedSizeHint {
                width: Some(Pt(100_000)),
                height: None,
            },
        }),
        ..Default::default()
    };
    let measured = measure_node(&container, SizeConstraint::infinite(), &ctx).unwrap();
    assert_eq!(measured.height, Pt(200_000));
}

#[test]
fn test_grid_auto_plus_fr_fails_when_height_unbounded() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);
    let container = SemanticNode {
        id: "g".to_string(),
        role: "section".to_string(),
        content: NodeContent::Container {
            children: vec![make_image("a"), make_image("b")],
        },
        layout: Some(LayoutHint::Grid {
            columns: vec![GridTrack::Pt { pt: 100_000 }],
            rows: vec![GridTrack::Auto { auto: true }, GridTrack::Fr { fr: 1 }],
            gap: 0,
            row_gap: None,
            column_gap: None,
            cell_align: Some(CellAlign::default()),
            size: Default::default(),
        }),
        ..Default::default()
    };
    let err = measure_node(&container, SizeConstraint::infinite(), &ctx).unwrap_err();
    assert!(err.contains("infinite available size"));
}
