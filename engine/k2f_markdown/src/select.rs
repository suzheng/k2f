use crate::emit::{nodes_to_markdown, MarkdownEmitOptions};
use crate::slice::slice_text_node;
use k2f_core::{
    NodeContent, RunningBlockNode, SemanticNode, TableDataSource, TableSpec,
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NodeCharRange {
    pub node_id: String,
    pub char_start: usize,
    pub char_end: usize,
}

/// Build a filtered semantic subtree from character ranges, then emit Markdown.
pub fn selection_to_markdown(
    root: &SemanticNode,
    running: &[RunningBlockNode],
    ranges: &[NodeCharRange],
    opts: MarkdownEmitOptions,
) -> String {
    if ranges.is_empty() {
        return String::new();
    }
    let by_id: HashMap<&str, &NodeCharRange> = ranges.iter().map(|r| (r.node_id.as_str(), r)).collect();
    let selected: HashSet<&str> = by_id.keys().copied().collect();

    let mut nodes = Vec::new();
    if let Some(n) = filter_node(root, &by_id, &selected) {
        if let NodeContent::Container { children } = n.content {
            nodes = children;
        } else {
            nodes.push(n);
        }
    }
    for rb in running {
        if selected.contains(rb.node.id.as_str()) {
            if let Some(n) = filter_node(&rb.node, &by_id, &selected) {
                nodes.push(n);
            }
        }
    }
    if nodes.is_empty() {
        return String::new();
    }
    nodes_to_markdown(&nodes, opts)
}

fn filter_node(
    node: &SemanticNode,
    by_id: &HashMap<&str, &NodeCharRange>,
    selected: &HashSet<&str>,
) -> Option<SemanticNode> {
    match &node.content {
        NodeContent::Container { children } => {
            let kids: Vec<_> = children
                .iter()
                .filter_map(|c| filter_node(c, by_id, selected))
                .collect();
            if kids.is_empty() && !selected.contains(node.id.as_str()) {
                return None;
            }
            let mut out = node.clone();
            out.content = NodeContent::Container { children: kids };
            Some(out)
        }
        NodeContent::Table(spec) => project_table(node, spec, by_id, selected),
        NodeContent::Text(_) | NodeContent::Math(_) => {
            let r = by_id.get(node.id.as_str())?;
            Some(slice_text_node(node, r.char_start, r.char_end))
        }
        _ => {
            if selected.contains(node.id.as_str()) {
                Some(node.clone())
            } else {
                None
            }
        }
    }
}

fn project_table(
    table: &SemanticNode,
    spec: &TableSpec,
    by_id: &HashMap<&str, &NodeCharRange>,
    selected: &HashSet<&str>,
) -> Option<SemanticNode> {
    let TableDataSource::Inline { rows } = &spec.data else {
        return None;
    };
    if rows.is_empty() {
        return None;
    }

    let mut hits: Vec<(usize, usize, SemanticNode)> = Vec::new();
    for (ri, row) in rows.iter().enumerate() {
        for (ci, cell) in row.iter().enumerate() {
            if let Some(r) = by_id.get(cell.id.as_str()) {
                hits.push((ri, ci, slice_text_node(cell, r.char_start, r.char_end)));
            } else if cell_has_selected_descendant(cell, selected) {
                // Nested content inside a cell — rare; keep whole cell if any descendant selected.
                if let Some(filtered) = filter_node(cell, by_id, selected) {
                    hits.push((ri, ci, filtered));
                }
            }
        }
    }
    if hits.is_empty() {
        return None;
    }

    let mut row_set: HashSet<usize> = hits.iter().map(|(r, _, _)| *r).collect();
    let col_set: HashSet<usize> = hits.iter().map(|(_, c, _)| *c).collect();
    let header_n = spec.header_rows.min(rows.len());
    // Always include original header rows for selected columns so GFM has a header.
    if header_n > 0 {
        for ri in 0..header_n {
            row_set.insert(ri);
        }
    }

    let mut keep_rows: Vec<usize> = row_set.into_iter().collect();
    keep_rows.sort_unstable();
    let mut keep_cols: Vec<usize> = col_set.into_iter().collect();
    keep_cols.sort_unstable();

    let hit_map: HashMap<(usize, usize), SemanticNode> =
        hits.into_iter().map(|(r, c, n)| ((r, c), n)).collect();

    let new_rows: Vec<Vec<SemanticNode>> = keep_rows
        .iter()
        .map(|&ri| {
            keep_cols
                .iter()
                .map(|&ci| {
                    if let Some(n) = hit_map.get(&(ri, ci)) {
                        n.clone()
                    } else if ri < header_n {
                        // Header fill from original (unsliced) for GFM structure.
                        rows[ri].get(ci).cloned().unwrap_or_else(empty_cell)
                    } else {
                        empty_cell()
                    }
                })
                .collect()
        })
        .collect();

    let new_widths: Vec<_> = keep_cols
        .iter()
        .filter_map(|&ci| spec.column_widths.get(ci).cloned())
        .collect();
    let widths = if new_widths.len() == keep_cols.len() {
        new_widths
    } else {
        keep_cols
            .iter()
            .map(|_| k2f_core::GridTrack::Fr { fr: 1 })
            .collect()
    };

    let mut out = table.clone();
    out.content = NodeContent::Table(TableSpec {
        column_widths: widths,
        header_rows: if header_n > 0 { 1.min(new_rows.len()) } else { 0 },
        gap: spec.gap,
        row_gap: spec.row_gap,
        column_gap: spec.column_gap,
        data: TableDataSource::Inline { rows: new_rows },
    });
    Some(out)
}

fn cell_has_selected_descendant(cell: &SemanticNode, selected: &HashSet<&str>) -> bool {
    if selected.contains(cell.id.as_str()) {
        return true;
    }
    let mut found = false;
    k2f_core::for_each_node(cell, &mut |n| {
        if selected.contains(n.id.as_str()) {
            found = true;
        }
    });
    found
}

fn empty_cell() -> SemanticNode {
    SemanticNode {
        id: String::new(),
        role: "table_row_cell".into(),
        content: NodeContent::Text(String::new()),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::apply_inline_markdown;
    use k2f_core::{GridTrack, Modifier};

    fn text(id: &str, role: &str, s: &str) -> SemanticNode {
        SemanticNode {
            id: id.into(),
            role: role.into(),
            content: NodeContent::Text(s.into()),
            ..Default::default()
        }
    }

    fn table_3x3() -> SemanticNode {
        let rows = vec![
            vec![
                text("t.h.c0", "table_header_cell", "A"),
                text("t.h.c1", "table_header_cell", "B"),
                text("t.h.c2", "table_header_cell", "C"),
            ],
            vec![
                text("t.r0.c0", "table_row_cell", "a0"),
                text("t.r0.c1", "table_row_cell", "b0"),
                text("t.r0.c2", "table_row_cell", "c0"),
            ],
            vec![
                text("t.r1.c0", "table_row_cell", "a1"),
                text("t.r1.c1", "table_row_cell", "b1"),
                text("t.r1.c2", "table_row_cell", "c1"),
            ],
        ];
        SemanticNode {
            id: "t".into(),
            role: "table".into(),
            content: NodeContent::Table(TableSpec {
                column_widths: vec![
                    GridTrack::Fr { fr: 1 },
                    GridTrack::Fr { fr: 1 },
                    GridTrack::Fr { fr: 1 },
                ],
                header_rows: 1,
                gap: 0,
                row_gap: None,
                column_gap: None,
                data: TableDataSource::Inline { rows },
            }),
            ..Default::default()
        }
    }

    #[test]
    fn subtable_projects_selected_cells_with_header() {
        let root = SemanticNode {
            id: "root".into(),
            role: "document".into(),
            content: NodeContent::Container {
                children: vec![table_3x3()],
            },
            ..Default::default()
        };
        let md = selection_to_markdown(
            &root,
            &[],
            &[
                NodeCharRange {
                    node_id: "t.r1.c0".into(),
                    char_start: 0,
                    char_end: 2,
                },
                NodeCharRange {
                    node_id: "t.r1.c1".into(),
                    char_start: 0,
                    char_end: 2,
                },
            ],
            MarkdownEmitOptions::clipboard(),
        );
        assert!(md.contains("| A | B |"), "got {md}");
        assert!(md.contains("---"), "got {md}");
        assert!(md.contains("| a1 | b1 |"), "got {md}");
        assert!(!md.contains("c1"), "must not leak unselected col: {md}");
        assert!(!md.contains("a0"), "must not leak unselected row: {md}");
    }

    #[test]
    fn hints_off_skips_warning_comment() {
        let mut n = text("w", "warning", "careful");
        n.variant = Some("critical".into());
        let root = SemanticNode {
            id: "root".into(),
            role: "document".into(),
            content: NodeContent::Container {
                children: vec![n],
            },
            ..Default::default()
        };
        let md = selection_to_markdown(
            &root,
            &[],
            &[NodeCharRange {
                node_id: "w".into(),
                char_start: 0,
                char_end: 7,
            }],
            MarkdownEmitOptions::clipboard(),
        );
        assert!(!md.contains("<!--"), "got {md}");
        assert!(md.contains("careful"), "got {md}");
    }

    #[test]
    fn slices_emphasis_in_selection() {
        let mut n = text("b", "body", "你好世界");
        n.modifiers = vec![Modifier {
            range: [3, 6],
            mod_type: "emphasis".into(),
            intent: "strong".into(),
        }];
        let root = SemanticNode {
            id: "root".into(),
            role: "document".into(),
            content: NodeContent::Container {
                children: vec![n],
            },
            ..Default::default()
        };
        let md = selection_to_markdown(
            &root,
            &[],
            &[NodeCharRange {
                node_id: "b".into(),
                char_start: 1,
                char_end: 2,
            }],
            MarkdownEmitOptions::clipboard(),
        );
        assert_eq!(md.trim(), "**好**");
        let _ = apply_inline_markdown;
    }
}
