use crate::Theme;
use crate::{arrange_node, measure_node, LayoutContext, Point, SizeConstraint};
use k2f_core::{
    Align, CellAlign, FixedSizeHint, GridTrack, JustifyContent, LayoutHint, NodeContent, Pt,
    SemanticNode, StackDirection,
};

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

#[test]
fn stack_justify_and_align_offsets_children_when_parent_has_extra_space() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    // Two 20pt-tall images with a 10pt gap => used_main = 50pt.
    // Parent inner height is 100pt => free = 50pt => justify center offset = 25pt.
    let a = make_image_sized("a", 30_000, 20_000);
    let b = make_image_sized("b", 30_000, 20_000);

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
        layout: Some(LayoutHint::Stack {
            direction: StackDirection::Vertical,
            gap: 10_000,
            align_items: Align::End,
            justify_content: JustifyContent::Center,
            size: FixedSizeHint {
                width: Some(Pt(100_000)),
                height: Some(Pt(100_000)),
            },
        }),
        ..Default::default()
    };

    let measured = measure_node(&container, SizeConstraint::infinite(), &ctx).unwrap();
    assert_eq!(measured.width, Pt(100_000));
    assert_eq!(measured.height, Pt(100_000));

    let geo = arrange_node(&container, Point::ZERO, measured, &ctx).unwrap();
    assert_eq!(geo.children.len(), 2);

    // align_items=end with inner width 100pt and child width 30pt => x offset = 70pt.
    assert_eq!(geo.children[0].x, Pt(70_000));
    assert_eq!(geo.children[1].x, Pt(70_000));

    // justify_content=center => start y = 25pt.
    assert_eq!(geo.children[0].y, Pt(25_000));
    // second child is after height(20) + gap(10)
    assert_eq!(geo.children[1].y, Pt(25_000) + Pt(20_000) + Pt(10_000));
}

#[test]
fn grid_cell_align_offsets_child_when_not_stretch() {
    let fonts = crate::test_utils::test_fonts();
    let theme = Theme::default();
    let ctx = LayoutContext::new(&fonts, &theme);

    // One cell: 100pt x 60pt. Child is 20pt x 10pt.
    let child = make_image_sized("img", 20_000, 10_000);
    let container = SemanticNode {
        id: "grid".to_string(),
        role: "section".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container {
            children: vec![child],
        },
        modifiers: vec![],
        layout: Some(LayoutHint::Grid {
            columns: vec![GridTrack::Pt { pt: 100_000 }],
            rows: vec![GridTrack::Pt { pt: 60_000 }],
            gap: 0,
            row_gap: None,
            column_gap: None,
            cell_align: Some(CellAlign {
                x: Align::Center,
                y: Align::End,
            }),
            size: Default::default(),
        }),
        ..Default::default()
    };

    let measured = measure_node(&container, SizeConstraint::infinite(), &ctx).unwrap();
    assert_eq!(measured.width, Pt(100_000));
    assert_eq!(measured.height, Pt(60_000));

    let geo = arrange_node(&container, Point::ZERO, measured, &ctx).unwrap();
    assert_eq!(geo.children.len(), 1);

    // x center: (100 - 20)/2 = 40
    assert_eq!(geo.children[0].x, Pt(40_000));
    // y end: (60 - 10) = 50
    assert_eq!(geo.children[0].y, Pt(50_000));
    assert_eq!(geo.children[0].width, Pt(20_000));
    assert_eq!(geo.children[0].height, Pt(10_000));
}
