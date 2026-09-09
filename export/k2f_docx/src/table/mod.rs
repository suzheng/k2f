mod build;
mod cell;
mod xml;

use k2f_core::{
    for_each_node, NodeContent, PaintOp, RunningBlockNode, SemanticNode, TableDataSource, TableSpec,
};
use std::collections::HashMap;

pub(crate) use build::table_ref_placeholder;
pub(crate) use xml::table_anchor;
pub use xml::table_cell_wml;

#[derive(Default)]
#[allow(dead_code)]
pub(crate) struct TableIndex {
    tables: HashMap<String, Harvested>,
    owner: HashMap<String, String>,
}

pub(crate) struct Harvested {
    ncols: usize,
    header_rows: usize,
}

pub fn can_emit_native_table(spec: &TableSpec) -> bool {
    harvestable_inline_rows(spec).is_some()
}

pub(crate) fn harvestable_inline_rows(spec: &TableSpec) -> Option<&[Vec<SemanticNode>]> {
    match &spec.data {
        TableDataSource::Inline { rows } if cells_are_plain_text(rows) => Some(rows),
        _ => None,
    }
}

fn cells_are_plain_text(rows: &[Vec<SemanticNode>]) -> bool {
    rows.iter()
        .flatten()
        .all(|cell| matches!(cell.content, NodeContent::Text(_)))
}

#[allow(dead_code)]
pub(crate) fn index_native_tables(root: &SemanticNode, running: &[RunningBlockNode]) -> TableIndex {
    let mut idx = TableIndex::default();
    collect(root, &mut idx);
    for rb in running {
        collect(&rb.node, &mut idx);
    }
    idx
}

fn collect(node: &SemanticNode, idx: &mut TableIndex) {
    if let NodeContent::Table(spec) = &node.content {
        if let Some(rows) = harvestable_inline_rows(spec) {
            idx.owner.insert(node.id.clone(), node.id.clone());
            for cell in rows.iter().flatten() {
                for_each_node(cell, &mut |n| {
                    idx.owner.insert(n.id.clone(), node.id.clone());
                });
            }
            idx.tables.insert(
                node.id.clone(),
                Harvested {
                    ncols: spec.column_widths.len(),
                    header_rows: spec.header_rows,
                },
            );
        }
    }
    match &node.content {
        NodeContent::Container { children } => {
            for child in children {
                collect(child, idx);
            }
        }
        NodeContent::Table(spec) => {
            if let TableDataSource::Inline { rows } = &spec.data {
                for cell in rows.iter().flatten() {
                    collect(cell, idx);
                }
            }
        }
        _ => {}
    }
}

impl TableIndex {
    #[allow(dead_code)]
    pub(crate) fn owner_of(&self, node_id: &str) -> Option<&str> {
        self.owner.get(node_id).map(String::as_str)
    }
}

pub(crate) fn paint_node_id(op: &PaintOp) -> Option<&str> {
    match op {
        PaintOp::BackdropBlur { node_id, .. }
        | PaintOp::DrawBox { node_id, .. }
        | PaintOp::DrawText { node_id, .. }
        | PaintOp::DrawImage { node_id, .. }
        | PaintOp::DrawTableReference { node_id, .. } => Some(node_id.as_str()),
        PaintOp::Unknown => None,
    }
}
