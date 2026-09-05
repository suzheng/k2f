mod build;

use k2f_core::{
    for_each_node, NodeContent, PaintOp, RunningBlockNode, SemanticNode, TableDataSource, TableSpec,
};
use std::collections::HashMap;

pub(crate) use build::{table_on_page, table_ref_placeholder};

#[derive(Default)]
pub(crate) struct TableIndex {
    pub(crate) tables: HashMap<String, Harvested>,
    owner: HashMap<String, String>,
}

pub(crate) struct Harvested {
    pub(crate) column_widths: Vec<k2f_core::GridTrack>,
    pub(crate) header_rows: usize,
}

/// `Some(rows)` means emit `a:tbl`. `None` means leave the table as box+text+image.
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
                    column_widths: spec.column_widths.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::{GridTrack, NodeContent, Pt, SemanticNode, TableDataSource, TableSpec};

    fn text_cell(id: &str) -> SemanticNode {
        SemanticNode {
            id: id.into(),
            role: "body".into(),
            content: NodeContent::Text("x".into()),
            ..Default::default()
        }
    }

    #[test]
    fn nested_or_image_cell_does_not_corrupt_file() {
        let image = TableSpec {
            column_widths: vec![GridTrack::Fr { fr: 1 }],
            header_rows: 0,
            gap: 0,
            data: TableDataSource::Inline {
                rows: vec![vec![SemanticNode {
                    id: "img".into(),
                    role: "body".into(),
                    content: NodeContent::Image {
                        src: "assets/x.png".into(),
                        width: Pt(10_000),
                        height: Pt(10_000),
                    },
                    ..Default::default()
                }]],
            },
        };
        assert!(harvestable_inline_rows(&image).is_none());
        let nested = TableSpec {
            column_widths: vec![GridTrack::Fr { fr: 1 }],
            header_rows: 0,
            gap: 0,
            data: TableDataSource::Inline {
                rows: vec![vec![SemanticNode {
                    id: "wrap".into(),
                    role: "body".into(),
                    content: NodeContent::Container {
                        children: vec![text_cell("inner")],
                    },
                    ..Default::default()
                }]],
            },
        };
        assert!(harvestable_inline_rows(&nested).is_none());
        let plain = TableSpec {
            column_widths: vec![GridTrack::Fr { fr: 1 }],
            header_rows: 0,
            gap: 0,
            data: TableDataSource::Inline {
                rows: vec![vec![text_cell("c")]],
            },
        };
        assert!(harvestable_inline_rows(&plain).is_some());
        let asset = TableSpec {
            column_widths: vec![GridTrack::Fr { fr: 1 }],
            header_rows: 0,
            gap: 0,
            data: TableDataSource::Asset {
                source: "assets/data/t.json".into(),
            },
        };
        assert!(harvestable_inline_rows(&asset).is_none());
    }
}
