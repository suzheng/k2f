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
    let shadowed = shapes_from_box("card", &rect(), &dec, w, h, 0, "FFFFFF").unwrap();
    assert_eq!(shadowed.len(), 1, "shadow-only boxes stay native fills");
    assert_eq!(shadowed[0].fill_hex.as_deref(), Some("FFFFFF"));
    dec.shadow = None;
    assert!(
        shapes_from_box("eq::rule_1", &rect(), &dec, w, h, 0, "FFFFFF")
            .unwrap()
            .is_empty()
    );
    dec.background = Some(FillRef::Inline(Fill::Solid {
        color: "#FFFFFF80".into(),
    }));
    let glass = shapes_from_box("glass", &rect(), &dec, w, h, 0, "FFFFFF").unwrap();
    assert_eq!(glass.len(), 1);
    assert_eq!(glass[0].fill_hex.as_deref(), Some("FFFFFF"));
    assert_eq!(glass[0].fill_alpha, 0x80);
    assert!(!glass[0].behind_doc);
    assert!(shapes_from_box(
        "empty",
        &rect(),
        &BoxDecoration::default(),
        w,
        h,
        0,
        "FFFFFF"
    )
    .unwrap()
    .is_empty());
}

#[test]
fn full_page_shadow_shell_is_native_fill_and_front_edge_bars() {
    let (w, h) = page();
    let full = Rect {
        x: Pt(0),
        y: Pt(0),
        width: w,
        height: h,
    };
    let dec = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#080A10".into(),
        })),
        border: Some(four_edge(BorderStyle::Solid)),
        shadow: Some(ShadowRef::Inline(Shadow { layers: vec![] })),
        ..Default::default()
    };
    assert!(!crate::effect::box_is_effect("card.front", &dec).unwrap());
    let boxes = shapes_from_box("card.front", &full, &dec, w, h, 0, "080A10").unwrap();
    assert_eq!(boxes.len(), 5, "page wash + 4 edge bars, got {boxes:?}");
    assert!(boxes[0].behind_doc);
    assert_eq!(boxes[0].fill_hex.as_deref(), Some("080A10"));
    assert!(boxes[0].line_hex.is_none());
    for b in &boxes[1..] {
        assert!(!b.behind_doc);
        assert!(b.node_id.contains("::edge_"), "{}", b.node_id);
        assert!(crate::geo::is_thin_fill_emu(b.cx_emu, b.cy_emu));
    }
}

#[test]
fn unresolved_fill_ref_fails() {
    let (w, h) = page();
    let dec = BoxDecoration {
        background: Some(FillRef::Ref("missing_surface".into())),
        ..Default::default()
    };
    let err = shapes_from_box("card", &rect(), &dec, w, h, 0, "FFFFFF").unwrap_err();
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
    let boxes = shapes_from_box("rule", &rect(), &dec, w, h, 10, "FFFFFF").unwrap();
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
    let boxes = shapes_from_box("card", &rect(), &grad, w, h, 0, "FFFFFF").unwrap();
    assert_eq!(boxes.len(), 1);
    assert!(
        boxes[0].gradient.is_some(),
        "linear gradient is a native fill"
    );
    assert!(
        !boxes[0].behind_doc,
        "non-page gradient stays in front of the page fill"
    );
    let full = Rect {
        x: Pt(0),
        y: Pt(0),
        width: w,
        height: h,
    };
    let page_grad = shapes_from_box("snap.shell", &full, &grad, w, h, 0, "FFFFFF").unwrap();
    assert_eq!(page_grad.len(), 1);
    assert!(
        page_grad[0].behind_doc,
        "full-page gradient must sit behind later text"
    );
    let blur = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#FFFFFF".into(),
        })),
        blur: Some(BlurRef::Inline(Blur { radius_pt: 8_000 })),
        ..Default::default()
    };
    assert!(shapes_from_box("glass", &rect(), &blur, w, h, 0, "FFFFFF")
        .unwrap()
        .is_empty());
}

