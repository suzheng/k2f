use crate::coord::{millipt_to_emu, pt_to_emu};
use crate::ir::{LineDash, ShapeBox};
use crate::PptxError;
use k2f_core::{Border, BorderEdge, BorderStyle, BoxDecoration, Fill, Rect};
use k2f_paint::{parse_hex_rgba, resolve_fill};

pub(crate) fn shapes_from_box(
    node_id: &str,
    rect: &Rect,
    decoration: &BoxDecoration,
) -> Result<Vec<ShapeBox>, PptxError> {
    if crate::effect::is_rule_id(node_id) || crate::effect::box_is_effect(node_id, decoration)? {
        return Ok(Vec::new());
    }
    let fill = match resolve_fill(decoration) {
        Ok(fill) => fill,
        Err(k2f_paint::PaintError::UnresolvedRef(name)) => {
            return Err(PptxError::Write(format!("unresolved fill ref '{name}'")));
        }
        Err(e) => return Err(e.into()),
    };
    let (fill_hex, fill_alpha) = match fill {
        Some(Fill::LinearGradient { .. }) => return Ok(Vec::new()),
        Some(Fill::Solid { color }) => match rgba_hex_alpha(&color)? {
            None => (None, 255),
            Some((hex, alpha)) => (Some(hex), alpha),
        },
        None => (None, 255),
    };
    let line = line_from(decoration)?;
    if fill_hex.is_none() && line.is_none() {
        return Ok(Vec::new());
    }
    let base = ShapeBox {
        node_id: node_id.to_string(),
        x_emu: pt_to_emu(rect.x),
        y_emu: pt_to_emu(rect.y),
        cx_emu: pt_to_emu(rect.width),
        cy_emu: pt_to_emu(rect.height),
        fill_hex,
        fill_alpha,
        corner_emu: millipt_to_emu(decoration.corner_radius_pt.unwrap_or(0).max(0)),
        line_hex: None,
        line_alpha: 255,
        line_w_emu: 0,
        line_dash: LineDash::Solid,
    };
    match line {
        None => Ok(vec![base]),
        Some(ln) if ln.all_four => {
            let mut s = base;
            s.line_hex = Some(ln.hex);
            s.line_alpha = ln.alpha;
            s.line_w_emu = ln.w_emu;
            s.line_dash = ln.dash;
            Ok(vec![s])
        }
        Some(ln) => {
            let mut out = Vec::new();
            if base.fill_hex.is_some() {
                out.push(base.clone());
            }
            out.extend(edge_bars(&base, decoration.border.as_ref().unwrap(), &ln));
            Ok(out)
        }
    }
}

struct LineSpec {
    hex: String,
    alpha: u8,
    w_emu: i64,
    dash: LineDash,
    all_four: bool,
}

fn line_from(decoration: &BoxDecoration) -> Result<Option<LineSpec>, PptxError> {
    let Some(border) = decoration.border.as_ref() else {
        return Ok(None);
    };
    if border.width_pt <= 0 || border.edges.is_empty() {
        return Ok(None);
    }
    let [r, g, b, a] = parse_hex_rgba(&border.color)
        .ok_or_else(|| PptxError::Write(format!("unparseable color '{}'", border.color)))?;
    if a == 0 {
        return Ok(None);
    }
    Ok(Some(LineSpec {
        hex: crate::text::pin_office_srgb(&format!("{r:02X}{g:02X}{b:02X}")),
        alpha: a,
        w_emu: millipt_to_emu(border.width_pt),
        dash: match border.style {
            BorderStyle::Solid => LineDash::Solid,
            BorderStyle::Dashed => LineDash::Dash,
            BorderStyle::Dotted => LineDash::Dot,
        },
        all_four: uses_closed_ln(border),
    }))
}

/// DrawingML `a:ln` is four-sided. Use it when every edge is present.
/// Partial edges become filled bars so a left-only quote rule is not a
/// full rectangle (same split as Word).
fn uses_closed_ln(border: &Border) -> bool {
    border.is_full_rect_stroke() || draws_all_four(border)
}

fn draws_all_four(border: &Border) -> bool {
    border.draws_edge(BorderEdge::Top)
        && border.draws_edge(BorderEdge::Right)
        && border.draws_edge(BorderEdge::Bottom)
        && border.draws_edge(BorderEdge::Left)
}

