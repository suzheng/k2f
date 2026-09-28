use k2f_paint::OpenedDocument;
use pdf_writer::Pdf;
use std::collections::{HashMap, HashSet};
use ttf_parser::Face;

use super::draw::draw_slice;
use super::raster::raster_slice;
use super::slice::Slice;
use crate::draw::PageDraw;
use crate::error::PdfError;
use crate::ids::Alloc;
use crate::image::{embed_rgba, ImageRes};
use crate::ops::paint_vector_op;

pub(crate) fn compose_page(
    doc: &OpenedDocument,
    page_idx: usize,
    faces: &HashMap<String, Face<'_>>,
    images: &std::collections::BTreeMap<String, ImageRes>,
    skip_text: &HashSet<String>,
    pdf: &mut Pdf,
    alloc: &mut Alloc,
    scale: f32,
    slices: &[Slice],
) -> Result<(PageDraw, Vec<ImageRes>), PdfError> {
    let lock = doc.lock().ok_or(PdfError::Unlocked)?;
    if lock.has_unknown_paint_ops() {
        return Err(PdfError::UnknownOp);
    }
    let page = lock
        .geometry
        .pages
        .get(page_idx)
        .ok_or(k2f_paint::PaintError::PageOutOfRange(page_idx))?;
    let plan = lock
        .render_plan
        .pages
        .iter()
        .find(|p| p.index == page.index)
        .ok_or(k2f_paint::PaintError::MissingRenderPlan(page.index))?;
    let mut geo_by_id = HashMap::new();
    k2f_paint::index_geometry_multi_str(&page.root, &mut geo_by_id);

    let slice_at: HashMap<usize, &Slice> = slices.iter().map(|s| (s.anchor, s)).collect();
    let mut skip = HashSet::new();
    let mut out = PageDraw::new(page.width.as_f64_pt(), page.height.as_f64_pt());
    let mut stamps = Vec::new();
    for (i, op) in plan.ops.iter().enumerate() {
        if skip.contains(&i) {
            continue;
        }
        if let Some(slice) = slice_at.get(&i) {
            for c in &slice.consumed {
                skip.insert(*c);
            }
            let (w, h, rgba) = raster_slice(doc, page_idx, slice, scale)?;
            let name = format!("Sl{page_idx}_{i}");
            let img = embed_rgba(pdf, alloc, w, h, &rgba, &name)?;
            draw_slice(&mut out, &img, &slice.crop);
            stamps.push(img);
            continue;
        }
        paint_vector_op(&mut out, faces, images, skip_text, &geo_by_id, op)?;
    }
    Ok((out, stamps))
}