#[test]
fn four_solid_edges_use_edge_bars() {
    let (w, h) = page();
    let dec = BoxDecoration {
        border: Some(four_edge(BorderStyle::Solid)),
        ..Default::default()
    };
    assert!(dec.border.as_ref().unwrap().is_full_rect_stroke());
    let boxes = shapes_from_box("card", &rect(), &dec, w, h, 10, "FFFFFF").unwrap();
    assert_eq!(boxes.len(), 4, "solid rect outline → 4 thin bars, got {boxes:?}");
    for b in &boxes {
        assert!(b.node_id.contains("::edge_"), "{}", b.node_id);
        assert_eq!(b.fill_hex.as_deref(), Some("FF0000"));
        assert!(b.line_hex.is_none());
    }
}

#[test]
fn filled_card_splits_to_edge_bars_when_axis_aligned() {
    let (w, h) = page();
    let dec = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#F8FAFC".into(),
        })),
        border: Some(four_edge(BorderStyle::Solid)),
        ..Default::default()
    };
    // Contrasting fill stays in front (not paper); solid rect rim is four
    // thin bars so a full-AABB ::stroke cannot steal clicks over labels.
    let boxes = shapes_from_box("card.panel", &rect(), &dec, w, h, 10, "FFFFFF").unwrap();
    assert_eq!(boxes.len(), 5, "fill + 4 edge bars, got {boxes:?}");
    assert!(!boxes[0].behind_doc);
    assert_eq!(boxes[0].fill_hex.as_deref(), Some("F8FAFC"));
    assert!(boxes[0].line_hex.is_none());
    for b in &boxes[1..] {
        assert!(b.node_id.contains("::edge_"), "{}", b.node_id);
        assert_eq!(b.fill_hex.as_deref(), Some("FF0000"));
    }
}

#[test]
fn rounded_filled_card_merges_stroke_onto_fill() {
    let (w, h) = page();
    let dec = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#F8FAFC".into(),
        })),
        border: Some(four_edge(BorderStyle::Solid)),
        corner_radius_pt: Some(8_000),
        ..Default::default()
    };
    let boxes = shapes_from_box("card.panel", &rect(), &dec, w, h, 10, "FFFFFF").unwrap();
    assert_eq!(boxes.len(), 1, "roundRect merges a:ln onto fill, got {boxes:?}");
    assert_eq!(boxes[0].fill_hex.as_deref(), Some("F8FAFC"));
    assert_eq!(boxes[0].line_hex.as_deref(), Some("FF0000"));
    assert!(boxes[0].corner_emu > 0);
    assert!(!boxes[0].node_id.contains("::stroke"));
}

#[test]
fn translucent_border_keeps_lock_alpha() {
    let (w, h) = page();
    let dec = BoxDecoration {
        border: Some(Border {
            width_pt: 500,
            color: "#1E3A8A66".into(),
            edges: vec![
                BorderEdge::Top,
                BorderEdge::Right,
                BorderEdge::Bottom,
                BorderEdge::Left,
            ],
            style: BorderStyle::Solid,
        }),
        ..Default::default()
    };
    let boxes = shapes_from_box("card", &rect(), &dec, w, h, 10, "FFFFFF").unwrap();
    assert_eq!(boxes.len(), 4, "solid rect outline → edge bars, got {boxes:?}");
    assert_eq!(boxes[0].fill_hex.as_deref(), Some("1E3A8A"));
    assert_eq!(boxes[0].fill_alpha, 0x66);
}

#[test]
fn translucent_partial_border_edge_bar_keeps_fill_alpha() {
    let (w, h) = page();
    let dec = BoxDecoration {
        border: Some(Border {
            width_pt: 2_500,
            color: "#38BDF866".into(),
            edges: vec![BorderEdge::Left],
            style: BorderStyle::Solid,
        }),
        ..Default::default()
    };
    let boxes = shapes_from_box("quote", &rect(), &dec, w, h, 10, "FFFFFF").unwrap();
    assert_eq!(boxes.len(), 1);
    assert!(boxes[0].node_id.ends_with("::edge_left"));
    assert_eq!(boxes[0].fill_hex.as_deref(), Some("38BDF8"));
    assert_eq!(boxes[0].fill_alpha, 0x66);
}

