use super::cell::{build_cell, cell_paints};
use super::TableIndex;
use crate::coord::{millipt_to_emu, millipt_to_twips, pt_to_emu, pt_to_twips};
use crate::geo::find_geo;
use crate::ir::{LineDash, ShapeBox, TableBox, TableRow};
use crate::text::FontCtx;
use crate::DocxError;
use k2f_core::{
    find_in_trees, table_row_slots, table_rows_on_page, GeometryNode, NodeContent, Page, PaintOp,
    Rect, RunningBlockNode, SemanticNode, TableCellSlot, TableDataSource,
};
use std::collections::{HashMap, HashSet};

#[allow(dead_code)]
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
    let Some(table_node) = find_in_trees(root, running, table_id) else {
        return Ok(None);
    };
    let NodeContent::Table(spec) = &table_node.content else {
        return Ok(None);
    };
    let TableDataSource::Inline { rows } = &spec.data else {
        return Ok(None);
    };
    let page_rows = page_row_geos(rows, geo);
    if page_rows.is_empty() {
        return Ok(None);
    }
    let paints = cell_paints(ops)?;
    let col_widths_twips: Vec<i64> = visual_col_widths_millipt(ncols, &page_rows, geo)
        .into_iter()
        .map(|m| millipt_to_twips(i64::try_from(m.max(0)).unwrap_or(0)))
        .collect();
    let heights = row_heights_twips_from_geos(&page_rows, geo);
    let mut rows_out = Vec::with_capacity(page_rows.len());
    for (r, (sem_row, cells_geo)) in page_rows.iter().enumerate() {
        let header = r < meta.header_rows;
        let slots = table_row_slots(sem_row);
        let mut cells = Vec::with_capacity(sem_row.len());
        for (i, g) in cells_geo.iter().enumerate() {
            let slot = slots.get(i).copied().unwrap_or(TableCellSlot {
                start_col: i,
                span: 1,
            });
            let w: i64 = col_widths_twips
                .get(slot.start_col..slot.start_col + slot.span)
                .map(|s| s.iter().sum())
                .unwrap_or(0);
            let node = super::cell::find_node(root, running, &g.id);
            let mut cell = build_cell(g, w, paints.get(&g.id), node, header, fonts)?;
            cell.colspan = slot.span as u32;
            cell.start_col = slot.start_col;
            cells.push(cell);
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
        fill_hex: None,
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
        line_alpha: 255,
        line_w_emu: millipt_to_emu(1_000),
        line_dash: LineDash::Solid,
        behind_doc: false,
        relative_height,
        pin_empty_txbox: true,
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

fn col_widths_from_geos(row: &[&GeometryNode], table: &GeometryNode) -> Vec<i128> {
    let n = row.len();
    (0..n)
        .map(|i| {
            if i + 1 < n {
                row[i + 1].x.0 - row[i].x.0
            } else {
                (table.x.0 + table.width.0) - row[i].x.0
            }
            .max(0)
        })
        .collect()
}

fn visual_col_widths_millipt(
    ncols: usize,
    page_rows: &[(&[SemanticNode], Vec<&GeometryNode>)],
    table: &GeometryNode,
) -> Vec<i128> {
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
        .map(|i| (xs[i + 1].unwrap_or(0) - xs[i].unwrap_or(0)).max(0))
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

fn row_heights_twips_from_geos(
    rows: &[(&[SemanticNode], Vec<&GeometryNode>)],
    table: &GeometryNode,
) -> Vec<i64> {
    rows.iter()
        .enumerate()
        .map(|(i, (_, row))| {
            let y = row.first().map(|c| c.y.0).unwrap_or(table.y.0);
            let next_y = rows
                .get(i + 1)
                .and_then(|(_, r)| r.first())
                .map(|c| c.y.0)
                .unwrap_or(table.y.0 + table.height.0);
            let from_gap = next_y - y;
            let from_cell = row.iter().map(|c| c.height.0).max().unwrap_or(0);
            millipt_to_twips(i64::try_from(from_gap.max(from_cell).max(0)).unwrap_or(0))
        })
        .collect()
}
