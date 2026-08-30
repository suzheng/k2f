use k2f_core::{LockFile, PaintOp};
use k2f_paint::{placed_glyphs, PaintError};
use std::collections::{BTreeMap, HashMap};
use ttf_parser::Face;

use crate::box_op::{draw_box, draw_placeholder};
use crate::draw::PageDraw;
use crate::error::PdfError;
use crate::image::{draw_image, ImageRes};
use crate::text::draw_glyph;

pub fn paint_page(
    lock: &LockFile,
    page_idx: usize,
    faces: &HashMap<String, Face<'_>>,
    images: &BTreeMap<String, ImageRes>,
) -> Result<PageDraw, PdfError> {
    if lock.has_unknown_paint_ops() {
        return Err(PdfError::UnknownOp);
    }
    let page = lock
        .geometry
        .pages
        .get(page_idx)
        .ok_or(PaintError::PageOutOfRange(page_idx))?;
    let plan = lock
        .render_plan
        .pages
        .iter()
        .find(|p| p.index == page.index)
        .ok_or(PaintError::MissingRenderPlan(page.index))?;
    let mut geo_by_id = HashMap::new();
    k2f_paint::index_geometry_multi_str(&page.root, &mut geo_by_id);
    let mut out = PageDraw::new(page.width.as_f64_pt(), page.height.as_f64_pt());
    for op in &plan.ops {
        match op {
            PaintOp::BackdropBlur { .. } => return Err(PdfError::RasterOp("backdrop_blur")),
            PaintOp::DrawBox {
                rect, decoration, ..
            } => draw_box(&mut out, rect, decoration)?,
            PaintOp::DrawText {
                node_id,
                rect,
                runs,
            } => {
                let geo = k2f_paint::geo_for_op_str(&geo_by_id, node_id, rect)
                    .ok_or_else(|| PaintError::MissingGeometry(node_id.clone()))?;
                for g in placed_glyphs(faces, geo, rect, runs)? {
                    draw_glyph(&mut out, faces, &g);
                }
            }
            PaintOp::DrawImage { rect, src, .. } => {
                draw_image(&mut out, rect, src, images)?;
            }
            PaintOp::DrawTableReference { rect, .. } => draw_placeholder(&mut out, rect),
            PaintOp::Unknown => return Err(PdfError::UnknownOp),
        }
    }
    Ok(out)
}
