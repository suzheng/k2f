use super::TableIndex;
use crate::align::{infer_text_align, line_spacing_spc_pts, vert_center};
use crate::coord::{millipt_to_emu, pt_to_emu};
use crate::ir::{
    BorderStroke, CellBorders, LineDash, ShapeBox, TableBox, TableCell, TableRow, TextAlign,
};
use crate::text::{cell_h_insets_emu, cell_runs, office_source_text, top_inset_emu, FontCtx};
use crate::PptxError;
use k2f_core::{
    find_in_trees, table_row_slots, table_rows_on_page, Border, BorderEdge, BorderStyle,
    GeometryNode, NodeContent, Page, PaintOp, Rect, RunningBlockNode, SemanticNode,
    TableDataSource, TextGlyphRun,
};
use k2f_paint::{parse_hex_rgba, resolve_fill};
use std::collections::{HashMap, HashSet};

pub(crate) fn table_on_page(
    index: &TableIndex,
    table_id: &str,
    page: &Page,
    ops: &[PaintOp],
    root: &SemanticNode,
    running: &[RunningBlockNode],
    fonts: &FontCtx,
) -> Result<Option<TableBox>, PptxError> {
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
    let col_widths_emu: Vec<i64> = visual_col_widths_millipt(ncols, &page_rows, geo)
        .into_iter()
        .map(|m| millipt_to_emu(i64::try_from(m.max(0)).unwrap_or(0)))
        .collect();
    let heights = row_heights_emu_from_geos(&page_rows, geo);
    let mut rows_out = Vec::with_capacity(page_rows.len());
    for (r, (sem_row, cells_geo)) in page_rows.iter().enumerate() {
        let header = r < meta.header_rows;
        let slots = table_row_slots(sem_row);
        let mut cells = Vec::with_capacity(sem_row.len());
        for (i, g) in cells_geo.iter().enumerate() {
            let slot = slots.get(i).copied().unwrap_or(k2f_core::TableCellSlot {
                start_col: i,
                span: 1,
            });
            let node = find_in_trees(root, running, &g.id);
            let paint = paints.get(&g.id);
            let no_text = paint.map(|p| p.runs.is_empty()).unwrap_or(true);
            let (runs, preserve) = cell_runs(
                node,
                paint.map(|p| p.runs.as_slice()).unwrap_or(&[]),
                Some(g),
                fonts,
                header && no_text,
            );
            let text = node.and_then(office_source_text).unwrap_or("");
            let align = if text.is_empty() {
                TextAlign::Left
            } else {
                infer_text_align(g, text)
            };
            let font_size = paint
                .and_then(|p| {
                    p.runs
                        .iter()
                        .map(|r| r.style.font_size)
                        .max_by_key(|pt| pt.0.abs())
                })
                .or_else(|| {
                    g.text_runs
                        .iter()
                        .map(|r| r.style.font_size)
                        .max_by_key(|pt| pt.0.abs())
                })
                .unwrap_or(k2f_core::Pt(12_000));
            let centered = vert_center(g, font_size);
            let mut line_spc_pts = line_spacing_spc_pts(Some(g));
            if line_spc_pts.is_none() && !runs.is_empty() {
                line_spc_pts = i32::try_from(font_size.0 / 10).ok().map(|v| v.max(100));
            }
            let (l_ins_emu, r_ins_emu) = cell_h_insets_emu(Some(g), align);
            cells.push(TableCell {
                node_id: g.id.clone(),
                runs,
                align,
                fill_hex: paint.and_then(|p| p.fill_hex.clone()),
                preserve_whitespace: preserve,
                borders: cell_borders(paint.and_then(|p| p.border.as_ref()))?,
                vert_center: centered,
                colspan: slot.span as u32,
                start_col: slot.start_col,
                t_ins_emu: if centered { 0 } else { top_inset_emu(Some(g)) },
                l_ins_emu,
                r_ins_emu,
                line_spc_pts,
            });
        }
        rows_out.push(TableRow {
            height_emu: heights.get(r).copied().unwrap_or(0),
            cells,
        });
    }
    Ok(Some(TableBox {
        node_id: table_id.to_string(),
        x_emu: pt_to_emu(geo.x),
        y_emu: pt_to_emu(geo.y),
        cx_emu: pt_to_emu(geo.width),
        cy_emu: pt_to_emu(geo.height),
        col_widths_emu,
        rows: rows_out,
    }))
}

