use crate::coord::{millipt_to_emu, pt_to_emu};
use crate::geo::is_full_page_rect;
use crate::ir::{GradientFill, GradientStopFill, LineDash, ShapeBox};
use crate::DocxError;
use k2f_core::{Border, BorderEdge, BorderStyle, BoxDecoration, Fill, LinearGradient, Pt, Rect};
use k2f_paint::{parse_hex_rgba, resolve_fill};

pub(crate) fn shapes_from_box(
    node_id: &str,
    rect: &Rect,
    decoration: &BoxDecoration,
    page_w: Pt,
    page_h: Pt,
    relative_height: u32,
    paper_hex: &str,
) -> Result<Vec<ShapeBox>, DocxError> {
    if node_id.contains("::rule_") {
        return Ok(Vec::new());
    }
    if decoration.shadow.is_some() || decoration.blur.is_some() {
        return Ok(Vec::new());
    }
    let (fill_hex, fill_alpha, gradient) = match resolve_fill(decoration) {
        Ok(Some(Fill::LinearGradient { value })) => (None, 255, Some(gradient_from_lock(&value)?)),
        Ok(Some(Fill::Solid { color })) => match rgba_hex_alpha(&color)? {
            None => (None, 255, None),
            Some((hex, alpha)) => (Some(hex), alpha, None),
        },
        Ok(None) => (None, 255, None),
        Err(k2f_paint::PaintError::UnresolvedRef(name)) => {
            return Err(DocxError::Write(format!("unresolved fill ref '{name}'")));
        }
        Err(e) => return Err(e.into()),
    };
    let line = line_from(decoration)?;
    if fill_hex.is_none() && gradient.is_none() && line.is_none() {
        return Ok(Vec::new());
    }
    // Paper-colored large fills sit behind text/pictures so later white
    // containers cannot cover stamps (LibreOffice paints in-front shapes over
    // pictures). Writer also paints behindDoc under `w:background`, so a
    // contrasting fill (cover body, yellow band) would vanish — those stay in
    // front as empty text boxes (in-front shapes cover later labels). Thin
    // fills (1 pt rules, few-pt accent bars) always stay in front.
    // Gradients and translucent fills stay in front: behindDoc would hide them
    // under white `w:background`, and they are not paper-colored cards.
    let behind_doc = behind_doc_for_fill(
        node_id,
        rect,
        fill_hex.as_deref(),
        fill_alpha,
        gradient.is_some(),
        page_w,
        page_h,
        paper_hex,
    );
    let base = ShapeBox {
        node_id: node_id.to_string(),
        x_emu: pt_to_emu(rect.x),
        y_emu: pt_to_emu(rect.y),
        cx_emu: pt_to_emu(rect.width),
        cy_emu: pt_to_emu(rect.height),
        fill_hex,
        fill_alpha,
        gradient,
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
            if behind_doc && (base.fill_hex.is_some() || base.gradient.is_some()) {
                // Fill stays behind text/pictures. The outline must not: LibreOffice
                // Writer paints behindDoc shapes under `w:background`, so a merged
                // fill+stroke frame (drawing title blocks, card shells) disappears.
                let fill = base.clone();
                let mut stroke = base;
                stroke.fill_hex = None;
                stroke.fill_alpha = 255;
                stroke.gradient = None;
                stroke.behind_doc = false;
                stroke.line_hex = Some(ln.hex);
                stroke.line_w_emu = ln.w_emu;
                stroke.line_dash = ln.dash;
                stroke.node_id = format!("{}::stroke", stroke.node_id);
                Ok(vec![fill, stroke])
            } else {
                let mut s = base;
                s.line_hex = Some(ln.hex);
                s.line_w_emu = ln.w_emu;
                s.line_dash = ln.dash;
                Ok(vec![s])
            }
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

fn gradient_from_lock(value: &LinearGradient) -> Result<GradientFill, DocxError> {
    let LinearGradient::Linear {
        angle_degrees,
        stops,
    } = value;
    let mut out = Vec::with_capacity(stops.len());
    for stop in stops {
        let Some((hex, alpha)) = rgba_hex_alpha(&stop.color)? else {
            continue;
        };
        out.push(GradientStopFill {
            pos: stop.pos.clamp(0, 1000),
            hex,
            alpha,
        });
    }
    if out.is_empty() {
        return Err(DocxError::Write("linear gradient has no stops".into()));
    }
    Ok(GradientFill {
        angle_degrees: *angle_degrees,
        stops: out,
    })
}

fn behind_doc_for_fill(
    _node_id: &str,
    rect: &Rect,
    fill_hex: Option<&str>,
    fill_alpha: u8,
    is_gradient: bool,
    page_w: Pt,
    page_h: Pt,
    paper_hex: &str,
) -> bool {
    // Full-page fills (gradient shells, named ::background) sit behind text.
    // A page-sized in-front shape covers later labels in LibreOffice Writer.
    // Non-page glass/translucent fills stay in front (above the page fill).
    if is_full_page_rect(page_w, page_h, rect) {
        return true;
    }
    if is_gradient || fill_alpha < 255 {
        return false;
    }
    let Some(hex) = fill_hex else {
        return false;
    };
    if crate::geo::is_thin_fill_rect(rect) {
        return false;
    }
    paper_hex_eq(hex, paper_hex)
}

fn paper_hex_eq(a: &str, b: &str) -> bool {
    fn norm(h: &str) -> String {
        match h.to_ascii_uppercase().as_str() {
            "FFFFFE" => "FFFFFF".into(),
            "000001" => "000000".into(),
            other => other.to_string(),
        }
    }
    norm(a) == norm(b)
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
            fill_alpha: 255,
            gradient: None,
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

fn rgba_hex_alpha(color: &str) -> Result<Option<(String, u8)>, DocxError> {
    let [r, g, b, a] = parse_hex_rgba(color)
        .ok_or_else(|| DocxError::Write(format!("unparseable fill color '{color}'")))?;
    if a == 0 {
        return Ok(None);
    }
    Ok(Some((format!("{r:02X}{g:02X}{b:02X}"), a)))
}

fn srgb_hex(color: &str) -> Result<String, DocxError> {
    let [r, g, b, _] = parse_hex_rgba(color)
        .ok_or_else(|| DocxError::Write(format!("unparseable color '{color}'")))?;
    Ok(format!("{r:02X}{g:02X}{b:02X}"))
}
