//! Post-layout diagnostics. Not part of the lock or appearance hash.

use crate::fixed_size::fixed_size_hint;
use crate::pagination::content_height;
use crate::resolved_style::padding_for_role_variant;
use crate::theme::Theme;
use k2f_core::{collect_boxes, for_each_node, GeometryNode, LayoutResult, Manifest, Pt};
use std::fmt;

/// Minimum share of the page content box a height-pinned node must occupy
/// before unused space is reported (skips small cards / `ex_grid` demos).
const MIN_HEIGHT_NUM: i128 = 40;
const MIN_HEIGHT_DEN: i128 = 100;
/// Unused below as a share of the inner box.
const UNUSED_NUM: i128 = 12;
const UNUSED_DEN: i128 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutDiag {
    pub node_id: String,
    pub unused_below: Pt,
    pub unused_above: Pt,
    pub inner_height: Pt,
    pub after_id: Option<String>,
}

impl LayoutDiag {
    pub fn unused_pct(&self) -> i128 {
        if self.inner_height.0 <= 0 {
            0
        } else {
            self.unused_below.0 * 100 / self.inner_height.0
        }
    }
}

impl fmt::Display for LayoutDiag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let after = self.after_id.as_deref().unwrap_or("-");
        write!(
            f,
            "LAYOUT_SLACK id={} unused_below={} ({}%) unused_above={} after={} hint=use {{fr:1}} body row; do not add spacer nodes",
            self.node_id,
            self.unused_below.0,
            self.unused_pct(),
            self.unused_above.0,
            after
        )
    }
}

pub fn layout_slack_diags(
    manifest: &Manifest,
    layout: &LayoutResult,
    theme: &Theme,
) -> Vec<LayoutDiag> {
    let content_h = content_height(&manifest.page_config);
    if content_h.0 <= 0 {
        return vec![];
    }
    let min_h = Pt(content_h.0 * MIN_HEIGHT_NUM / MIN_HEIGHT_DEN);

    let mut diags = Vec::new();
    for_each_node(&manifest.root, &mut |node| {
        let Some(h) = fixed_size_hint(&node.layout).height else {
            return;
        };
        if h.0 < min_h.0 {
            return;
        }
        let Some(geo) = find_geo(layout, &node.id) else {
            return;
        };
        let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), theme)
            .unwrap_or(crate::resolved_style::EdgeInsets::ZERO);
        if let Some(diag) = slack_for_box(&node.id, geo, padding) {
            diags.push(diag);
        }
    });
    diags
}

fn find_geo<'a>(layout: &'a LayoutResult, id: &str) -> Option<&'a GeometryNode> {
    for page in &layout.pages {
        let mut boxes = Vec::new();
        collect_boxes(&page.root, id, &mut boxes);
        if let Some(geo) = boxes.first().copied() {
            return Some(geo);
        }
    }
    None
}

fn slack_for_box(
    id: &str,
    geo: &GeometryNode,
    padding: crate::resolved_style::EdgeInsets,
) -> Option<LayoutDiag> {
    let inner_h = Pt((geo.height.0 - padding.vertical().0).max(0));
    if inner_h.0 <= 0 {
        return None;
    }
    let inner_top = geo.y + padding.top;
    let inner_bottom = inner_top + inner_h;

    let (unused_above, unused_below, after_id) = if geo.children.is_empty() {
        (Pt::ZERO, inner_h, None)
    } else {
        let first = geo.children.first().unwrap();
        let last = geo.children.last().unwrap();
        let above = Pt((first.y.0 - inner_top.0).max(0));
        let below = Pt((inner_bottom.0 - last.y.0 - last.height.0).max(0));
        (above, below, Some(last.id.clone()))
    };

    if unused_below.0 * UNUSED_DEN < inner_h.0 * UNUSED_NUM {
        return None;
    }
    if unused_above.0 > 0 && unused_below.0 <= unused_above.0 * 2 {
        return None;
    }

    Some(LayoutDiag {
        node_id: id.to_string(),
        unused_below,
        unused_above,
        inner_height: inner_h,
        after_id,
    })
}