fn edge_bars(base: &ShapeBox, border: &Border, ln: &LineSpec) -> Vec<ShapeBox> {
    let w = ln.w_emu.max(1);
    let mut out = Vec::new();
    let edges = [
        (
            BorderEdge::Top,
            "top",
            base.x_emu,
            base.y_emu,
            base.cx_emu,
            w,
        ),
        (
            BorderEdge::Bottom,
            "bottom",
            base.x_emu,
            base.y_emu + base.cy_emu - w,
            base.cx_emu,
            w,
        ),
        (
            BorderEdge::Left,
            "left",
            base.x_emu,
            base.y_emu,
            w,
            base.cy_emu,
        ),
        (
            BorderEdge::Right,
            "right",
            base.x_emu + base.cx_emu - w,
            base.y_emu,
            w,
            base.cy_emu,
        ),
    ];
    for (edge, name, x, y, cx, cy) in edges {
        if !border.draws_edge(edge) {
            continue;
        }
        out.push(ShapeBox {
            node_id: format!("{}::edge_{name}", base.node_id),
            x_emu: x,
            y_emu: y,
            cx_emu: cx,
            cy_emu: cy,
            fill_hex: Some(ln.hex.clone()),
            fill_alpha: ln.alpha,
            corner_emu: 0,
            line_hex: None,
            line_alpha: 255,
            line_w_emu: 0,
            line_dash: LineDash::Solid,
        });
    }
    out
}

fn rgba_hex_alpha(color: &str) -> Result<Option<(String, u8)>, PptxError> {
    let [r, g, b, a] = parse_hex_rgba(color)
        .ok_or_else(|| PptxError::Write(format!("unparseable fill color '{color}'")))?;
    if a == 0 {
        return Ok(None);
    }
    Ok(Some((
        crate::text::pin_office_srgb(&format!("{r:02X}{g:02X}{b:02X}")),
        a,
    )))
}

