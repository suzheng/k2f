use k2f_core::{BoxDecoration, Fill, GeometryNode, Page, PaintOp, Pt, Rect};
use k2f_paint::{parse_hex_rgba, resolve_fill};

/// Fills thinner than this stay in front of the document. LibreOffice Writer
/// paints `behindDoc` shapes under `w:background`, so 1 pt rules and a few-pt
/// accent bars would vanish if they sat behind. Paper-colored card/cell fills
/// stay behind; contrasting large fills also stay in front.
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

fn opaque_srgb_hex(color: &str) -> Option<String> {
    let [r, g, b, a] = parse_hex_rgba(color)?;
    if a < 255 {
        return None;
    }
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
    fn thin_fill_rect_detects_rules() {
        assert!(is_thin_fill_rect(&r(0, 0, 114_000, 1_000)));
        assert!(is_thin_fill_rect(&r(0, 0, 6_000, 25_000)));
        assert!(!is_thin_fill_rect(&r(0, 0, 150_000, 86_000)));
    }
}
