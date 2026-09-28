//! Effect slices for PDF: blur, shadow, gradient, and translucent boxes.
//! Copied in spirit from the PPTX/DOCX chrome filter, with a wider predicate
//! because PDF has no native shadow, axial shading, or fill opacity.

use k2f_core::{
    Border, BoxDecoration, Fill, PaintOp, Pt, Rect, Shadow, ShadowRef,
};
use k2f_paint::{parse_hex_rgba, resolve_fill};

use crate::error::PdfError;

#[derive(Debug, Clone)]
pub struct Slice {
    /// Index of the op that starts this slice (`BackdropBlur` or `DrawBox`).
    pub anchor: usize,
    /// Anchor plus a following plate box consumed with a blur.
    pub consumed: Vec<usize>,
    pub crop: Rect,
    /// Ops rasterized for color (backdrop included for blur).
    pub color_ops: Vec<PaintOp>,
    /// Effect plate only, rendered on a transparent canvas. Empty for a
    /// shadow/gradient/translucent box that already has natural alpha.
    pub plate_ops: Vec<PaintOp>,
    /// When set, pixels inside this rect with plate coverage use the color
    /// pass at alpha 255 (baked blur). Outside it, the plate's own RGBA is
    /// the shadow halo.
    pub cover: Option<Rect>,
}

pub fn slices_for_page(ops: &[PaintOp], page_w: Pt, page_h: Pt) -> Result<Vec<Slice>, PdfError> {
    let mut out = Vec::new();
    let mut consumed = vec![false; ops.len()];
    for i in 0..ops.len() {
        if consumed[i] {
            continue;
        }
        match &ops[i] {
            PaintOp::BackdropBlur { node_id, rect, .. } => {
                let follow = consume_following_box(ops, i, node_id);
                if let Some(j) = follow {
                    consumed[j] = true;
                }
                consumed[i] = true;
                let mut crop = glass_crop(rect, ops, follow)?;
                crop = clamp_rect_to_page(crop, page_w, page_h);
                let plate_idx = follow;
                let plate_ops = plate_idx.map(|j| vec![ops[j].clone()]).unwrap_or_default();
                let color_ops = backdrop_color_ops(ops, i, follow, &crop);
                out.push(Slice {
                    anchor: i,
                    consumed: match follow {
                        Some(j) => vec![i, j],
                        None => vec![i],
                    },
                    crop,
                    color_ops,
                    plate_ops,
                    cover: Some(rect.clone()),
                });
            }
            PaintOp::DrawBox {
                node_id,
                rect,
                decoration,
            } if box_is_slice(node_id, decoration)? => {
                consumed[i] = true;
                let crop = clamp_rect_to_page(effect_box_crop(rect, decoration)?, page_w, page_h);
                out.push(Slice {
                    anchor: i,
                    consumed: vec![i],
                    crop: crop.clone(),
                    color_ops: vec![ops[i].clone()],
                    plate_ops: Vec::new(),
                    cover: None,
                });
            }
            _ => {}
        }
    }
    Ok(out)
}

pub fn box_is_slice(node_id: &str, decoration: &BoxDecoration) -> Result<bool, PdfError> {
    if is_rule_id(node_id) {
        return Ok(false);
    }
    if decoration.blur.is_some() || decoration.shadow.is_some() {
        return Ok(true);
    }
    match resolve_fill(decoration) {
        Ok(Some(Fill::LinearGradient { .. })) => return Ok(true),
        Ok(Some(Fill::Solid { color })) if alpha_below_opaque(&color) => return Ok(true),
        Ok(Some(Fill::Solid { .. }) | None) => {}
        Err(e) => return Err(PdfError::Paint(e)),
    }
    if border_alpha_below_opaque(decoration.border.as_ref()) {
        return Ok(true);
    }
    Ok(false)
}

pub fn consume_following_box(ops: &[PaintOp], i: usize, node_id: &str) -> Option<usize> {
    match ops.get(i + 1) {
        Some(PaintOp::DrawBox { node_id: bid, .. }) if bid == node_id => Some(i + 1),
        _ => None,
    }
}

pub fn glass_crop(blur_rect: &Rect, ops: &[PaintOp], follow: Option<usize>) -> Result<Rect, PdfError> {
    let mut crop = blur_rect.clone();
    if let Some(idx) = follow {
        if let PaintOp::DrawBox {
            rect, decoration, ..
        } = &ops[idx]
        {
            crop = union_rect(crop, rect.clone());
            crop = expand_rect_for_shadow(crop, decoration)?;
        }
    }
    Ok(crop)
}

