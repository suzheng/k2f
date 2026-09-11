use crate::DocxError;
use k2f_core::{BoxDecoration, NodeContent, SemanticNode, ROLE_MATH};
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

pub fn box_is_effect(node_id: &str, decoration: &BoxDecoration) -> Result<bool, DocxError> {
    if is_rule_id(node_id) {
        return Ok(false);
    }
    // Gradients and translucent solids are native DrawingML fills. Rasterizing
    // them as pictures makes LibreOffice Writer paint the slice above later
    // text. Shadow-only boxes stay native too: an opaque shadow PNG covers
    // later labels even as wps:wsp. Keep rasters for blur (no native equivalent).
    Ok(decoration.blur.is_some())
}
