use crate::coord::millipt_to_pt;
use crate::effect::{box_is_effect, is_rule_id};
use crate::ir::{LineDash, ShapeBox};
use crate::IdmlError;
use k2f_core::{Border, BorderEdge, BorderStyle, BoxDecoration, Fill, Pt, Rect};
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
    let (fill_hex, fill_alpha) = match fill {
        Some(Fill::LinearGradient { .. }) => return Ok(Vec::new()),
        Some(Fill::Solid { color }) => match solid_opaque_hex(&color)? {
            None => return Ok(Vec::new()),
            Some(hex) => (Some(hex), 255u8),
        },
        None => (None, 255),
    };
    let line = line_from(decoration)?;
    if fill_hex.is_none() && line.is_none() {
        return Ok(Vec::new());
    }
    let base = ShapeBox {
        node_id: node_id.to_string(),
        rect: rect.clone(),
        fill_hex,
        fill_alpha,
        corner_pt: millipt_to_pt(decoration.corner_radius_pt.unwrap_or(0).max(0) as i128),
        line_hex: None,
        line_w_pt: 0.0,
        line_dash: LineDash::Solid,
    };
    match line {
        None => Ok(vec![base]),
        Some(ln) if ln.all_four => {
            let mut s = base;
            s.line_hex = Some(ln.hex);
            s.line_w_pt = ln.w_pt;
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
            fill_alpha: 255,
            corner_pt: 0.0,
            line_hex: None,
            line_w_pt: 0.0,
            line_dash: LineDash::Solid,
        });
    }
    out
}

fn solid_opaque_hex(color: &str) -> Result<Option<String>, IdmlError> {
    let [r, g, b, a] = parse_hex_rgba(color)
        .ok_or_else(|| IdmlError::Write(format!("unparseable fill color '{color}'")))?;
    if a < 255 {
        return Ok(None);
    }
    Ok(Some(format!("{r:02X}{g:02X}{b:02X}")))
}