pub(crate) fn round_rect_adj(corner_emu: i64, cx: i64, cy: i64) -> i64 {
    let min = cx.min(cy);
    if corner_emu <= 0 || min <= 0 {
        return 0;
    }
    (corner_emu.saturating_mul(100_000) / min).clamp(0, 50_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::{FillRef, Pt, Shadow, ShadowRef};

    fn rect() -> Rect {
        Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(100_000),
            height: Pt(50_000),
        }
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
    fn skips_shadow_and_rule_and_empty() {
        let mut dec = BoxDecoration {
            background: Some(FillRef::Inline(Fill::Solid {
                color: "#FFFFFF".into(),
            })),
            shadow: Some(ShadowRef::Inline(Shadow { layers: vec![] })),
            ..Default::default()
        };
        assert!(!crate::effect::box_is_effect("card", &dec).unwrap());
        let shadowed = shapes_from_box("card", &rect(), &dec).unwrap();
        assert_eq!(shadowed.len(), 1, "shadow-only boxes stay native fills");
        assert_eq!(shadowed[0].fill_hex.as_deref(), Some("FFFFFE"));
        dec.shadow = None;
        assert!(shapes_from_box("eq::rule_1", &rect(), &dec)
            .unwrap()
            .is_empty());
        let empty = BoxDecoration::default();
        assert!(shapes_from_box("empty", &rect(), &empty)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn emits_opaque_page_background() {
        let dec = BoxDecoration {
            background: Some(FillRef::Inline(Fill::Solid {
                color: "#FFFFFF".into(),
            })),
            ..Default::default()
        };
        let boxes = shapes_from_box("root::page_0::background", &rect(), &dec).unwrap();
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].fill_hex.as_deref(), Some("FFFFFE"));
        assert_eq!(boxes[0].corner_emu, 0);
    }

    #[test]
    fn adj_clamped_to_half_min_side() {
        assert_eq!(round_rect_adj(0, 10_000, 10_000), 0);
        assert_eq!(round_rect_adj(1_000, 10_000, 20_000), 10_000);
        assert_eq!(round_rect_adj(10_000, 10_000, 10_000), 50_000);
    }

    #[test]
    fn unresolved_fill_ref_fails_export() {
        let dec = BoxDecoration {
            background: Some(FillRef::Ref("missing_surface".into())),
            ..Default::default()
        };
        let err = shapes_from_box("card", &rect(), &dec).unwrap_err();
        match err {
            PptxError::Write(msg) => assert!(msg.contains("unresolved fill ref")),
            other => panic!("expected Write, got {other:?}"),
        }
    }

    #[test]
    fn partial_border_emits_edge_bars_not_four_sided_ln() {
        let dec = BoxDecoration {
            border: Some(Border {
                width_pt: 2_500,
                color: "#0C0C0E".into(),
                edges: vec![BorderEdge::Left],
                style: BorderStyle::Solid,
            }),
            ..Default::default()
        };
        let boxes = shapes_from_box("quote", &rect(), &dec).unwrap();
        assert_eq!(boxes.len(), 1);
        assert!(boxes[0].node_id.ends_with("::edge_left"));
        assert_eq!(boxes[0].fill_hex.as_deref(), Some("0C0C0E"));
        assert!(boxes[0].line_hex.is_none());
        assert_eq!(boxes[0].cx_emu, millipt_to_emu(2_500));
        assert_eq!(boxes[0].cy_emu, pt_to_emu(rect().height));
    }

    #[test]
    fn four_solid_edges_use_ln() {
        let dec = BoxDecoration {
            border: Some(four_edge(BorderStyle::Solid)),
            ..Default::default()
        };
        let boxes = shapes_from_box("card", &rect(), &dec).unwrap();
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].line_hex.as_deref(), Some("FF0000"));
        assert_eq!(boxes[0].line_alpha, 255);
        assert_eq!(boxes[0].line_dash, LineDash::Solid);
        assert!(!boxes[0].node_id.contains("::edge_"));
    }

    #[test]
    fn translucent_border_keeps_lock_alpha() {
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
        let boxes = shapes_from_box("card", &rect(), &dec).unwrap();
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].line_hex.as_deref(), Some("1E3A8A"));
        assert_eq!(boxes[0].line_alpha, 0x66);
    }

    #[test]
    fn translucent_partial_border_edge_bar_keeps_fill_alpha() {
        let dec = BoxDecoration {
            border: Some(Border {
                width_pt: 2_500,
                color: "#38BDF866".into(),
                edges: vec![BorderEdge::Left],
                style: BorderStyle::Solid,
            }),
            ..Default::default()
        };
        let boxes = shapes_from_box("quote", &rect(), &dec).unwrap();
        assert_eq!(boxes.len(), 1);
        assert!(boxes[0].node_id.ends_with("::edge_left"));
        assert_eq!(boxes[0].fill_hex.as_deref(), Some("38BDF8"));
        assert_eq!(boxes[0].fill_alpha, 0x66);
    }

    #[test]
    fn four_dashed_edges_use_ln_not_bars() {
        let dec = BoxDecoration {
            border: Some(four_edge(BorderStyle::Dashed)),
            ..Default::default()
        };
        let boxes = shapes_from_box("card", &rect(), &dec).unwrap();
        assert_eq!(boxes.len(), 1, "dashed rectangle must stay one a:ln");
        assert_eq!(boxes[0].line_dash, LineDash::Dash);
        assert_eq!(boxes[0].line_hex.as_deref(), Some("FF0000"));
    }

    #[test]
    fn translucent_solid_fill_keeps_lock_alpha() {
        let dec = BoxDecoration {
            background: Some(FillRef::Inline(Fill::Solid {
                color: "#FFFFFFE6".into(),
            })),
            ..Default::default()
        };
        assert!(!crate::effect::box_is_effect("plaque", &dec).unwrap());
        let boxes = shapes_from_box("plaque", &rect(), &dec).unwrap();
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].fill_hex.as_deref(), Some("FFFFFE"));
        assert_eq!(boxes[0].fill_alpha, 0xE6);
        assert!(boxes[0].line_hex.is_none());
    }

    #[test]
    fn fill_plus_left_border_keeps_both() {
        let dec = BoxDecoration {
            background: Some(FillRef::Inline(Fill::Solid {
                color: "#FFE600".into(),
            })),
            border: Some(Border {
                width_pt: 1_000,
                color: "#111111".into(),
                edges: vec![BorderEdge::Left],
                style: BorderStyle::Solid,
            }),
            ..Default::default()
        };
        let boxes = shapes_from_box("callout", &rect(), &dec).unwrap();
        assert_eq!(boxes.len(), 2);
        assert_eq!(boxes[0].node_id, "callout");
        assert_eq!(boxes[0].fill_hex.as_deref(), Some("FFE600"));
        assert!(boxes[1].node_id.ends_with("::edge_left"));
    }
}
