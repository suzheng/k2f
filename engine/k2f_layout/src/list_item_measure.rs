use crate::resolved_style::resolve_list_style;
use crate::LayoutContext;
use k2f_core::{Pt, SemanticNode};

/// Geometry inputs needed to measure a list item deterministically.
///
/// Measurement must not depend on the marker label text (e.g. "9." vs "10.").
/// Instead, the theme provides a fixed marker box width, plus a deterministic gap and per-depth indent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListItemMeasureSpec {
    pub indent: Pt,
    pub marker_box_width: Pt,
    pub marker_gap: Pt,
}

impl ListItemMeasureSpec {
    pub fn leading_width(self) -> Pt {
        self.indent + self.marker_box_width + self.marker_gap
    }
}

pub fn list_item_measure_spec(
    node: &SemanticNode,
    ctx: &LayoutContext,
) -> Result<ListItemMeasureSpec, String> {
    let style =
        resolve_list_style(&node.role, node.variant.as_deref(), ctx.theme).ok_or_else(|| {
            format!(
            "Missing list_style for role '{}' (required to measure list items deterministically)",
            node.role
        )
        })?;

    let marker_box_width = style.marker_box_width_pt.ok_or_else(|| {
        "list_style.marker_box_width_pt is required for list item measurement".to_string()
    })?;
    let marker_gap = style.marker_gap_pt.ok_or_else(|| {
        "list_style.marker_gap_pt is required for list item measurement".to_string()
    })?;
    let depth_indent = style.depth_indent_pt.ok_or_else(|| {
        "list_style.depth_indent_pt is required for list item measurement".to_string()
    })?;

    let depth = node.depth.unwrap_or(0) as i128;
    let indent = depth_indent * depth;

    Ok(ListItemMeasureSpec {
        indent,
        marker_box_width,
        marker_gap,
    })
}
