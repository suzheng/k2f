use super::TableIndex;
use crate::align::{infer_text_align, vert_center};
use crate::coord::{millipt_to_emu, pt_to_emu};
use crate::ir::{
    BorderStroke, CellBorders, LineDash, ShapeBox, TableBox, TableCell, TableRow, TextAlign,
};
use crate::text::{cell_runs, FontCtx};
use crate::PptxError;
use k2f_core::{
    find_in_trees, node_text, Border, BorderEdge, BorderStyle, GeometryNode, Page, PaintOp, Rect,
    RunningBlockNode, SemanticNode, TextGlyphRun,
};
use k2f_paint::{parse_hex_rgba, resolve_fill};
use std::collections::HashMap;

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
    let col_widths_emu = col_widths_from_row(row_slices[0], geo);
    let heights = row_heights_emu(&row_slices, geo);
    let mut rows_out = Vec::with_capacity(complete);
    for (r, cells_geo) in row_slices.iter().enumerate() {
        let header = r < meta.header_rows;
        let mut cells = Vec::with_capacity(ncols);
        for g in *cells_geo {
            let node = find_in_trees(root, running, &g.id);
            let paint = paints.get(&g.id);
            let no_text = paint.map(|p| p.runs.is_empty()).unwrap_or(true);
            let (runs, preserve) = cell_runs(
                node,
                paint.map(|p| p.runs.as_slice()).unwrap_or(&[]),
                fonts,
                header && no_text,
            );
            let text = node.and_then(node_text).unwrap_or("");
            let align = if text.is_empty() {
                TextAlign::Left
            } else {
                infer_text_align(g, text)
            };
            let font_size = paint
                .and_then(|p| p.runs.first())
                .map(|r| r.style.font_size)
                .or_else(|| g.text_runs.first().map(|r| r.style.font_size))
                .unwrap_or(k2f_core::Pt(12_000));
            cells.push(TableCell {
                node_id: g.id.clone(),
                runs,
                align,
                fill_hex: paint.and_then(|p| p.fill_hex.clone()),
                preserve_whitespace: preserve,
                borders: cell_borders(paint.and_then(|p| p.border.as_ref()))?,
                vert_center: vert_center(g, font_size),
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
        fill_alpha_ppt: None,
        corner_emu: 0,
        line_hex: Some("D0D0D0".into()),
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
    Ok(format!("{r:02X}{g:02X}{b:02X}"))
}

fn opaque_solid_hex(decoration: &k2f_core::BoxDecoration) -> Result<Option<String>, PptxError> {
    match resolve_fill(decoration) {
        Ok(Some(k2f_core::Fill::Solid { color })) => {
            let [r, g, b, a] = parse_hex_rgba(&color)
                .ok_or_else(|| PptxError::Write(format!("unparseable fill color '{color}'")))?;
            if a < 255 {
                return Ok(None);
            }
            Ok(Some(format!("{r:02X}{g:02X}{b:02X}")))
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

fn col_widths_from_row(row: &[GeometryNode], table: &GeometryNode) -> Vec<i64> {
    let n = row.len();
    (0..n)
        .map(|i| {
            let millipt = if i + 1 < n {
                row[i + 1].x.0 - row[i].x.0
            } else {
                (table.x.0 + table.width.0) - row[i].x.0
            };
            millipt_to_emu(i64::try_from(millipt.max(0)).unwrap_or(0))
        })
        .collect()
}

fn row_heights_emu(rows: &[&[GeometryNode]], table: &GeometryNode) -> Vec<i64> {
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
            millipt_to_emu(i64::try_from(from_gap.max(from_cell).max(0)).unwrap_or(0))
        })
        .collect()
}