pub(crate) fn table_ref_placeholder(node_id: &str, rect: &Rect) -> ShapeBox {
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
        line_hex: Some("D0D0D0".into()),
        line_alpha: 255,
        line_w_emu: millipt_to_emu(1_000),
        line_dash: LineDash::Solid,
    }
}

struct CellPaint {
    fill_hex: Option<String>,
    border: Option<Border>,
    runs: Vec<TextGlyphRun>,
}

impl CellPaint {
    fn empty() -> Self {
        Self {
            fill_hex: None,
            border: None,
            runs: Vec::new(),
        }
    }
}

fn cell_paints(ops: &[PaintOp]) -> Result<HashMap<String, CellPaint>, PptxError> {
    let mut map: HashMap<String, CellPaint> = HashMap::new();
    for op in ops {
        match op {
            PaintOp::DrawBox {
                node_id,
                decoration,
                ..
            } => {
                let fill = opaque_solid_hex(decoration)?;
                let e = map.entry(node_id.clone()).or_insert_with(CellPaint::empty);
                e.fill_hex = fill;
                e.border = decoration.border.clone();
            }
            PaintOp::DrawText { node_id, runs, .. } => {
                map.entry(node_id.clone())
                    .or_insert_with(CellPaint::empty)
                    .runs = runs.clone();
            }
            _ => {}
        }
    }
    Ok(map)
}

fn cell_borders(border: Option<&Border>) -> Result<CellBorders, PptxError> {
    let Some(b) = border else {
        return Ok(CellBorders::default());
    };
    if b.width_pt <= 0 || b.edges.is_empty() {
        return Ok(CellBorders::default());
    }
    let stroke = BorderStroke {
        color_hex: srgb_hex(&b.color)?,
        w_emu: millipt_to_emu(b.width_pt).max(1),
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

fn srgb_hex(color: &str) -> Result<String, PptxError> {
    let [r, g, b, _] = parse_hex_rgba(color)
        .ok_or_else(|| PptxError::Write(format!("unparseable color '{color}'")))?;
    Ok(crate::text::pin_office_srgb(&format!(
        "{r:02X}{g:02X}{b:02X}"
    )))
}

fn opaque_solid_hex(decoration: &k2f_core::BoxDecoration) -> Result<Option<String>, PptxError> {
    match resolve_fill(decoration) {
        Ok(Some(k2f_core::Fill::Solid { color })) => {
            let [r, g, b, a] = parse_hex_rgba(&color)
                .ok_or_else(|| PptxError::Write(format!("unparseable fill color '{color}'")))?;
            if a < 255 {
                return Ok(None);
            }
            Ok(Some(crate::text::pin_office_srgb(&format!(
                "{r:02X}{g:02X}{b:02X}"
            ))))
        }
        Ok(_) => Ok(None),
        Err(k2f_paint::PaintError::UnresolvedRef(name)) => {
            Err(PptxError::Write(format!("unresolved fill ref '{name}'")))
        }
        Err(e) => Err(e.into()),
    }
}

fn find_geo<'a>(node: &'a GeometryNode, id: &str) -> Option<&'a GeometryNode> {
    if node.id == id {
        return Some(node);
    }
    node.children.iter().find_map(|c| find_geo(c, id))
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

fn row_heights_emu_from_geos(
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
            millipt_to_emu(i64::try_from(from_gap.max(from_cell).max(0)).unwrap_or(0))
        })
        .collect()
}
