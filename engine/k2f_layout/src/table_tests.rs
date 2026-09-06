use crate::{arrange_node, measure_node, LayoutContext, Point, Size, SizeConstraint, Theme};
use k2f_core::{GridTrack, NodeContent, Pt, SemanticNode, TableDataSource, TableSpec};

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

#[test]
fn test_strict_table_resolves_columns_with_fr_and_arranges_cells_with_gap() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    // 3 columns: fr(1), pt(50), fr(1) with 1pt gap between columns.
    // Total available includes gaps.
    let gap = 1_000_i64;
    let total_w = Pt(252_000); // 250pt for tracks + 2pt gaps (3 cols -> 2 gaps)

    let table = SemanticNode {
        id: "tbl".to_string(),
        role: "table".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Table(TableSpec {
            column_widths: vec![
                GridTrack::Fr { fr: 1 },
                GridTrack::Pt { pt: 50_000 },
                GridTrack::Fr { fr: 1 },
            ],
            header_rows: 0,
            gap,
            data: TableDataSource::Inline {
                rows: vec![vec![
                    make_image("c0", 10_000, 20_000),
                    make_image("c1", 10_000, 40_000),
                    make_image("c2", 10_000, 30_000),
                ]],
            },
        }),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    let constraint = SizeConstraint::new(Size::ZERO, Size::new(total_w, Pt(i128::MAX)));
    let measured = measure_node(&table, constraint, &ctx).unwrap();
    assert_eq!(measured.width, total_w);
    // Single row height = max cell height (40pt)
    assert_eq!(measured.height, Pt(40_000));

    let start = Point::new(Pt(10_000), Pt(20_000));
    let geo = arrange_node(&table, start, measured, &ctx).unwrap();
    assert_eq!(geo.children.len(), 3);

    // With total_w=252pt and gap=1pt:
    // available_for_tracks = 250pt, fixed=50pt, remaining=200pt => each fr = 100pt.
    let col0 = Pt(100_000);
    let col1 = Pt(50_000);
    let gap_pt = Pt(gap as i128);

    assert_eq!(geo.children[0].x, start.x);
    assert_eq!(geo.children[0].y, start.y);
    assert_eq!(geo.children[1].x, start.x + col0 + gap_pt);
    assert_eq!(geo.children[1].y, start.y);
    assert_eq!(geo.children[2].x, start.x + col0 + gap_pt + col1 + gap_pt);
    assert_eq!(geo.children[2].y, start.y);

    // v1: cells are stretched to full cell rect.
    assert_eq!(geo.children[0].width, col0);
    assert_eq!(geo.children[1].width, col1);
    assert_eq!(geo.children[2].width, col0);
    assert_eq!(geo.children[0].height, Pt(40_000));
}

#[test]
fn test_strict_table_row_heights_are_max_cell_height_and_y_positions_include_gap() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    let gap = 2_000_i64; // 2pt
    let gap_pt = Pt(gap as i128);

    let table = SemanticNode {
        id: "tbl".to_string(),
        role: "table".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Table(TableSpec {
            column_widths: vec![GridTrack::Pt { pt: 100_000 }, GridTrack::Pt { pt: 100_000 }],
            header_rows: 0,
            gap,
            data: TableDataSource::Inline {
                rows: vec![
                    vec![
                        make_image("r0c0", 10_000, 30_000),
                        make_image("r0c1", 10_000, 50_000),
                    ],
                    vec![
                        make_image("r1c0", 10_000, 20_000),
                        make_image("r1c1", 10_000, 40_000),
                    ],
                ],
            },
        }),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    // Total width includes 1 gap between 2 columns.
    let total_w = Pt(100_000 + 100_000 + gap_pt.0);
    let constraint = SizeConstraint::new(Size::ZERO, Size::new(total_w, Pt(i128::MAX)));
    let measured = measure_node(&table, constraint, &ctx).unwrap();

    let row0_h = Pt(50_000);
    let row1_h = Pt(40_000);
    assert_eq!(measured.height, Pt(row0_h.0 + gap_pt.0 + row1_h.0));

    let start = Point::new(Pt(0), Pt(0));
    let geo = arrange_node(&table, start, measured, &ctx).unwrap();
    assert_eq!(geo.children.len(), 4);

    // Child order is row-major: r0c0, r0c1, r1c0, r1c1
    assert_eq!(geo.children[0].y, Pt(0));
    assert_eq!(geo.children[1].y, Pt(0));
    assert_eq!(geo.children[2].y, row0_h + gap_pt);
    assert_eq!(geo.children[3].y, row0_h + gap_pt);

    assert_eq!(geo.children[0].height, row0_h);
    assert_eq!(geo.children[1].height, row0_h);
    assert_eq!(geo.children[2].height, row1_h);
    assert_eq!(geo.children[3].height, row1_h);
}

#[test]
fn table_cell_self_align_centers_short_child_in_tall_row() {
    let fonts = crate::test_utils::test_fonts();
    let theme: crate::Theme = serde_json::from_str(
        r#"{
      "palette": {},
      "roles": {
        "table": {
          "font_family": "default", "font_size": 12000,
          "line_height_mult": 1200, "color": "black"
        },
        "mid": {
          "font_family": "default", "font_size": 12000,
          "line_height_mult": 1200, "color": "black",
          "self_align": "center"
        },
        "body": {
          "font_family": "default", "font_size": 12000,
          "line_height_mult": 1200, "color": "black"
        }
      }
    }"#,
    )
    .unwrap();
    let ctx = LayoutContext::new(&fonts, &theme);

    let mut short = make_image("short", 10_000, 10_000);
    short.role = "mid".to_string();
    let tall = make_image("tall", 10_000, 40_000);

    let table = SemanticNode {
        id: "tbl".to_string(),
        role: "table".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Table(TableSpec {
            column_widths: vec![GridTrack::Pt { pt: 50_000 }, GridTrack::Pt { pt: 50_000 }],
            header_rows: 0,
            gap: 0,
            data: TableDataSource::Inline {
                rows: vec![vec![short, tall]],
            },
        }),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    let constraint = SizeConstraint::new(Size::ZERO, Size::new(Pt(100_000), Pt(i128::MAX)));
    let measured = measure_node(&table, constraint, &ctx).unwrap();
    assert_eq!(measured.height, Pt(40_000));
    let geo = arrange_node(&table, Point::ZERO, measured, &ctx).unwrap();
    assert_eq!(geo.children[0].height, Pt(10_000));
    assert_eq!(geo.children[0].y, Pt(15_000));
    assert_eq!(geo.children[1].height, Pt(40_000));
    assert_eq!(geo.children[1].y, Pt(0));
}
