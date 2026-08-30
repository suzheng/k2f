use crate::resolved_style::resolve_list_style;
use crate::Theme;
use k2f_core::{ListMarkerType, NodeContent, SemanticNode, TableDataSource};
use std::collections::HashMap;

/// Derived marker labels for semantic list items, keyed by node id.
///
/// Marker labels are derived from each node-stream (e.g. a container's children slice),
/// and are intended to be used at arrangement time without mutating the semantic tree.
pub type ListMarkerMap = HashMap<String, String>;

#[derive(Debug, Default, Clone)]
struct ListIdNumberState {
    counters_by_depth: Vec<u32>,
}

#[derive(Debug, Default, Clone)]
struct StreamListState {
    // Per list_id numbering state, scoped to a single stream.
    by_list_id: HashMap<String, ListIdNumberState>,
}

pub fn derive_list_marker_map(root: &SemanticNode, theme: &Theme) -> Result<ListMarkerMap, String> {
    let mut out: ListMarkerMap = HashMap::new();
    walk_node(root, theme, &mut out)?;
    Ok(out)
}

fn walk_node(node: &SemanticNode, theme: &Theme, out: &mut ListMarkerMap) -> Result<(), String> {
    match &node.content {
        NodeContent::Container { children } => {
            scan_stream(children, theme, out)?;
            for child in children {
                walk_node(child, theme, out)?;
            }
        }
        NodeContent::Table(spec) => {
            // Tables contain normal SemanticNodes inside cells; treat each row as its own stream.
            if let TableDataSource::Inline { rows } = &spec.data {
                for row in rows {
                    scan_stream(row, theme, out)?;
                    for cell in row {
                        walk_node(cell, theme, out)?;
                    }
                }
            }
        }
        NodeContent::Text(_)
        | NodeContent::CodeBlock(_)
        | NodeContent::Math(_)
        | NodeContent::Image { .. }
        | NodeContent::TableReference { .. } => {}
    }
    Ok(())
}

fn scan_stream(
    nodes: &[SemanticNode],
    theme: &Theme,
    out: &mut ListMarkerMap,
) -> Result<(), String> {
    let mut state = StreamListState::default();

    for node in nodes {
        if node.role == "list_item" {
            let list_id = node
                .list_id
                .as_deref()
                .ok_or_else(|| format!("list_item '{}' is missing list_id", node.id))?;

            let marker_type = node.marker_type.unwrap_or(ListMarkerType::Bullet);
            let label = match marker_type {
                ListMarkerType::Bullet => resolve_bullet_label(node, theme)?,
                ListMarkerType::Number => resolve_number_label(node, list_id, theme, &mut state)?,
            };

            out.insert(node.id.clone(), label);
        }
    }

    Ok(())
}

fn resolve_bullet_label(node: &SemanticNode, theme: &Theme) -> Result<String, String> {
    let style =
        resolve_list_style(&node.role, node.variant.as_deref(), theme).ok_or_else(|| {
            format!(
                "Missing list_style for role '{}' (required to derive list marker labels)",
                node.role
            )
        })?;
    Ok(style.bullet_glyph.unwrap_or_else(|| "•".to_string()))
}

fn resolve_number_label(
    node: &SemanticNode,
    list_id: &str,
    theme: &Theme,
    state: &mut StreamListState,
) -> Result<String, String> {
    let style =
        resolve_list_style(&node.role, node.variant.as_deref(), theme).ok_or_else(|| {
            format!(
                "Missing list_style for role '{}' (required to derive list marker labels)",
                node.role
            )
        })?;
    let suffix = style.number_suffix.unwrap_or_else(|| ".".to_string());

    let depth = node.depth.unwrap_or(0) as usize;
    let entry = state
        .by_list_id
        .entry(list_id.to_string())
        .or_insert_with(ListIdNumberState::default);

    let n = advance_number(&mut entry.counters_by_depth, depth)?;
    Ok(format!("{}{}", n, suffix))
}

