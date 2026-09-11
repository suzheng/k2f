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
    if decoration.blur.is_some() {
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
    // 1. Page paper — lock full-page solid matching paper RGB is behindDoc
    //    (plus paper-colored body clones). Contrasting full-page slide shells
    //    stay in front so they do not fight the wash in Word Dark Mode.
    //    w:background is page-0 only (Office page-color slot). Word Dark Mode
    //    may hide that slot; the shape is the wash that still paints.
    // 2. Cards/titles — never behindDoc. Hosts z-order that stack by size, so
    //    a page-sized behindDoc drawing would hide every smaller contrasting
    //    fill. Nested labels fold into the shell (classify). Overlapped empty
    //    shells drop the txBox pin instead of going behind.
    // 3. Dark Mode color identity — lock sRGB pins in xml.rs / theme1.xml.
    // Thin fills, gradients, and frost stay in front of the wash.
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
        line_alpha: 255,
        line_w_emu: 0,
        line_dash: LineDash::Solid,
        behind_doc,
        relative_height,
        pin_empty_txbox: !behind_doc,
    };
    match line {
        None => Ok(vec![base]),
        Some(ln) if ln.all_four => {
            let border = decoration.border.as_ref().unwrap();
            // Axis-aligned solid rims → four thin `::edge_*` bars. A closed
            // noFill `::stroke` sibling shares the full AABB and hosts still
            // hit-test that frame over nested labels even when demoted in
            // relativeHeight (full-page business-card shells).
            //
            // Rounded solid rims keep `a:ln` on the fill shape (no sibling):
            // a separate roundRect ::stroke is the same click-shield class.
            // Dashed outlines keep a closed `a:ln` shape so dash stays one path.
            let solid = ln.dash == LineDash::Solid;
            let use_edge_bars = base.corner_emu <= 0 && solid;
            if base.fill_hex.is_some() || base.gradient.is_some() {
                if use_edge_bars {
                    let mut out = vec![base.clone()];
                    out.extend(edge_bars(&base, border, &ln));
                    return Ok(out);
                }
                if solid {
                    // roundRect (or other non-zero corner): merge outline onto fill.
                    let mut s = base;
                    s.line_hex = Some(ln.hex);
                    s.line_alpha = ln.alpha;
                    s.line_w_emu = ln.w_emu;
                    s.line_dash = ln.dash;
                    return Ok(vec![s]);
                }
                // Dashed: fill + closed outline sibling (dash needs `a:ln`).
                let fill = base.clone();
                let mut stroke = base;
                stroke.fill_hex = None;
                stroke.fill_alpha = 255;
                stroke.gradient = None;
                stroke.behind_doc = false;
                stroke.line_hex = Some(ln.hex);
                stroke.line_alpha = ln.alpha;
                stroke.line_w_emu = ln.w_emu;
                stroke.line_dash = ln.dash;
                stroke.node_id = format!("{}::stroke", stroke.node_id);
                Ok(vec![fill, stroke])
            } else if use_edge_bars {
                Ok(edge_bars(&base, border, &ln))
            } else {
                let mut s = base;
                s.line_hex = Some(ln.hex);
                s.line_alpha = ln.alpha;
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
    // Only the page wash (and large paper-colored body clones) go behind text.
    // Contrasting fills stay in front — including full-page slide shells whose
    // RGB differs from paper (e.g. cream paper + dark slide.04). Sending those
    // behindDoc fought the real wash and Word Dark Mode remapped the page.
    //
    // Small paper-colored chips (palette swatches, badges) must stay in front:
    // behindDoc stacks by size, so the full-page wash would hide them.
    let full_page = is_full_page_rect(page_w, page_h, rect);
    if full_page && is_gradient {
        return true;
    }
    if is_gradient || fill_alpha < 255 {
        return false;
    }
    let Some(hex) = fill_hex else {
        return false;
    };
    if !full_page && crate::geo::is_thin_fill_rect(rect) {
        return false;
    }
    if !paper_hex_eq(hex, paper_hex) {
        return false;
    }
    // Small paper-colored chips (palette swatches) must stay in front:
    // behindDoc stacks by size, so the full-page wash would hide them.
    full_page || paper_colored_body_clone(rect, page_w, page_h)
}

/// Large paper-colored regions (invoice white containers) sit behind so they
/// cannot cover stamps. Palette swatches are a tiny fraction of the page and
/// must stay in front (behindDoc stacks by size under the page wash).
fn paper_colored_body_clone(rect: &Rect, page_w: Pt, page_h: Pt) -> bool {
    // area/page >= 1/200 (~0.5%): A4 test cells (~1%) qualify; IG swatches (~0.3%) do not.
    let area = rect.width.0.saturating_mul(rect.height.0);
    let page_area = page_w.0.saturating_mul(page_h.0);
    page_area > 0 && area.saturating_mul(200) >= page_area
}

fn paper_hex_eq(a: &str, b: &str) -> bool {
    fn norm(h: &str) -> String {
        let t = h.trim().trim_start_matches('#').to_ascii_uppercase();
        match t.as_str() {
            "FFFFFE" => "FFFFFF".into(),
            "000001" => "000000".into(),
            other => other.to_string(),
        }
    }
    norm(a) == norm(b)
}

struct LineSpec {
    hex: String,
    alpha: u8,
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
    let Some((hex, alpha)) = rgba_hex_alpha(&border.color)? else {
        return Ok(None);
    };
    Ok(Some(LineSpec {
        hex,
        alpha,
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
            fill_alpha: ln.alpha,
            gradient: None,
            corner_emu: 0,
            line_hex: None,
            line_alpha: 255,
            line_w_emu: 0,
            line_dash: LineDash::Solid,
            behind_doc: false,
            relative_height: base.relative_height,
            pin_empty_txbox: true,
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
