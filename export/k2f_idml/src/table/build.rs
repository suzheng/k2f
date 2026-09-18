use super::geom::col_widths_from_tracks;
use super::paint::{cell_paints, CellPaint};
use super::{cell_borders, harvestable_inline_rows, TableIndex};
use crate::align::infer_text_align_for;
use crate::coord::millipt_to_pt;
use crate::geo::find_geo;
use crate::ir::{TableBox, TableCell, TableRow, TextAlign};
use crate::text::{cell_runs, insets, TextFonts};
use crate::IdmlError;
use k2f_core::{
    find_in_trees, node_text, table_row_slots, table_rows_on_page, GeometryNode, NodeContent, Page,
    PaintOp, Pt, Rect, RunningBlockNode, SemanticNode, TableCellSlot,
};
use std::collections::{HashMap, HashSet};

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
    let built = if geo.children.is_empty() {
        from_spec_rows(table_id, geo, meta, &paints, root, running, fonts)?
    } else {
        from_page_rows(
            table_id,
            geo,
            ncols,
            meta.header_rows,
            &paints,
            root,
            running,
            fonts,
        )?
    };
    let Some((col_widths_pt, mut rows_out)) = built else {
        return Ok(None);
    };
    inherit_table_chrome(&mut rows_out, paints.get(table_id), col_widths_pt.len())?;
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
    ncols: usize,
) -> Result<(), IdmlError> {
    let Some(paint) = table_paint else {
        return Ok(());
    };
    let outer = cell_borders(paint.border.as_ref())?;
    let n_rows = rows.len();
    for (r, row) in rows.iter_mut().enumerate() {
        for cell in row.cells.iter_mut() {
            let span = cell.colspan.max(1) as usize;
            if cell.fill_hex.is_none() {
                cell.fill_hex = paint.fill_hex.clone();
            }
            if r == 0 && cell.borders.top.is_none() {
                cell.borders.top = outer.top.clone();
            }
            if cell.start_col == 0 && cell.borders.left.is_none() {
                cell.borders.left = outer.left.clone();
            }
            if r + 1 == n_rows && cell.borders.bottom.is_none() {
                cell.borders.bottom = outer.bottom.clone();
            }
            if cell.start_col + span == ncols && cell.borders.right.is_none() {
                cell.borders.right = outer.right.clone();
            }
        }
    }
    Ok(())
}

fn covering_index(cells: &[TableCell], col: usize) -> Option<usize> {
    cells.iter().position(|c| {
        let span = c.colspan.max(1) as usize;
        col >= c.start_col && col < c.start_col + span
    })
}

fn starting_index(cells: &[TableCell], col: usize) -> Option<usize> {
    cells.iter().position(|c| c.start_col == col)
}

/// Copy Bottom onto the next row's empty Top, and Right onto the next
/// column's empty Left, so interior rules survive InDesign's conflict rule.
fn promote_shared_edges(rows: &mut [TableRow]) {
    let n_rows = rows.len();
    for r in 0..n_rows {
        let n_cells = rows[r].cells.len();
        for c in 0..n_cells {
            let start = rows[r].cells[c].start_col;
            let span = rows[r].cells[c].colspan.max(1) as usize;
            if r + 1 < n_rows {
                let stroke = rows[r].cells[c].borders.bottom.clone();
                if stroke.is_some() {
                    if let Some(idx) = covering_index(&rows[r + 1].cells, start) {
                        let next = &mut rows[r + 1].cells[idx];
                        if next.borders.top.is_none() {
                            next.borders.top = stroke;
                        }
                    }
                }
            }
            let right = rows[r].cells[c].borders.right.clone();
            if right.is_some() {
                if let Some(idx) = starting_index(&rows[r].cells, start + span) {
                    let next = &mut rows[r].cells[idx];
                    if next.borders.left.is_none() {
                        next.borders.left = right;
                    }
                }
            }
        }
    }
}

fn page_row_geos<'a>(
    rows: &'a [Vec<SemanticNode>],
    geo: &'a GeometryNode,
) -> Vec<(&'a [SemanticNode], Vec<&'a GeometryNode>)> {
    let present: HashSet<&str> = geo.children.iter().map(|c| c.id.as_str()).collect();
    let by_id: HashMap<&str, &GeometryNode> =
        geo.children.iter().map(|c| (c.id.as_str(), c)).collect();
    table_rows_on_page(rows, &present)
        .into_iter()
        .filter_map(|row| {
            let geos: Vec<&GeometryNode> = row
                .iter()
                .map(|c| by_id.get(c.id.as_str()).copied())
                .collect::<Option<Vec<_>>>()?;
            Some((row, geos))
        })
        .collect()
}

fn col_widths_from_geos(row: &[&GeometryNode], table: &GeometryNode) -> Vec<f64> {
    let n = row.len();
    (0..n)
        .map(|i| {
            let millipt = if i + 1 < n {
                row[i + 1].x.0 - row[i].x.0
            } else {
                (table.x.0 + table.width.0) - row[i].x.0
            };
            millipt_to_pt(millipt.max(0))
        })
        .collect()
}

