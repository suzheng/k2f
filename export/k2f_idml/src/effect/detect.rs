use crate::IdmlError;
use k2f_core::{
    BoxDecoration, Fill, NodeContent, Pt, Rect, SemanticNode, Shadow, ShadowRef, ROLE_MATH,
};
use k2f_paint::{parse_hex_rgba, resolve_fill};
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

/// Boxes that cannot be native InDesign rectangles: blur, engine shadow,
/// linear gradient, or translucent solid. Fraction rules stay with math.
pub fn box_is_effect(node_id: &str, decoration: &BoxDecoration) -> Result<bool, IdmlError> {
    if is_rule_id(node_id) {
        return Ok(false);
    }
    if decoration.blur.is_some() {
        return Ok(true);
    }
    if has_engine_shadow(decoration) {
        return Ok(true);
    }
    match resolve_fill(decoration) {
        Ok(Some(Fill::LinearGradient { .. })) => Ok(true),
        Ok(Some(Fill::Solid { color })) => translucent_solid(&color),
        Ok(None) => Ok(false),
        Err(k2f_paint::PaintError::UnresolvedRef(name)) => {
            Err(IdmlError::Write(format!("unresolved fill ref '{name}'")))
        }
        Err(e) => Err(e.into()),
    }
}

pub fn has_engine_shadow(decoration: &BoxDecoration) -> bool {
    match &decoration.shadow {
        Some(ShadowRef::Inline(Shadow { layers })) => !layers.is_empty(),
        Some(ShadowRef::Ref(_)) => true,
        None => false,
    }
}

fn translucent_solid(color: &str) -> Result<bool, IdmlError> {
    let [_, _, _, a] = parse_hex_rgba(color)
        .ok_or_else(|| IdmlError::Write(format!("unparseable fill color '{color}'")))?;
    Ok(a < 255)
}
