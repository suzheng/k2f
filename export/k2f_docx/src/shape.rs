use crate::coord::{millipt_to_emu, pt_to_emu};
use crate::geo::is_full_page_rect;
use crate::ir::{LineDash, ShapeBox};
use crate::DocxError;
use k2f_core::{Border, BorderEdge, BorderStyle, BoxDecoration, Fill, Pt, Rect};
use k2f_paint::{parse_hex_rgba, resolve_fill};

pub(crate) fn shapes_from_box(
    node_id: &str,
    rect: &Rect,
    decoration: &BoxDecoration,
    page_w: Pt,
    page_h: Pt,
    relative_height: u32,
) -> Result<Vec<ShapeBox>, DocxError> {
    if node_id.contains("::rule_") {
        return Ok(Vec::new());
    }
    if decoration.shadow.is_some() || decoration.blur.is_some() {
        return Ok(Vec::new());
    }
    let fill_hex = match resolve_fill(decoration) {
        Ok(Some(Fill::LinearGradient { .. })) => return Ok(Vec::new()),
        Ok(Some(Fill::Solid { color })) => Some(opaque_srgb_hex(&color)?),
        Ok(None) => None,
        Err(k2f_paint::PaintError::UnresolvedRef(name)) => {
            return Err(DocxError::Write(format!("unresolved fill ref '{name}'")));
        }
        Err(e) => return Err(e.into()),
    };
    let fill_hex = match fill_hex {
        Some(None) => return Ok(Vec::new()),
        Some(Some(hex)) => Some(hex),
        None => None,
    };
    let line = line_from(decoration)?;
    if fill_hex.is_none() && line.is_none() {
        return Ok(Vec::new());
    }
    // Large fills sit behind text/pictures. LibreOffice Writer otherwise paints
    // later container rects on top of pictures (stamp) and text (address).
    // 1 pt rules stay in front so form underlines remain visible.
    let behind_doc = (fill_hex.is_some() && !crate::geo::is_thin_fill_rect(rect))
        || (node_id.contains("::background") && is_full_page_rect(page_w, page_h, rect));
    let base = ShapeBox {
        node_id: node_id.to_string(),
        x_emu: pt_to_emu(rect.x),
        y_emu: pt_to_emu(rect.y),
        cx_emu: pt_to_emu(rect.width),
        cy_emu: pt_to_emu(rect.height),
        fill_hex,
        corner_emu: millipt_to_emu(decoration.corner_radius_pt.unwrap_or(0).max(0)),
        line_hex: None,
        line_w_emu: 0,
        line_dash: LineDash::Solid,
        behind_doc,
        relative_height,
    };
    match line {
        None => Ok(vec![base]),
        Some(ln) if ln.all_four => {
            let mut s = base;
            s.line_hex = Some(ln.hex);
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
    w_emu: i64,
    dash: LineDash,
    all_four: bool,
}

fn line_from(decoration: &BoxDecoration) -> Result<Option<LineSpec>, DocxError> {
    let Some(border) = decoration.border.as_ref() else {
        return Ok(None);
    };
    if border.width_pt <= 0 || border.edges.is_empty() {
        return Ok(None);
    }
    Ok(Some(LineSpec {
        hex: srgb_hex(&border.color)?,
        w_emu: millipt_to_emu(border.width_pt),
        dash: match border.style {
            BorderStyle::Solid => LineDash::Solid,
            BorderStyle::Dashed => LineDash::Dash,
            BorderStyle::Dotted => LineDash::Dot,
        },
        all_four: uses_closed_ln(border),
    }))
}

/// Word `a:ln` is four-sided. Use it when every edge is present.
///
/// `Border::is_full_rect_stroke` is the plan’s solid-rect predicate. Dashed/dotted
/// four-edge strokes still use `a:ln` + `prstDash` so they are not replaced by four
/// solid bars.
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
            corner_emu: 0,
            line_hex: None,
            line_w_emu: 0,
            line_dash: LineDash::Solid,
            behind_doc: false,
            relative_height: base.relative_height,
        });
    }
    out
}

pub(crate) fn round_rect_adj(corner_emu: i64, cx: i64, cy: i64) -> i64 {
    let min = cx.min(cy);
    if corner_emu <= 0 || min <= 0 {
        return 0;
    }
    (corner_emu.saturating_mul(100_000) / min).clamp(0, 50_000)
}

fn opaque_srgb_hex(color: &str) -> Result<Option<String>, DocxError> {
    let [r, g, b, a] = parse_hex_rgba(color)
        .ok_or_else(|| DocxError::Write(format!("unparseable fill color '{color}'")))?;
    if a < 255 {
        return Ok(None);
    }
    Ok(Some(format!("{r:02X}{g:02X}{b:02X}")))
}

fn srgb_hex(color: &str) -> Result<String, DocxError> {
    let [r, g, b, _] = parse_hex_rgba(color)
        .ok_or_else(|| DocxError::Write(format!("unparseable color '{color}'")))?;
    Ok(format!("{r:02X}{g:02X}{b:02X}"))
}