fn visual_col_widths_pt(
    ncols: usize,
    page_rows: &[(&[SemanticNode], Vec<&GeometryNode>)],
    table: &GeometryNode,
) -> Vec<f64> {
    for (sem, geos) in page_rows {
        let slots = table_row_slots(sem);
        if slots.len() == ncols && slots.iter().all(|s| s.span == 1) && geos.len() == ncols {
            return col_widths_from_geos(geos, table);
        }
    }
    let mut xs = vec![None; ncols + 1];
    xs[ncols] = Some(table.x.0 + table.width.0);
    for (sem, geos) in page_rows {
        let slots = table_row_slots(sem);
        for (i, slot) in slots.iter().enumerate() {
            let Some(g) = geos.get(i) else { continue };
            xs[slot.start_col] = Some(g.x.0);
            let end = slot.start_col + slot.span;
            if end < ncols {
                if let Some(next) = geos.get(i + 1) {
                    xs[end] = Some(next.x.0);
                }
            }
        }
    }
    if xs[0].is_none() {
        xs[0] = Some(table.x.0);
    }
    fill_missing_xs(&mut xs);
    (0..ncols)
        .map(|i| millipt_to_pt((xs[i + 1].unwrap_or(0) - xs[i].unwrap_or(0)).max(0)))
        .collect()
}

fn fill_missing_xs(xs: &mut [Option<i128>]) {
    let n = xs.len();
    let mut i = 0;
    while i < n {
        if xs[i].is_some() {
            i += 1;
            continue;
        }
        let left_i = i.saturating_sub(1);
        let left = xs[left_i];
        let mut j = i;
        while j < n && xs[j].is_none() {
            j += 1;
        }
        let right = if j < n { xs[j] } else { None };
        match (left, right) {
            (Some(l), Some(r)) => {
                let span = (j - left_i) as i128;
                if span <= 0 {
                    i = j;
                    continue;
                }
                for k in i..j {
                    xs[k] = Some(l + (r - l) * (k as i128 - left_i as i128) / span);
                }
            }
            (Some(l), None) => {
                for k in i..j {
                    xs[k] = Some(l);
                }
            }
            (None, Some(r)) => {
                for k in i..j {
                    xs[k] = Some(r);
                }
            }
            _ => {
                for k in i..j {
                    xs[k] = Some(0);
                }
            }
        }
        i = j;
    }
}

fn from_page_rows(
    table_id: &str,
    geo: &GeometryNode,
    ncols: usize,
    header_rows: usize,
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
    let page_rows = page_row_geos(rows, geo);
    if page_rows.is_empty() {
        return Ok(None);
    }
    let col_widths_pt = visual_col_widths_pt(ncols, &page_rows, geo);
    let heights: Vec<f64> = page_rows
        .iter()
        .enumerate()
        .map(|(i, (_, row))| {
            let y = row.first().map(|c| c.y.0).unwrap_or(geo.y.0);
            let next_y = page_rows
                .get(i + 1)
                .and_then(|(_, r)| r.first())
                .map(|c| c.y.0)
                .unwrap_or(geo.y.0 + geo.height.0);
            let from_gap = next_y - y;
            let from_cell = row.iter().map(|c| c.height.0).max().unwrap_or(0);
            millipt_to_pt(from_gap.max(from_cell).max(0))
        })
        .collect();
    let mut rows_out = Vec::with_capacity(page_rows.len());
    for (r, (sem_row, cells_geo)) in page_rows.iter().enumerate() {
        let header = r < header_rows;
        let slots = table_row_slots(sem_row);
        let mut cells = Vec::with_capacity(sem_row.len());
        for (i, g) in cells_geo.iter().enumerate() {
            let slot = slots.get(i).copied().unwrap_or(TableCellSlot {
                start_col: i,
                span: 1,
            });
            let node = find_in_trees(root, running, &g.id);
            cells.push(build_cell(g, paints.get(&g.id), node, header, fonts, slot)?);
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
    let col_widths_pt = col_widths_from_tracks(&meta.column_widths, millipt_to_pt(geo.width.0));
    let n_rows = rows.len();
    let row_h = millipt_to_pt(geo.height.0) / n_rows.max(1) as f64;
    let mut rows_out = Vec::with_capacity(n_rows);
    for (r, row) in rows.iter().enumerate() {
        let header = r < meta.header_rows;
        let slots = table_row_slots(row);
        let mut cells = Vec::with_capacity(row.len());
        for (cell_node, slot) in row.iter().zip(slots) {
            let cell_geo = find_geo(geo, &cell_node.id).unwrap_or(geo);
            cells.push(build_cell(
                cell_geo,
                paints.get(&cell_node.id),
                Some(cell_node),
                header,
                fonts,
                slot,
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
    slot: TableCellSlot,
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
        colspan: slot.span as u32,
        start_col: slot.start_col,
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

    fn cell(start_col: usize, bottom: bool, right: bool) -> TableCell {
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
            colspan: 1,
            start_col,
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
                cells: vec![cell(0, true, true), cell(1, true, false)],
            },
            TableRow {
                height_pt: 20.0,
                cells: vec![cell(0, true, false), cell(1, false, false)],
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

    #[test]
    fn spanned_cell_right_neighbors_visual_column() {
        let mut span2 = cell(0, true, true);
        span2.colspan = 2;
        let mut rows = vec![
            TableRow {
                height_pt: 20.0,
                cells: vec![span2, cell(2, true, false)],
            },
            TableRow {
                height_pt: 20.0,
                cells: vec![
                    cell(0, false, false),
                    cell(1, false, false),
                    cell(2, false, false),
                ],
            },
        ];
        promote_shared_edges(&mut rows);
        assert!(
            rows[0].cells[1].borders.left.is_some(),
            "span-2 right must land on the cell that starts at col 2"
        );
        assert!(
            rows[1].cells[2].borders.top.is_some(),
            "span-1 at col 2 bottom must copy to the cell covering col 2 below"
        );
        assert!(
            rows[1].cells[1].borders.top.is_none(),
            "must not copy onto authored index 1 when visual start is 2"
        );
    }
}