fn advance_number(counters_by_depth: &mut Vec<u32>, depth: usize) -> Result<u32, String> {
    // Ensure the stack has the desired depth.
    if counters_by_depth.len() <= depth {
        counters_by_depth.resize(depth + 1, 0);
    } else {
        counters_by_depth.truncate(depth + 1);
    }

    let slot = counters_by_depth
        .get_mut(depth)
        .ok_or_else(|| "internal error: counters_by_depth missing depth slot".to_string())?;
    *slot = slot
        .checked_add(1)
        .ok_or_else(|| "list numbering overflowed u32".to_string())?;
    Ok(*slot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::list_style::ListStyle;
    use crate::theme::{RoleStyle, Theme};
    use k2f_core::{NodeContent, Pt, SemanticNode};
    use std::collections::HashMap;

    fn theme_with_list_style() -> Theme {
        let mut roles = HashMap::new();
        roles.insert(
            "list_item".to_string(),
            RoleStyle {
                font_family: "default".to_string(),
                font_size: Pt(12_000),
                line_height_mult: 1_200,
                color: "black".to_string(),
                text_align: crate::style::TextAlign::Start,
                self_align: None,
                box_decoration: None,
                list_style: Some(ListStyle {
                    marker_box_width_pt: Some(Pt(18_000)),
                    marker_gap_pt: Some(Pt(4_000)),
                    depth_indent_pt: Some(Pt(14_000)),
                    bullet_glyph: Some("*".to_string()),
                    number_suffix: Some(".".to_string()),
                    ..ListStyle::default()
                }),
                bold: false,
                italic: false,
                letter_spacing_pt: Pt::ZERO,
            first_line_indent_pt: Pt::ZERO,
                variants: HashMap::new(),
            },
        );

        Theme {
            palette: HashMap::new(),
            primitives: Default::default(),
            roles,
            modifiers: crate::ModifierTheme::default(),
            font_aliases: HashMap::new(),
        }
    }

    #[test]
    fn derives_numbered_markers_with_depth_stack_per_stream() {
        let theme = theme_with_list_style();

        let root = SemanticNode {
            id: "root".to_string(),
            role: "section".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            modifiers: vec![],
            layout: None,
            content: NodeContent::Container {
                children: vec![
                    SemanticNode {
                        id: "a".to_string(),
                        role: "list_item".to_string(),
                        variant: None,
                        preserve_whitespace: None,
                        list_id: Some("L".to_string()),
                        depth: Some(0),
                        marker_type: Some(ListMarkerType::Number),
                        content: NodeContent::Text("One".to_string()),
                        modifiers: vec![],
                        layout: None,
                        ..Default::default()
                    },
                    SemanticNode {
                        id: "b".to_string(),
                        role: "list_item".to_string(),
                        variant: None,
                        preserve_whitespace: None,
                        list_id: Some("L".to_string()),
                        depth: Some(1),
                        marker_type: Some(ListMarkerType::Number),
                        content: NodeContent::Text("Two".to_string()),
                        modifiers: vec![],
                        layout: None,
                        ..Default::default()
                    },
                    SemanticNode {
                        id: "c".to_string(),
                        role: "list_item".to_string(),
                        variant: None,
                        preserve_whitespace: None,
                        list_id: Some("L".to_string()),
                        depth: Some(0),
                        marker_type: Some(ListMarkerType::Number),
                        content: NodeContent::Text("Three".to_string()),
                        modifiers: vec![],
                        layout: None,
                        ..Default::default()
                    },
                ],
            },
            ..Default::default()
        };

        let markers = derive_list_marker_map(&root, &theme).unwrap();
        assert_eq!(markers.get("a").map(String::as_str), Some("1."));
        assert_eq!(markers.get("b").map(String::as_str), Some("1."));
        assert_eq!(markers.get("c").map(String::as_str), Some("2."));
    }
}
