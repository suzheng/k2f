use crate::coord::millipt_to_pt;
use crate::effect::{box_is_effect, is_rule_id};
use crate::ir::{GradientFill, GradientStopFill, LineDash, ShapeBox};
use crate::IdmlError;
use k2f_core::{Border, BorderEdge, BorderStyle, BoxDecoration, Fill, LinearGradient, Pt, Rect};
use k2f_paint::{parse_hex_rgba, resolve_fill};

pub fn shapes_from_box(
    node_id: &str,
    rect: &Rect,
    decoration: &BoxDecoration,
) -> Result<Vec<ShapeBox>, IdmlError> {
    if is_rule_id(node_id) || box_is_effect(node_id, decoration)? {
        return Ok(Vec::new());
    }
    let fill = match resolve_fill(decoration) {
        Ok(fill) => fill,
        Err(k2f_paint::PaintError::UnresolvedRef(name)) => {
            return Err(IdmlError::Write(format!("unresolved fill ref '{name}'")));
        }
        Err(e) => return Err(e.into()),
    };
    let (fill_hex, fill_alpha, gradient) = match fill {
        Some(Fill::LinearGradient { value }) => (None, 255u8, Some(gradient_from_lock(&value)?)),
        Some(Fill::Solid { color }) => match rgba_hex_alpha(&color)? {
            None => (None, 255u8, None),
            Some((hex, alpha)) => (Some(hex), alpha, None),
        },
        None => (None, 255, None),
    };
    let line = line_from(decoration)?;
    if fill_hex.is_none() && gradient.is_none() && line.is_none() {
        return Ok(Vec::new());
    }
    let base = ShapeBox {
        node_id: node_id.to_string(),
        rect: rect.clone(),
        fill_hex,
        fill_alpha,
        gradient,
        corner_pt: millipt_to_pt(decoration.corner_radius_pt.unwrap_or(0).max(0) as i128),
        line_hex: None,
        line_alpha: 255,
        line_w_pt: 0.0,
        line_dash: LineDash::Solid,
    };
    match line {
        None => Ok(vec![base]),
        Some(ln) if ln.all_four => {
            let mut s = base;
            s.line_hex = Some(ln.hex);
            s.line_alpha = ln.alpha;
            s.line_w_pt = ln.w_pt;
            s.line_dash = ln.dash;
            Ok(vec![s])
        }
        Some(ln) => {
            let mut out = Vec::new();
            if base.fill_hex.is_some() || base.gradient.is_some() {
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
    w_pt: f64,
    dash: LineDash,
    all_four: bool,
}

fn line_from(decoration: &BoxDecoration) -> Result<Option<LineSpec>, IdmlError> {
    let Some(border) = decoration.border.as_ref() else {
        return Ok(None);
    };
    if border.width_pt <= 0 || border.edges.is_empty() {
        return Ok(None);
    }
    let [r, g, b, a] = parse_hex_rgba(&border.color)
        .ok_or_else(|| IdmlError::Write(format!("unparseable color '{}'", border.color)))?;
    if a == 0 {
        return Ok(None);
    }
    Ok(Some(LineSpec {
        hex: format!("{r:02X}{g:02X}{b:02X}"),
        alpha: a,
        w_pt: millipt_to_pt(border.width_pt as i128),
        dash: match border.style {
            BorderStyle::Solid => LineDash::Solid,
            BorderStyle::Dashed => LineDash::Dash,
            BorderStyle::Dotted => LineDash::Dot,
        },
        all_four: uses_closed_stroke(border),
    }))
}

fn uses_closed_stroke(border: &Border) -> bool {
    border.is_full_rect_stroke() || draws_all_four(border)
}

fn draws_all_four(border: &Border) -> bool {
    border.draws_edge(BorderEdge::Top)
        && border.draws_edge(BorderEdge::Right)
        && border.draws_edge(BorderEdge::Bottom)
        && border.draws_edge(BorderEdge::Left)
}

fn edge_bars(base: &ShapeBox, border: &Border, ln: &LineSpec) -> Vec<ShapeBox> {
    let w = Pt(border.width_pt.max(1) as i128);
    let r = &base.rect;
    let edges = [
        (
            BorderEdge::Top,
            "top",
            Rect {
                x: r.x,
                y: r.y,
                width: r.width,
                height: w,
            },
        ),
        (
            BorderEdge::Bottom,
            "bottom",
            Rect {
                x: r.x,
                y: Pt(r.y.0 + r.height.0 - w.0),
                width: r.width,
                height: w,
            },
        ),
        (
            BorderEdge::Left,
            "left",
            Rect {
                x: r.x,
                y: r.y,
                width: w,
                height: r.height,
            },
        ),
        (
            BorderEdge::Right,
            "right",
            Rect {
                x: Pt(r.x.0 + r.width.0 - w.0),
                y: r.y,
                width: w,
                height: r.height,
            },
        ),
    ];
    let mut out = Vec::new();
    for (edge, name, rect) in edges {
        if !border.draws_edge(edge) {
            continue;
        }
        out.push(ShapeBox {
            node_id: format!("{}::edge_{name}", base.node_id),
            rect,
            fill_hex: Some(ln.hex.clone()),
            fill_alpha: ln.alpha,
            gradient: None,
            corner_pt: 0.0,
            line_hex: None,
            line_alpha: 255,
            line_w_pt: 0.0,
            line_dash: LineDash::Solid,
        });
    }
    out
}

fn gradient_from_lock(value: &LinearGradient) -> Result<GradientFill, IdmlError> {
    let LinearGradient::Linear {
        angle_degrees,
        stops,
    } = value;
    let mut out = Vec::with_capacity(stops.len());
    for stop in stops {
        let Some((hex, _alpha)) = rgba_hex_alpha(&stop.color)? else {
            continue;
        };
        out.push(GradientStopFill {
            pos: stop.pos.clamp(0, 1000),
            hex,
        });
    }
    if out.is_empty() {
        return Err(IdmlError::Write("linear gradient has no stops".into()));
    }
    Ok(GradientFill {
        angle_degrees: *angle_degrees,
        stops: out,
    })
}

fn rgba_hex_alpha(color: &str) -> Result<Option<(String, u8)>, IdmlError> {
    let [r, g, b, a] = parse_hex_rgba(color)
        .ok_or_else(|| IdmlError::Write(format!("unparseable fill color '{color}'")))?;
    if a == 0 {
        return Ok(None);
    }
    Ok(Some((format!("{r:02X}{g:02X}{b:02X}"), a)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::{FillRef, GradientStop, LinearGradient};

    fn rect() -> Rect {
        Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(80_000),
            height: Pt(26_800),
        }
    }

    #[test]
    fn linear_gradient_is_native_shape() {
        let dec = BoxDecoration {
            background: Some(FillRef::Inline(Fill::LinearGradient {
                value: LinearGradient::Linear {
                    angle_degrees: 135,
                    stops: vec![
                        GradientStop {
                            pos: 0,
                            color: "#FF8A00".into(),
                        },
                        GradientStop {
                            pos: 1000,
                            color: "#FF5E00".into(),
                        },
                    ],
                },
            })),
            corner_radius_pt: Some(12_000),
            ..Default::default()
        };
        let boxes = shapes_from_box("chip", &rect(), &dec).unwrap();
        assert_eq!(boxes.len(), 1);
        let g = boxes[0].gradient.as_ref().expect("native gradient");
        assert_eq!(g.angle_degrees, 135);
        assert_eq!(g.stops.len(), 2);
        assert_eq!(g.stops[0].hex, "FF8A00");
        assert!((boxes[0].corner_pt - 12.0).abs() < 0.001);
        assert!(boxes[0].fill_hex.is_none());
    }

    #[test]
    fn translucent_solid_fill_keeps_lock_alpha() {
        let dec = BoxDecoration {
            background: Some(FillRef::Inline(Fill::Solid {
                color: "#00000044".into(),
            })),
            corner_radius_pt: Some(7_000),
            ..Default::default()
        };
        let boxes = shapes_from_box("top_hero.scrim", &rect(), &dec).unwrap();
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].fill_hex.as_deref(), Some("000000"));
        assert_eq!(boxes[0].fill_alpha, 0x44);
        assert!(boxes[0].gradient.is_none());
    }

    #[test]
    fn translucent_four_side_stroke_keeps_line_alpha() {
        use k2f_core::{Border, BorderEdge, BorderStyle};

        let dec = BoxDecoration {
            background: Some(FillRef::Inline(Fill::Solid {
                color: "#FFFFFF24".into(),
            })),
            border: Some(Border {
                width_pt: 1000,
                color: "#FFFFFF59".into(),
                edges: vec![
                    BorderEdge::Top,
                    BorderEdge::Right,
                    BorderEdge::Bottom,
                    BorderEdge::Left,
                ],
                style: BorderStyle::Solid,
            }),
            corner_radius_pt: Some(16_000),
            ..Default::default()
        };
        let boxes = shapes_from_box("glass_card", &rect(), &dec).unwrap();
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].fill_hex.as_deref(), Some("FFFFFF"));
        assert_eq!(boxes[0].fill_alpha, 0x24);
        assert_eq!(boxes[0].line_hex.as_deref(), Some("FFFFFF"));
        assert_eq!(boxes[0].line_alpha, 0x59);
    }
}
