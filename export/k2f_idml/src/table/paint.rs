use crate::IdmlError;
use k2f_core::{BoxDecoration, Fill, PaintOp, TextGlyphRun};
use k2f_paint::{parse_hex_rgba, resolve_fill};
use std::collections::HashMap;

pub(super) struct CellPaint {
    pub fill_hex: Option<String>,
    pub border: Option<k2f_core::Border>,
    pub runs: Vec<TextGlyphRun>,
    pub corner_radius_pt: i64,
}

impl CellPaint {
    fn empty() -> Self {
        Self {
            fill_hex: None,
            border: None,
            runs: Vec::new(),
            corner_radius_pt: 0,
        }
    }
}

pub(super) fn cell_paints(ops: &[PaintOp]) -> Result<HashMap<String, CellPaint>, IdmlError> {
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
                e.corner_radius_pt = decoration.corner_radius_pt.unwrap_or(0).max(0);
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

fn opaque_solid_hex(decoration: &BoxDecoration) -> Result<Option<String>, IdmlError> {
    match resolve_fill(decoration) {
        Ok(Some(Fill::Solid { color })) => {
            let [r, g, b, a] = parse_hex_rgba(&color)
                .ok_or_else(|| IdmlError::Write(format!("unparseable fill color '{color}'")))?;
            if a < 255 {
                return Ok(None);
            }
            Ok(Some(format!("{r:02X}{g:02X}{b:02X}")))
        }
        Ok(_) => Ok(None),
        Err(k2f_paint::PaintError::UnresolvedRef(name)) => {
            Err(IdmlError::Write(format!("unresolved fill ref '{name}'")))
        }
        Err(e) => Err(e.into()),
    }
}
