use k2f_core::{BoxDecoration, Fill, GeometryNode, LinearGradient, Page, PaintOp, Pt, Rect};
use k2f_paint::{parse_hex_rgba, resolve_fill};

/// Fills thinner than this stay in front of the document. Page paper is the
/// only behindDoc fill (hosts z-order that stack by size). Paper-colored
/// body fills also sit behind so they cannot cover stamps. Contrasting
/// cards stay in front; classify folds nested labels into the shell.
const THIN_FILL_PT: i128 = 8_000;

pub(crate) fn find_geo<'a>(node: &'a GeometryNode, id: &str) -> Option<&'a GeometryNode> {
    if node.id == id {
        return Some(node);
    }
    node.children.iter().find_map(|c| find_geo(c, id))
}

pub(crate) fn page_bg_hex(page: &Page, ops: &[PaintOp]) -> String {
    match ops.first() {
        Some(PaintOp::DrawBox {
            rect, decoration, ..
        }) if is_full_page_rect(page.width, page.height, rect) => match resolve_fill(decoration) {
            Ok(Some(Fill::Solid { color })) => {
                opaque_srgb_hex(&color).unwrap_or_else(|| "FFFFFF".into())
            }
            Ok(Some(Fill::LinearGradient { value })) => {
                first_gradient_stop_hex(&value).unwrap_or_else(|| "FFFFFF".into())
            }
            _ => "FFFFFF".into(),
        },
        _ => "FFFFFF".into(),
    }
}

pub(crate) fn is_full_page_rect(page_w: Pt, page_h: Pt, rect: &Rect) -> bool {
    rect.x == Pt(0) && rect.y == Pt(0) && rect.width == page_w && rect.height == page_h
}

pub(crate) fn is_thin_fill_rect(rect: &Rect) -> bool {
    rect.width.0.min(rect.height.0) <= THIN_FILL_PT
}

pub(crate) fn is_thin_fill_emu(cx_emu: i64, cy_emu: i64) -> bool {
    let cap = crate::coord::millipt_to_emu(i64::try_from(THIN_FILL_PT).unwrap_or(i64::MAX));
    cx_emu.min(cy_emu) <= cap
}

fn rect_contains(outer: &Rect, inner: &Rect) -> bool {
    inner.x.0 >= outer.x.0
        && inner.y.0 >= outer.y.0
        && inner.x.0.saturating_add(inner.width.0) <= outer.x.0.saturating_add(outer.width.0)
        && inner.y.0.saturating_add(inner.height.0) <= outer.y.0.saturating_add(outer.height.0)
}

fn opaque_solid_fill(decoration: &BoxDecoration) -> bool {
    match resolve_fill(decoration) {
        Ok(Some(Fill::Solid { color })) => {
            parse_hex_rgba(&color).is_some_and(|[_, _, _, a]| a == 255)
        }
        _ => false,
    }
}

/// LibreOffice Writer paints pictures above DrawingML shapes even when the
/// picture's `behindDoc` is set. Skip a lock image that a later opaque box
/// fully covers — the host would otherwise show pixels the lock hid.
pub(crate) fn image_occluded_by_later_opaque_box(rect: &Rect, later: &[PaintOp]) -> bool {
    later.iter().any(|op| match op {
        PaintOp::DrawBox {
            rect: outer,
            decoration,
            ..
        } => opaque_solid_fill(decoration) && rect_contains(outer, rect),
        _ => false,
    })
}

fn rects_intersect(a: &Rect, b: &Rect) -> bool {
    a.x.0 < b.x.0.saturating_add(b.width.0)
        && b.x.0 < a.x.0.saturating_add(a.width.0)
        && a.y.0 < b.y.0.saturating_add(b.height.0)
        && b.y.0 < a.y.0.saturating_add(a.height.0)
}

