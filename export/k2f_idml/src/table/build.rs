use super::geom::{col_widths_from_row, col_widths_from_tracks, row_heights_pt};
use super::paint::{cell_paints, CellPaint};
use super::{cell_borders, harvestable_inline_rows, TableIndex};
use crate::align::infer_text_align_for;
use crate::coord::millipt_to_pt;
use crate::geo::find_geo;
use crate::ir::{TableBox, TableCell, TableRow, TextAlign};
use crate::text::{cell_runs, insets, TextFonts};
use crate::IdmlError;
use k2f_core::{
    find_in_trees, node_text, GeometryNode, NodeContent, Page, PaintOp, Pt, Rect,
    RunningBlockNode, SemanticNode,
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
    let Some((col_widths_pt, mut rows_out)) = built else {
        return Ok(None);
    };
    inherit_table_chrome(&mut rows_out, paints.get(table_id))?;
    // InDesign paints a shared grid line from the lower/right cell when
    // priorities are equal. A weight-0 Top/Left would hide Bottom/Right.
    promote_shared_edges(&mut rows_out);
    let row_sum_millipt = rows_out
        .iter()
        .map(|r| (r.height_pt * 1000.0).ceil() as i128)
        .sum();
    let height = Pt(geo.height.0.max(row_sum_millipt));
    Ok(Some(TableBox {
        node_id: table_id.to_string(),
        rect: Rect {
            x: geo.x,
            y: geo.y,
            width: geo.width,
            height,
        },
        header_rows: meta.header_rows.min(rows_out.len()),
        col_widths_pt,
        rows: rows_out,
    }))
}

/// Table-node DrawBox (fill + outer stroke) is skipped once the native Table is
/// emitted. Copy that chrome onto cells so the grid is not just inner edges.
fn inherit_table_chrome(
    rows: &mut [TableRow],
    table_paint: Option<&CellPaint>,
) -> Result<(), IdmlError> {
    let Some(paint) = table_paint else {
        return Ok(());
    };
    let outer = cell_borders(paint.border.as_ref())?;
    let n_rows = rows.len();
    for (r, row) in rows.iter_mut().enumerate() {
        let n_cols = row.cells.len();
        for (c, cell) in row.cells.iter_mut().enumerate() {
            if cell.fill_hex.is_none() {
                cell.fill_hex = paint.fill_hex.clone();
            }
            if r == 0 && cell.borders.top.is_none() {
                cell.borders.top = outer.top.clone();
            }
            if c == 0 && cell.borders.left.is_none() {
                cell.borders.left = outer.left.clone();
            }
            if r + 1 == n_rows && cell.borders.bottom.is_none() {
                cell.borders.bottom = outer.bottom.clone();
            }
            if c + 1 == n_cols && cell.borders.right.is_none() {
                cell.borders.right = outer.right.clone();
            }
        }
    }
    Ok(())
}

/// Copy Bottom onto the next row's empty Top, and Right onto the next
/// column's empty Left, so interior rules survive InDesign's conflict rule.
fn promote_shared_edges(rows: &mut [TableRow]) {
    let n_rows = rows.len();
    for r in 0..n_rows {
        let n_cols = rows[r].cells.len();
        for c in 0..n_cols {
            if r + 1 < n_rows {
                let stroke = rows[r].cells[c].borders.bottom.clone();
                if stroke.is_some() {
                    if let Some(next) = rows[r + 1].cells.get_mut(c) {
                        if next.borders.top.is_none() {
                            next.borders.top = stroke;
                        }
                    }
                }
            }
            if c + 1 < n_cols {
                let stroke = rows[r].cells[c].borders.right.clone();
                if stroke.is_some() {
                    if let Some(next) = rows[r].cells.get_mut(c + 1) {
                        if next.borders.left.is_none() {
                            next.borders.left = stroke;
                        }
                    }
                }
            }
        }
    }
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
        infer_text_align_for(
            geo,
            text,
            node.map(|n| n.modifiers.as_slice()).unwrap_or(&[]),
        )
    };
    let paint_runs = paint.map(|p| p.runs.as_slice()).unwrap_or(&[]);
    let runs = cell_runs(
        node,
        paint_runs,
        Some(geo),
        fonts,
        header && paint_runs.is_empty(),
    );
    let (inset_top, inset_left, inset_bottom, inset_right) = insets(Some(geo), align);
    // Fixed-height rows: CenterAlign lets host metrics bleed into the next row.
    Ok(TableCell {
        node_id: node.map(|n| n.id.clone()).unwrap_or_else(|| geo.id.clone()),
        runs,
        align,
        fill_hex: paint.and_then(|p| p.fill_hex.clone()),
        borders: cell_borders(paint.and_then(|p| p.border.as_ref()))?,
        vert_center: false,
        inset_top,
        inset_left,
        inset_bottom,
        inset_right,
    })
}

#[cfg(test)]
mod tests {
    use super::promote_shared_edges;
    use crate::ir::{BorderStroke, CellBorders, LineDash, TableCell, TableRow, TextAlign};

    fn hairline() -> BorderStroke {
        BorderStroke {
            color_hex: "E2E8F0".into(),
            weight_pt: 0.75,
            dash: LineDash::Solid,
        }
    }

    fn cell(bottom: bool, right: bool) -> TableCell {
        TableCell {
            node_id: "c".into(),
            runs: Vec::new(),
            align: TextAlign::Left,
            fill_hex: None,
            borders: CellBorders {
                top: None,
                left: None,
                bottom: bottom.then(hairline),
                right: right.then(hairline),
            },
            vert_center: false,
            inset_top: 0.0,
            inset_left: 0.0,
            inset_bottom: 0.0,
            inset_right: 0.0,
        }
    }

    #[test]
    fn bottom_hairline_is_copied_to_next_row_top() {
        let mut rows = vec![
            TableRow {
                height_pt: 20.0,
                cells: vec![cell(true, true), cell(true, false)],
            },
            TableRow {
                height_pt: 20.0,
                cells: vec![cell(true, false), cell(false, false)],
            },
        ];
        promote_shared_edges(&mut rows);
        assert!(
            rows[1].cells[0].borders.top.is_some(),
            "next row must inherit the shared bottom rule"
        );
        assert!(
            rows[0].cells[1].borders.left.is_some(),
            "right hairline must appear as the next cell's left"
        );
        assert!(
            rows[0].cells[0].borders.left.is_none(),
            "must not invent a left on a cell that had no right neighbor stroke"
        );
    }
}