pub fn effect_box_crop(rect: &Rect, decoration: &BoxDecoration) -> Result<Rect, PdfError> {
    expand_rect_for_shadow(rect.clone(), decoration)
}

fn backdrop_color_ops(
    ops: &[PaintOp],
    anchor: usize,
    follow: Option<usize>,
    crop: &Rect,
) -> Vec<PaintOp> {
    let mut out = Vec::new();
    for (i, op) in ops.iter().enumerate() {
        if i == anchor || Some(i) == follow {
            out.push(op.clone());
            continue;
        }
        if i > anchor {
            continue;
        }
        if op_intersects(op, crop) {
            out.push(op.clone());
        }
    }
    out
}

fn op_intersects(op: &PaintOp, crop: &Rect) -> bool {
    match op {
        PaintOp::BackdropBlur { rect, .. }
        | PaintOp::DrawBox { rect, .. }
        | PaintOp::DrawText { rect, .. }
        | PaintOp::DrawImage { rect, .. }
        | PaintOp::DrawTableReference { rect, .. } => rects_intersect(rect, crop),
        PaintOp::Unknown => false,
    }
}

fn is_rule_id(node_id: &str) -> bool {
    node_id.contains("::rule_")
}

fn alpha_below_opaque(color: &str) -> bool {
    parse_hex_rgba(color).is_some_and(|px| px[3] < 255)
}

fn border_alpha_below_opaque(border: Option<&Border>) -> bool {
    border.is_some_and(|b| alpha_below_opaque(&b.color))
}

fn union_rect(a: Rect, b: Rect) -> Rect {
    let x1 = a.x.0.min(b.x.0);
    let y1 = a.y.0.min(b.y.0);
    let x2 = (a.x.0 + a.width.0).max(b.x.0 + b.width.0);
    let y2 = (a.y.0 + a.height.0).max(b.y.0 + b.height.0);
    Rect {
        x: Pt(x1),
        y: Pt(y1),
        width: Pt((x2 - x1).max(1)),
        height: Pt((y2 - y1).max(1)),
    }
}

pub fn clamp_rect_to_page(rect: Rect, page_w: Pt, page_h: Pt) -> Rect {
    let x = rect.x.0.max(0);
    let y = rect.y.0.max(0);
    let x2 = (rect.x.0 + rect.width.0).min(page_w.0).max(x + 1);
    let y2 = (rect.y.0 + rect.height.0).min(page_h.0).max(y + 1);
    Rect {
        x: Pt(x),
        y: Pt(y),
        width: Pt(x2 - x),
        height: Pt(y2 - y),
    }
}

fn expand_rect_for_shadow(rect: Rect, decoration: &BoxDecoration) -> Result<Rect, PdfError> {
    let Some(shadow) = decoration.shadow.as_ref() else {
        return Ok(rect);
    };
    let layers = match shadow {
        ShadowRef::Inline(Shadow { layers }) => layers,
        ShadowRef::Ref(name) => {
            return Err(PdfError::Write(format!("unresolved shadow ref '{name}'")));
        }
    };
    if layers.is_empty() {
        return Ok(rect);
    }
    let mut pad_l = 0i128;
    let mut pad_t = 0i128;
    let mut pad_r = 0i128;
    let mut pad_b = 0i128;
    for layer in layers {
        let extra = i128::from(layer.blur_radius_pt.max(0) + layer.spread_radius_pt.max(0));
        pad_l = pad_l.max(extra + i128::from((-layer.offset_x_pt).max(0)));
        pad_r = pad_r.max(extra + i128::from(layer.offset_x_pt.max(0)));
        pad_t = pad_t.max(extra + i128::from((-layer.offset_y_pt).max(0)));
        pad_b = pad_b.max(extra + i128::from(layer.offset_y_pt.max(0)));
    }
    Ok(Rect {
        x: Pt(rect.x.0 - pad_l),
        y: Pt(rect.y.0 - pad_t),
        width: Pt(rect.width.0 + pad_l + pad_r),
        height: Pt(rect.height.0 + pad_t + pad_b),
    })
}

fn rects_intersect(a: &Rect, b: &Rect) -> bool {
    let ax2 = a.x.0 + a.width.0;
    let ay2 = a.y.0 + a.height.0;
    let bx2 = b.x.0 + b.width.0;
    let by2 = b.y.0 + b.height.0;
    a.x.0 < bx2 && b.x.0 < ax2 && a.y.0 < by2 && b.y.0 < ay2
}
