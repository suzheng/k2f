mod build;
mod geom;
mod paint;
mod xml;

use k2f_core::{
    for_each_node, Border, BorderEdge, BorderStyle, NodeContent, Page, PaintOp, Rect,
    RunningBlockNode, SemanticNode, TableDataSource, TableSpec,
};
use std::collections::{HashMap, HashSet};

pub(crate) use build::table_on_page;
pub use xml::cell_edge_attrs;
pub(crate) use xml::table_xml;

use crate::coord::millipt_to_pt;
use crate::ir::{BorderStroke, CellBorders, LineDash, ShapeBox, TableBox};
use crate::text::TextFonts;
use crate::IdmlError;

#[derive(Default)]
pub(crate) struct TableIndex {
    pub(crate) tables: HashMap<String, Harvested>,
    owner: HashMap<String, String>,
}

pub(crate) struct Harvested {
    pub(crate) column_widths: Vec<k2f_core::GridTrack>,
    pub(crate) header_rows: usize,
}

/// `Some(rows)` means emit a native InDesign `Table`. `None` leaves box+text+image.
pub fn harvestable_inline_rows(spec: &TableSpec) -> Option<&[Vec<SemanticNode>]> {
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

/// Empty `DrawTableReference` placeholder. Gray is hardcoded for this op only;
/// cell borders must not copy it.
pub(crate) fn table_ref_placeholder(node_id: &str, rect: &Rect) -> ShapeBox {
    ShapeBox {
        node_id: node_id.to_string(),
        rect: rect.clone(),
        fill_hex: None,
        fill_alpha: 255,
        gradient: None,
        corner_pt: 0.0,
        line_hex: Some("808080".into()),
        line_alpha: 255,
        line_w_pt: 1.0,
        line_dash: LineDash::Solid,
    }
}

pub fn cell_borders(border: Option<&Border>) -> Result<CellBorders, IdmlError> {
    let Some(b) = border else {
        return Ok(CellBorders::default());
    };
    if b.width_pt <= 0 || b.edges.is_empty() {
        return Ok(CellBorders::default());
    }
    let [r, g, bl, a] = k2f_paint::parse_hex_rgba(&b.color)
        .ok_or_else(|| IdmlError::Write(format!("unparseable color '{}'", b.color)))?;
    if a == 0 {
        return Ok(CellBorders::default());
    }
    let stroke = BorderStroke {
        color_hex: format!("{r:02X}{g:02X}{bl:02X}"),
        weight_pt: millipt_to_pt(b.width_pt as i128),
        dash: match b.style {
            BorderStyle::Solid => LineDash::Solid,
            BorderStyle::Dashed => LineDash::Dash,
            BorderStyle::Dotted => LineDash::Dot,
        },
    };
    Ok(CellBorders {
        top: edge(b, BorderEdge::Top, &stroke),
        left: edge(b, BorderEdge::Left, &stroke),
        bottom: edge(b, BorderEdge::Bottom, &stroke),
        right: edge(b, BorderEdge::Right, &stroke),
    })
}

fn edge(b: &Border, e: BorderEdge, stroke: &BorderStroke) -> Option<BorderStroke> {
    b.draws_edge(e).then(|| stroke.clone())
}

pub(crate) enum TakeTable {
    NotMember,
    Skip,
    Built(TableBox),
}

pub(crate) fn take_table(
    op: &PaintOp,
    tables: &TableIndex,
    emitted: &mut HashSet<String>,
    page: &Page,
    ops: &[PaintOp],
    root: &SemanticNode,
    running: &[RunningBlockNode],
    fonts: &TextFonts,
) -> Result<TakeTable, IdmlError> {
    let Some(tid) = paint_node_id(op).and_then(|id| tables.owner_of(id)) else {
        return Ok(TakeTable::NotMember);
    };
    if emitted.contains(tid) {
        return Ok(TakeTable::Skip);
    }
    if let Some(tbl) = table_on_page(tables, tid, page, ops, root, running, fonts)? {
        emitted.insert(tid.to_string());
        return Ok(TakeTable::Built(tbl));
    }
    Ok(TakeTable::NotMember)
}
