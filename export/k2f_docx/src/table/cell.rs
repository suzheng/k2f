use crate::coord::pt_to_twips;
use crate::ir::{BorderStroke, CellBorders, TableCell, TextAlign};
use crate::text::{infer_text_align, line_spacing_twips, runs_from_paint, vert_center, FontCtx};
use crate::DocxError;
use k2f_core::{
    node_text, Border, BorderEdge, BorderStyle, BoxDecoration, Fill, GeometryNode, PaintOp, Rect,
    RunningBlockNode, SemanticNode, TextGlyphRun,
};
use k2f_paint::{parse_hex_rgba, resolve_fill};
use std::collections::HashMap;

pub(crate) struct CellPaint {
    fill_hex: Option<String>,
    border: Option<Border>,
    runs: Vec<TextGlyphRun>,
}

pub(crate) fn cell_paints(ops: &[PaintOp]) -> Result<HashMap<String, CellPaint>, DocxError> {
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

impl CellPaint {
    fn empty() -> Self {
        Self {
            fill_hex: None,
            border: None,
            runs: Vec::new(),
        }
    }
}

pub(crate) fn build_cell(
    geo: &GeometryNode,
    width_twips: i64,
    paint: Option<&CellPaint>,
    node: Option<&SemanticNode>,
    header: bool,
    fonts: &FontCtx,
) -> Result<TableCell, DocxError> {
    let text = node.and_then(node_text).unwrap_or("");
    let align = if text.is_empty() {
        TextAlign::Left
    } else {
        infer_text_align(geo, text)
    };
    let modifiers = node.map(|n| n.modifiers.as_slice()).unwrap_or(&[]);
    let paint_runs = paint.map(|p| p.runs.as_slice()).unwrap_or(&[]);
    let mut runs = if text.is_empty() {
        Vec::new()
    } else {
        runs_from_paint(text, paint_runs, modifiers, Some(geo), fonts)
    };
    if header && paint_runs.is_empty() {
        for r in &mut runs {
            r.bold = true;
        }
    }
    let preserve = node
        .map(|n| n.preserve_whitespace == Some(true) || n.role == "code_block")
        .unwrap_or(false);
    let font_size = paint
        .and_then(|p| p.runs.first())
        .map(|r| r.style.font_size)
        .or_else(|| geo.text_runs.first().map(|r| r.style.font_size))
        .unwrap_or(k2f_core::Pt(12_000));
    let cell_rect = Rect {
        x: geo.x,
        y: geo.y,
        width: geo.width,
        height: geo.height,
    };
    let centered = vert_center(Some(geo), &cell_rect, font_size);
    let mut line_twips = line_spacing_twips(Some(geo));
    if line_twips.is_none() && !runs.is_empty() {
        line_twips = Some(pt_to_twips(font_size).max(20));
    }
    Ok(TableCell {
        node_id: geo.id.clone(),
        width_twips,
        runs,
        align,
        fill_hex: paint.and_then(|p| p.fill_hex.clone()),
        preserve_whitespace: preserve,
        borders: cell_borders(paint.and_then(|p| p.border.as_ref()))?,
        vert_center: centered,
        line_twips,
    })
}

pub(crate) fn cell_borders(border: Option<&Border>) -> Result<CellBorders, DocxError> {
    let Some(b) = border else {
        return Ok(CellBorders::default());
    };
    if b.width_pt <= 0 || b.edges.is_empty() {
        return Ok(CellBorders::default());
    }
    let stroke = BorderStroke {
        color_hex: srgb_hex(&b.color)?,
        sz: border_sz(b.width_pt),
        val: match b.style {
            BorderStyle::Solid => "single",
            BorderStyle::Dashed => "dashed",
            BorderStyle::Dotted => "dotted",
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

fn border_sz(width_pt_millipt: i64) -> i64 {
    (width_pt_millipt / 125).clamp(2, 96)
}

fn opaque_solid_hex(decoration: &BoxDecoration) -> Result<Option<String>, DocxError> {
    match resolve_fill(decoration) {
        Ok(Some(Fill::Solid { color })) => {
            let [r, g, b, a] = parse_hex_rgba(&color)
                .ok_or_else(|| DocxError::Write(format!("unparseable fill color '{color}'")))?;
            if a < 255 {
                return Ok(None);
            }
            Ok(Some(format!("{r:02X}{g:02X}{b:02X}")))
        }
        Ok(_) => Ok(None),
        Err(k2f_paint::PaintError::UnresolvedRef(name)) => {
            Err(DocxError::Write(format!("unresolved fill ref '{name}'")))
        }
        Err(e) => Err(e.into()),
    }
}

fn srgb_hex(color: &str) -> Result<String, DocxError> {
    let [r, g, b, _] = parse_hex_rgba(color)
        .ok_or_else(|| DocxError::Write(format!("unparseable color '{color}'")))?;
    Ok(format!("{r:02X}{g:02X}{b:02X}"))
}

pub(crate) fn find_node<'a>(
    root: &'a SemanticNode,
    running: &'a [RunningBlockNode],
    id: &str,
) -> Option<&'a SemanticNode> {
    k2f_core::find_in_trees(root, running, id)
}
