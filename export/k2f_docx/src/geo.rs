use k2f_core::{Fill, GeometryNode, Page, PaintOp, Pt, Rect};
use k2f_paint::{parse_hex_rgba, resolve_fill};

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

fn opaque_srgb_hex(color: &str) -> Option<String> {
    let [r, g, b, a] = parse_hex_rgba(color)?;
    if a < 255 {
        return None;
    }
    Some(format!("{r:02X}{g:02X}{b:02X}"))
}
