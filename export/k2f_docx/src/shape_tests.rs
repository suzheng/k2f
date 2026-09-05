use crate::ir::LineDash;
use crate::shape::{round_rect_adj, shapes_from_box};
use crate::DocxError;
use k2f_core::{
    Blur, BlurRef, Border, BorderEdge, BorderStyle, BoxDecoration, Fill, FillRef, GradientStop,
    LinearGradient, Pt, Rect, Shadow, ShadowRef,
};

fn rect() -> Rect {
    Rect {
        x: Pt(10_000),
        y: Pt(20_000),
        width: Pt(100_000),
        height: Pt(50_000),
    }
}

fn page() -> (Pt, Pt) {
    (Pt(595_000), Pt(842_000))
}

#[test]
fn skips_shadow_rule_alpha_empty() {
    let (w, h) = page();
    let mut dec = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#FFFFFF".into(),
        })),
        shadow: Some(ShadowRef::Inline(Shadow { layers: vec![] })),
        ..Default::default()
    };
    assert!(shapes_from_box("card", &rect(), &dec, w, h, 0)
        .unwrap()
        .is_empty());
    dec.shadow = None;
    assert!(shapes_from_box("eq::rule_1", &rect(), &dec, w, h, 0)
        .unwrap()
        .is_empty());
    dec.background = Some(FillRef::Inline(Fill::Solid {
        color: "#FFFFFF80".into(),
    }));
    assert!(shapes_from_box("glass", &rect(), &dec, w, h, 0)
        .unwrap()
        .is_empty());
    assert!(
        shapes_from_box("empty", &rect(), &BoxDecoration::default(), w, h, 0)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn unresolved_fill_ref_fails() {
    let (w, h) = page();
    let dec = BoxDecoration {
        background: Some(FillRef::Ref("missing_surface".into())),
        ..Default::default()
    };
    let err = shapes_from_box("card", &rect(), &dec, w, h, 0).unwrap_err();
    match err {
        DocxError::Write(msg) => assert!(msg.contains("unresolved fill ref")),
        other => panic!("expected Write, got {other:?}"),
    }
}

#[test]
fn adj_clamped() {
    assert_eq!(round_rect_adj(0, 10_000, 10_000), 0);
    assert_eq!(round_rect_adj(1_000, 10_000, 20_000), 10_000);
    assert_eq!(round_rect_adj(10_000, 10_000, 10_000), 50_000);
}

#[test]
fn partial_border_emits_edge_bars_not_four_sided_ln() {
    let (w, h) = page();
    let dec = BoxDecoration {
        border: Some(Border {
            width_pt: 1_000,
            color: "#111111".into(),
            edges: vec![BorderEdge::Bottom],
            style: BorderStyle::Solid,
        }),
        ..Default::default()
    };
    let boxes = shapes_from_box("rule", &rect(), &dec, w, h, 10).unwrap();
    assert_eq!(boxes.len(), 1);
    assert!(boxes[0].node_id.ends_with("::edge_bottom"));
    assert_eq!(boxes[0].fill_hex.as_deref(), Some("111111"));
    assert!(boxes[0].line_hex.is_none());
}

fn four_edge(style: BorderStyle) -> Border {
    Border {
        width_pt: 1_000,
        color: "#FF0000".into(),
        edges: vec![
            BorderEdge::Top,
            BorderEdge::Right,
            BorderEdge::Bottom,
            BorderEdge::Left,
        ],
        style,
    }
}

#[test]
fn skips_gradient_and_blur() {
    let (w, h) = page();
    let grad = BoxDecoration {
        background: Some(FillRef::Inline(Fill::LinearGradient {
            value: LinearGradient::Linear {
                angle_degrees: 90,
                stops: vec![
                    GradientStop {
                        pos: 0,
                        color: "#000000".into(),
                    },
                    GradientStop {
                        pos: 1000,
                        color: "#FFFFFF".into(),
                    },
                ],
            },
        })),
        ..Default::default()
    };
    assert!(shapes_from_box("card", &rect(), &grad, w, h, 0)
        .unwrap()
        .is_empty());
    let blur = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#FFFFFF".into(),
        })),
        blur: Some(BlurRef::Inline(Blur { radius_pt: 8_000 })),
        ..Default::default()
    };
    assert!(shapes_from_box("glass", &rect(), &blur, w, h, 0)
        .unwrap()
        .is_empty());
}

#[test]
fn four_solid_edges_use_ln() {
    let (w, h) = page();
    let dec = BoxDecoration {
        border: Some(four_edge(BorderStyle::Solid)),
        ..Default::default()
    };
    assert!(dec.border.as_ref().unwrap().is_full_rect_stroke());
    let boxes = shapes_from_box("card", &rect(), &dec, w, h, 10).unwrap();
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].line_hex.as_deref(), Some("FF0000"));
    assert_eq!(boxes[0].line_dash, LineDash::Solid);
    assert!(!boxes[0].node_id.contains("::edge_"));
}

#[test]
fn four_dashed_edges_use_ln_not_bars() {
    let (w, h) = page();
    let border = four_edge(BorderStyle::Dashed);
    assert!(!border.is_full_rect_stroke());
    let dec = BoxDecoration {
        border: Some(border),
        ..Default::default()
    };
    let boxes = shapes_from_box("card", &rect(), &dec, w, h, 10).unwrap();
    assert_eq!(
        boxes.len(),
        1,
        "dashed rectangle must stay one a:ln, not four bars"
    );
    assert_eq!(boxes[0].line_dash, LineDash::Dash);
    assert_eq!(boxes[0].line_hex.as_deref(), Some("FF0000"));
}

#[test]
fn page_background_is_behind_doc() {
    let (w, h) = page();
    let full = Rect {
        x: Pt(0),
        y: Pt(0),
        width: w,
        height: h,
    };
    let dec = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#FFFFFF".into(),
        })),
        ..Default::default()
    };
    let bg = shapes_from_box("root::page_0::background", &full, &dec, w, h, 0).unwrap();
    assert_eq!(bg.len(), 1);
    assert!(bg[0].behind_doc);
    let cell = shapes_from_box("invoice.th.item", &rect(), &dec, w, h, 10).unwrap();
    assert_eq!(cell.len(), 1);
    assert!(!cell[0].behind_doc);
}

#[test]
fn corner_radius_recorded() {
    let (w, h) = page();
    let dec = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#1A73E8".into(),
        })),
        corner_radius_pt: Some(8_000),
        ..Default::default()
    };
    let boxes = shapes_from_box("card", &rect(), &dec, w, h, 0).unwrap();
    assert_eq!(boxes[0].corner_emu, crate::coord::millipt_to_emu(8_000));
}
