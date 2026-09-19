use crate::PptxError;
use k2f_core::{BoxDecoration, Fill, NodeContent, Pt, Rect, SemanticNode, ROLE_MATH};
use k2f_paint::resolve_fill;
use std::collections::HashSet;

pub fn is_rule_id(node_id: &str) -> bool {
    node_id.contains("::rule_")
}

pub fn is_math_node(node: &SemanticNode) -> bool {
    node.role == ROLE_MATH || matches!(node.content, NodeContent::Math(_))
}

pub fn box_belongs_to_effect(node_id: &str, effect_ids: &HashSet<String>) -> bool {
    if effect_ids.contains(node_id) {
        return true;
    }
    effect_ids.iter().any(|id| {
        node_id.len() > id.len()
            && node_id.starts_with(id.as_str())
            && node_id[id.len()..].starts_with("::rule_")
    })
}

pub fn is_full_page(page_w: Pt, page_h: Pt, rect: &Rect) -> bool {
    rect.x == Pt(0) && rect.y == Pt(0) && rect.width == page_w && rect.height == page_h
}

pub fn box_is_effect(node_id: &str, decoration: &BoxDecoration) -> Result<bool, PptxError> {
    if is_rule_id(node_id) {
        return Ok(false);
    }
    // Shadow-only boxes stay native fill+stroke. An opaque shadow PNG is
    // expanded for blur/spread, so it covers earlier labels that sit in the
    // glow halo (invoice totals, raised plaques). Same iceberg as Word.
    // Keep rasters for blur (no native equivalent).
    if decoration.blur.is_some() {
        return Ok(true);
    }
    // Linear gradients and translucent solids are native DrawingML fills
    // (`a:gradFill` / `a:solidFill`+`a:alpha`) — same as Word. Rasterizing
    // a rounded gradient as opaque RGB put a page-background AABB around
    // pills and clipped transparent SVG corners.
    match resolve_fill(decoration) {
        Ok(Some(Fill::LinearGradient { .. })) | Ok(Some(Fill::Solid { .. })) | Ok(None) => {
            Ok(false)
        }
        Err(k2f_paint::PaintError::UnresolvedRef(name)) => {
            Err(PptxError::Write(format!("unresolved fill ref '{name}'")))
        }
        Err(e) => Err(e.into()),
    }
}
