use super::cell::{build_cell, cell_paints};
use super::TableIndex;
use crate::coord::{millipt_to_emu, millipt_to_twips, pt_to_emu, pt_to_twips};
use crate::geo::find_geo;
use crate::ir::{LineDash, ShapeBox, TableBox, TableRow};
use crate::text::FontCtx;
use crate::DocxError;
use k2f_core::{GeometryNode, Page, PaintOp, Rect, RunningBlockNode, SemanticNode};

pub(crate) fn table_on_page(
    index: &TableIndex,
    table_id: &str,
    page: &Page,
    ops: &[PaintOp],
    root: &SemanticNode,
    running: &[RunningBlockNode],
    fonts: &FontCtx,
    relative_height: u32,
) -> Result<Option<TableBox>, DocxError> {
    let Some(meta) = index.tables.get(table_id) else {
        return Ok(None);
    };
    let Some(geo) = find_geo(&page.root, table_id) else {
        return Ok(None);
    };
    let ncols = meta.ncols;
    if ncols == 0 {
        return Ok(None);
    }
    let complete = geo.children.len() / ncols;
    if complete == 0 {
        return Ok(None);
    }
    let mut row_slices: Vec<&[GeometryNode]> = Vec::with_capacity(complete);
    for r in 0..complete {
        let start = r * ncols;
        row_slices.push(&geo.children[start..start + ncols]);
    }
    let paints = cell_paints(ops)?;
    let col_widths_twips = col_widths_from_row(row_slices[0], geo);
    let heights = row_heights_twips(&row_slices, geo);
    let mut rows_out = Vec::with_capacity(complete);
    for (r, cells_geo) in row_slices.iter().enumerate() {
        let header = r < meta.header_rows;
        let mut cells = Vec::with_capacity(ncols);
        for (c, g) in cells_geo.iter().enumerate() {
            let w = col_widths_twips.get(c).copied().unwrap_or(0);
            let node = super::cell::find_node(root, running, &g.id);
            cells.push(build_cell(g, w, paints.get(&g.id), node, header, fonts)?);
        }
        rows_out.push(TableRow {
            height_twips: heights.get(r).copied().unwrap_or(0),
            cells,
        });
    }
    Ok(Some(TableBox {
        node_id: table_id.to_string(),
        x_emu: pt_to_emu(geo.x),
        y_emu: pt_to_emu(geo.y),
        cx_emu: pt_to_emu(geo.width),
        cy_emu: pt_to_emu(geo.height),
        width_twips: pt_to_twips(geo.width),
        col_widths_twips,
        rows: rows_out,
        relative_height,
    }))
}

pub(crate) fn table_ref_placeholder(node_id: &str, rect: &Rect, relative_height: u32) -> ShapeBox {
    ShapeBox {
        node_id: node_id.to_string(),
        x_emu: pt_to_emu(rect.x),
        y_emu: pt_to_emu(rect.y),
        cx_emu: pt_to_emu(rect.width),
        cy_emu: pt_to_emu(rect.height),
        fill_hex: None,
        fill_alpha: 255,
        gradient: None,
        corner_emu: 0,
        line_hex: Some("808080".into()),
        line_w_emu: millipt_to_emu(1_000),
        line_dash: LineDash::Solid,
        behind_doc: false,
        relative_height,
    }
}

fn col_widths_from_row(row: &[GeometryNode], table: &GeometryNode) -> Vec<i64> {
    let n = row.len();
    (0..n)
        .map(|i| {
            let millipt = if i + 1 < n {
                row[i + 1].x.0 - row[i].x.0
            } else {
                (table.x.0 + table.width.0) - row[i].x.0
            };
            millipt_to_twips(i64::try_from(millipt.max(0)).unwrap_or(0))
        })
        .collect()
}

fn row_heights_twips(rows: &[&[GeometryNode]], table: &GeometryNode) -> Vec<i64> {
    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            let y = row.first().map(|c| c.y.0).unwrap_or(table.y.0);
            let next_y = rows
                .get(i + 1)
                .and_then(|r| r.first())
                .map(|c| c.y.0)
                .unwrap_or(table.y.0 + table.height.0);
            let from_gap = next_y - y;
            let from_cell = row.iter().map(|c| c.height.0).max().unwrap_or(0);
            millipt_to_twips(i64::try_from(from_gap.max(from_cell).max(0)).unwrap_or(0))
        })
        .collect()
}
