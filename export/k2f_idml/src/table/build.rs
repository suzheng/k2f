use super::geom::{col_widths_from_row, col_widths_from_tracks, row_heights_pt};
use super::paint::{cell_paints, CellPaint};
use super::{cell_borders, harvestable_inline_rows, TableIndex};
use crate::align::infer_text_align;
use crate::coord::millipt_to_pt;
use crate::geo::find_geo;
use crate::ir::{TableBox, TableCell, TableRow, TextAlign};
use crate::text::{cell_runs, vert_center, TextFonts};
use crate::IdmlError;
use k2f_core::{
    find_in_trees, node_text, GeometryNode, NodeContent, Page, PaintOp, Rect, RunningBlockNode,
    SemanticNode,
};
use std::collections::HashMap;

pub(crate) fn table_on_page(
    index: &TableIndex,
    table_id: &str,
    page: &Page,
    ops: &[PaintOp],
    root: &SemanticNode,
    running: &[RunningBlockNode],
    fonts: &TextFonts,
) -> Result<Option<TableBox>, IdmlError> {
    let Some(meta) = index.tables.get(table_id) else {
        return Ok(None);
    };
    let Some(geo) = find_geo(&page.root, table_id) else {
        return Ok(None);
    };
    let ncols = meta.column_widths.len();
    if ncols == 0 {
        return Ok(None);
    }
    let paints = cell_paints(ops)?;
    let built = if geo.children.len() >= ncols {
        from_geometry(geo, ncols, meta.header_rows, &paints, root, running, fonts)?
    } else if geo.children.is_empty() {
        from_spec_rows(table_id, geo, meta, &paints, root, running, fonts)?
    } else {
        return Ok(None);
    };
    let Some((col_widths_pt, rows_out)) = built else {
        return Ok(None);
    };
    Ok(Some(TableBox {
        node_id: table_id.to_string(),
        rect: Rect {
            x: geo.x,
            y: geo.y,
            width: geo.width,
            height: geo.height,
        },
        header_rows: meta.header_rows.min(rows_out.len()),
        col_widths_pt,
        rows: rows_out,
    }))
}

fn from_geometry(
    geo: &GeometryNode,
    ncols: usize,
    header_rows: usize,
    paints: &HashMap<String, CellPaint>,
    root: &SemanticNode,
    running: &[RunningBlockNode],
    fonts: &TextFonts,
) -> Result<Option<(Vec<f64>, Vec<TableRow>)>, IdmlError> {
    let complete = geo.children.len() / ncols;
    if complete == 0 {
        return Ok(None);
    }
    let mut row_slices: Vec<&[GeometryNode]> = Vec::with_capacity(complete);
    for r in 0..complete {
        let start = r * ncols;
        row_slices.push(&geo.children[start..start + ncols]);
    }
    let col_widths_pt = col_widths_from_row(row_slices[0], geo);
    let heights = row_heights_pt(&row_slices, geo);
    let mut rows_out = Vec::with_capacity(complete);
    for (r, cells_geo) in row_slices.iter().enumerate() {
        let header = r < header_rows;
        let mut cells = Vec::with_capacity(ncols);
        for g in *cells_geo {
            let node = find_in_trees(root, running, &g.id);
            cells.push(build_cell(g, paints.get(&g.id), node, header, fonts)?);
        }
        rows_out.push(TableRow {
            height_pt: heights.get(r).copied().unwrap_or(0.0),
            cells,
        });
    }
    Ok(Some((col_widths_pt, rows_out)))
}

fn from_spec_rows(
    table_id: &str,
    geo: &GeometryNode,
    meta: &super::Harvested,
    paints: &HashMap<String, CellPaint>,
    root: &SemanticNode,
    running: &[RunningBlockNode],
    fonts: &TextFonts,
) -> Result<Option<(Vec<f64>, Vec<TableRow>)>, IdmlError> {
    let Some(node) = find_in_trees(root, running, table_id) else {
        return Ok(None);
    };
    let NodeContent::Table(spec) = &node.content else {
        return Ok(None);
    };
    let Some(rows) = harvestable_inline_rows(spec) else {
        return Ok(None);
    };
    if rows.is_empty() {
        return Ok(None);
    }
    let ncols = meta.column_widths.len();
    let col_widths_pt = col_widths_from_tracks(&meta.column_widths, millipt_to_pt(geo.width.0));
    let n_rows = rows.len();
    let row_h = millipt_to_pt(geo.height.0) / n_rows.max(1) as f64;
    let mut rows_out = Vec::with_capacity(n_rows);
    for (r, row) in rows.iter().enumerate() {
        let header = r < meta.header_rows;
        let mut cells = Vec::with_capacity(ncols);
        for cell_node in row.iter().take(ncols) {
            let cell_geo = find_geo(geo, &cell_node.id).unwrap_or(geo);
            cells.push(build_cell(
                cell_geo,
                paints.get(&cell_node.id),
                Some(cell_node),
                header,
                fonts,
            )?);
        }
        rows_out.push(TableRow {
            height_pt: row_h,
            cells,
        });
    }
    Ok(Some((col_widths_pt, rows_out)))
}

fn build_cell(
    geo: &GeometryNode,
    paint: Option<&CellPaint>,
    node: Option<&SemanticNode>,
    header: bool,
    fonts: &TextFonts,
) -> Result<TableCell, IdmlError> {
    let text = node.and_then(node_text).unwrap_or("");
    let align = if text.is_empty() {
        TextAlign::Left
    } else {
        infer_text_align(geo, text)
    };
    let paint_runs = paint.map(|p| p.runs.as_slice()).unwrap_or(&[]);
    let runs = cell_runs(
        node,
        paint_runs,
        Some(geo),
        fonts,
        header && paint_runs.is_empty(),
    );
    let font_size = paint
        .and_then(|p| {
            p.runs
                .iter()
                .map(|r| r.style.font_size)
                .max_by_key(|pt| pt.0.abs())
        })
        .or_else(|| {
            geo.text_runs
                .iter()
                .map(|r| r.style.font_size)
                .max_by_key(|pt| pt.0.abs())
        })
        .unwrap_or(k2f_core::Pt(12_000));
    let cell_rect = Rect {
        x: geo.x,
        y: geo.y,
        width: geo.width,
        height: geo.height,
    };
    Ok(TableCell {
        node_id: node.map(|n| n.id.clone()).unwrap_or_else(|| geo.id.clone()),
        runs,
        align,
        fill_hex: paint.and_then(|p| p.fill_hex.clone()),
        borders: cell_borders(paint.and_then(|p| p.border.as_ref()))?,
        vert_center: vert_center(Some(geo), &cell_rect, font_size),
    })
}