#[test]
fn large_fill_keeps_edge_bars_in_front() {
    let (w, h) = page();
    let frame = Rect {
        x: Pt(56_667),
        y: Pt(14_115),
        width: Pt(524_166),
        height: Pt(813_770),
    };
    let dec = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#FFFFFF".into(),
        })),
        border: Some(four_edge(BorderStyle::Solid)),
        ..Default::default()
    };
    let boxes = shapes_from_box("main_frame", &frame, &dec, w, h, 150, "FFFFFF").unwrap();
    assert_eq!(boxes.len(), 5, "fill + 4 edge bars, got {boxes:?}");
    assert!(boxes[0].behind_doc, "large fill stays behind text");
    assert!(boxes[0].line_hex.is_none());
    assert_eq!(boxes[0].fill_hex.as_deref(), Some("FFFFFF"));
    for b in &boxes[1..] {
        assert!(
            !b.behind_doc,
            "frame rim must stay in front of w:background"
        );
        assert!(b.node_id.contains("::edge_"), "{}", b.node_id);
        assert_eq!(b.fill_hex.as_deref(), Some("FF0000"));
    }
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
    let boxes = shapes_from_box("card", &rect(), &dec, w, h, 10, "FFFFFF").unwrap();
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
    let bg = shapes_from_box("root::page_0::background", &full, &dec, w, h, 0, "FFFFFF").unwrap();
    assert_eq!(bg.len(), 1, "full-page solid is the page-paper drawing, got {bg:?}");
    assert!(bg[0].behind_doc, "page paper is the only behindDoc fill");
    assert!(!bg[0].pin_empty_txbox, "page paper must not be an empty txBox");
    assert_eq!(bg[0].fill_hex.as_deref(), Some("FFFFFF"));
    let cell = shapes_from_box("invoice.th.item", &rect(), &dec, w, h, 10, "FFFFFF").unwrap();
    assert_eq!(cell.len(), 1);
    assert!(
        cell[0].behind_doc,
        "paper-colored large fills must sit behind text/pictures"
    );
    let rule_rect = Rect {
        x: Pt(0),
        y: Pt(0),
        width: Pt(114_000),
        height: Pt(1_000),
    };
    let rule = shapes_from_box("addr_line", &rule_rect, &dec, w, h, 10, "FFFFFF").unwrap();
    assert_eq!(rule.len(), 1);
    assert!(
        !rule[0].behind_doc,
        "1 pt rules must stay in front, got behind_doc={}",
        rule[0].behind_doc
    );
    let bar_rect = Rect {
        x: Pt(0),
        y: Pt(0),
        width: Pt(6_000),
        height: Pt(25_000),
    };
    let bar = shapes_from_box("bar.magenta", &bar_rect, &dec, w, h, 10, "FFFFFF").unwrap();
    assert_eq!(bar.len(), 1);
    assert!(
        !bar[0].behind_doc,
        "few-pt accent bars must stay in front, got behind_doc={}",
        bar[0].behind_doc
    );
}

#[test]
fn dark_page_paper_is_behind_doc_wash() {
    // Word Dark Mode remaps/hides w:background; dark designs need the lock
    // full-page solid as behindDoc DrawingML (not background-only).
    let (w, h) = page();
    let full = Rect {
        x: Pt(0),
        y: Pt(0),
        width: w,
        height: h,
    };
    let paper = "#0B0F19";
    let dec = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: paper.into(),
        })),
        ..Default::default()
    };
    let bg = shapes_from_box("root::page_0::background", &full, &dec, w, h, 0, paper).unwrap();
    assert_eq!(bg.len(), 1, "dark full-page solid is the wash, got {bg:?}");
    assert!(bg[0].behind_doc, "dark page paper must be behindDoc");
    assert!(!bg[0].pin_empty_txbox, "page paper must not pin an empty txBox");
    assert_eq!(bg[0].fill_hex.as_deref(), Some("0B0F19"));
    let card = Rect {
        x: Pt(36_000),
        y: Pt(36_000),
        width: Pt(523_000),
        height: Pt(200_000),
    };
    let card_dec = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#162032".into(),
        })),
        ..Default::default()
    };
    let card_boxes = shapes_from_box("dash.card", &card, &card_dec, w, h, 10, paper).unwrap();
    assert_eq!(card_boxes.len(), 1);
    assert!(
        !card_boxes[0].behind_doc,
        "dark cards stay in front; only page paper uses behindDoc"
    );
}