fn op_rect(op: &PaintOp) -> Option<&Rect> {
    match op {
        PaintOp::DrawText { rect, .. }
        | PaintOp::DrawBox { rect, .. }
        | PaintOp::DrawImage { rect, .. }
        | PaintOp::DrawTableReference { rect, .. }
        | PaintOp::BackdropBlur { rect, .. } => Some(rect),
        PaintOp::Unknown => None,
    }
}

/// Writer paints `pic:pic` above every `wps:wsp`, so a decorative image that
/// later text/shapes sit on must join the shape z-order stack (`wps:wsp` +
/// `a:blipFill`) instead of staying a picture. Logos that do not intersect
/// later paint stay `pic:pic` (replaceable).
pub(crate) fn image_overlapped_by_later_content(rect: &Rect, later: &[PaintOp]) -> bool {
    later
        .iter()
        .any(|op| op_rect(op).is_some_and(|other| rects_intersect(rect, other)))
}

fn opaque_srgb_hex(color: &str) -> Option<String> {
    let [r, g, b, a] = parse_hex_rgba(color)?;
    if a < 255 {
        return None;
    }
    Some(format!("{r:02X}{g:02X}{b:02X}"))
}

fn first_gradient_stop_hex(value: &LinearGradient) -> Option<String> {
    let LinearGradient::Linear { stops, .. } = value;
    let stop = stops.first()?;
    let [r, g, b, _] = parse_hex_rgba(&stop.color)?;
    Some(format!("{r:02X}{g:02X}{b:02X}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::FillRef;

    fn r(x: i128, y: i128, w: i128, h: i128) -> Rect {
        Rect {
            x: Pt(x),
            y: Pt(y),
            width: Pt(w),
            height: Pt(h),
        }
    }

    fn solid(color: &str) -> BoxDecoration {
        BoxDecoration {
            background: Some(FillRef::Inline(Fill::Solid {
                color: color.into(),
            })),
            ..Default::default()
        }
    }

    #[test]
    fn occluded_when_later_opaque_box_covers() {
        let img = r(10, 10, 100, 100);
        let cover = PaintOp::DrawBox {
            node_id: "grid".into(),
            rect: r(0, 0, 200, 200),
            decoration: solid("#FFFFFF"),
        };
        assert!(image_occluded_by_later_opaque_box(&img, &[cover]));
    }

    #[test]
    fn not_occluded_by_transparent_or_partial_cover() {
        let img = r(10, 10, 100, 100);
        let glass = PaintOp::DrawBox {
            node_id: "g".into(),
            rect: r(0, 0, 200, 200),
            decoration: solid("#FFFFFF80"),
        };
        assert!(!image_occluded_by_later_opaque_box(&img, &[glass]));
        let partial = PaintOp::DrawBox {
            node_id: "p".into(),
            rect: r(50, 50, 20, 20),
            decoration: solid("#FFFFFF"),
        };
        assert!(!image_occluded_by_later_opaque_box(&img, &[partial]));
        assert!(!image_occluded_by_later_opaque_box(&img, &[]));
    }

    #[test]
    fn overlapped_when_later_text_intersects() {
        let img = r(0, 0, 400, 300);
        let title = PaintOp::DrawText {
            node_id: "title".into(),
            rect: r(50, 80, 200, 40),
            runs: vec![],
        };
        assert!(image_overlapped_by_later_content(&img, &[title]));
        let aside = PaintOp::DrawText {
            node_id: "aside".into(),
            rect: r(500, 0, 80, 20),
            runs: vec![],
        };
        assert!(!image_overlapped_by_later_content(&img, &[aside]));
        assert!(!image_overlapped_by_later_content(&img, &[]));
    }

    #[test]
    fn thin_fill_rect_detects_rules() {
        assert!(is_thin_fill_rect(&r(0, 0, 114_000, 1_000)));
        assert!(is_thin_fill_rect(&r(0, 0, 6_000, 25_000)));
        assert!(!is_thin_fill_rect(&r(0, 0, 150_000, 86_000)));
    }
}