#[test]
fn small_paper_swatch_stays_in_front() {
    // Instagram-style palette chips are paper-RGB but tiny; behindDoc would
    // hide them under the page wash (hosts stack behindDoc by size).
    let (w, h) = page();
    let swatch = Rect {
        x: Pt(40_000),
        y: Pt(40_000),
        width: Pt(48_000),
        height: Pt(26_000),
    };
    let dec = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#FFFFFF".into(),
        })),
        ..Default::default()
    };
    let boxes = shapes_from_box("palette.swatch1", &swatch, &dec, w, h, 30, "FFFFFF").unwrap();
    assert_eq!(boxes.len(), 1);
    assert!(
        !boxes[0].behind_doc,
        "tiny paper swatch must stay in front, got behind_doc"
    );
}

#[test]
fn contrasting_full_page_shell_stays_in_front() {
    // Cream paper + dark full-page slide (editorial-story p4) or dark paper +
    // white full-page slide (swiss-signal p2): only paper-RGB is behindDoc.
    let (w, h) = page();
    let full = Rect {
        x: Pt(0),
        y: Pt(0),
        width: w,
        height: h,
    };
    let paper = "#F9F8F5";
    let paper_dec = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: paper.into(),
        })),
        ..Default::default()
    };
    let wash =
        shapes_from_box("root::page_3::background", &full, &paper_dec, w, h, 0, paper).unwrap();
    assert!(wash[0].behind_doc);
    let slide_dec = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#141312".into(),
        })),
        ..Default::default()
    };
    let slide = shapes_from_box("slide.04", &full, &slide_dec, w, h, 10, paper).unwrap();
    assert_eq!(slide.len(), 1);
    assert!(
        !slide[0].behind_doc,
        "contrasting full-page slide must stay in front of paper wash"
    );
    let dark_paper = "#0A0A0A";
    let white_slide = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#FFFFFF".into(),
        })),
        ..Default::default()
    };
    let white = shapes_from_box("slide.02", &full, &white_slide, w, h, 10, dark_paper).unwrap();
    assert!(
        !white[0].behind_doc,
        "white full-page slide on dark paper must stay in front"
    );
}

#[test]
fn contrasting_large_fill_stays_in_front() {
    let (w, h) = page();
    let body = Rect {
        x: Pt(0),
        y: Pt(170_000),
        width: w,
        height: Pt(672_000),
    };
    let blue = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#002AA6".into(),
        })),
        ..Default::default()
    };
    let boxes = shapes_from_box("body", &body, &blue, w, h, 50, "FFFFFF").unwrap();
    assert_eq!(boxes.len(), 1);
    assert!(
        !boxes[0].behind_doc,
        "cover-page body must stay in front of white w:background, got behind_doc"
    );
    let stripe = Rect {
        x: Pt(0),
        y: Pt(153_000),
        width: w,
        height: Pt(17_000),
    };
    let yellow = BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#F2CB00".into(),
        })),
        ..Default::default()
    };
    let bar = shapes_from_box("stripe", &stripe, &yellow, w, h, 40, "FFFFFF").unwrap();
    assert_eq!(bar.len(), 1);
    assert!(
        !bar[0].behind_doc,
        "17pt contrasting band must stay visible, got behind_doc"
    );
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
    let boxes = shapes_from_box("card", &rect(), &dec, w, h, 0, "FFFFFF").unwrap();
    assert_eq!(boxes[0].corner_emu, crate::coord::millipt_to_emu(8_000));
}
