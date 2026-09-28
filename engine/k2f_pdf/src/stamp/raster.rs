use k2f_core::{PaintOp, Rect};
use k2f_paint::{render_lockfile_page_rgba, OpenedDocument};

use super::slice::Slice;
use crate::error::PdfError;

pub fn raster_slice(
    doc: &OpenedDocument,
    page_idx: usize,
    slice: &Slice,
    scale: f32,
) -> Result<(u32, u32, Vec<u8>), PdfError> {
    let (w, h, color) = render_ops(doc, page_idx, &slice.color_ops, scale)?;
    let rgba = if slice.plate_ops.is_empty() {
        color
    } else {
        let (_pw, _ph, plate) = render_ops(doc, page_idx, &slice.plate_ops, scale)?;
        if plate.len() != color.len() {
            return Err(PdfError::Write("slice passes differ in size".into()));
        }
        let cover = slice.cover.as_ref().ok_or_else(|| {
            PdfError::Write("blur slice missing cover rect".into())
        })?;
        composite_blur(&color, &plate, w, h, cover, scale)
    };
    crop_rgba(&rgba, w, h, &slice.crop, scale)
}

fn render_ops(
    doc: &OpenedDocument,
    page_idx: usize,
    ops: &[PaintOp],
    scale: f32,
) -> Result<(u32, u32, Vec<u8>), PdfError> {
    let mut lock = doc.lock().ok_or(PdfError::Unlocked)?.clone();
    let page_index = lock
        .geometry
        .pages
        .get(page_idx)
        .ok_or(k2f_paint::PaintError::PageOutOfRange(page_idx))?
        .index;
    let plan = lock
        .render_plan
        .pages
        .iter_mut()
        .find(|p| p.index == page_index)
        .ok_or(k2f_paint::PaintError::MissingRenderPlan(page_index))?;
    plan.ops = ops.to_vec();
    Ok(render_lockfile_page_rgba(
        &lock,
        page_idx,
        scale,
        doc.fonts(),
        doc.assets(),
    )?)
}

fn composite_blur(
    color: &[u8],
    plate: &[u8],
    img_w: u32,
    img_h: u32,
    cover: &Rect,
    scale: f32,
) -> Vec<u8> {
    let (left, top, right, bottom) = px_bounds(cover, scale, img_w, img_h);
    let mut out = plate.to_vec();
    for y in top..bottom {
        for x in left..right {
            let i = ((y as u32) * img_w + x as u32) as usize * 4;
            if plate[i + 3] == 0 {
                out[i] = 0;
                out[i + 1] = 0;
                out[i + 2] = 0;
                out[i + 3] = 0;
            } else {
                out[i] = color[i];
                out[i + 1] = color[i + 1];
                out[i + 2] = color[i + 2];
                out[i + 3] = 255;
            }
        }
    }
    out
}

fn crop_rgba(
    rgba: &[u8],
    img_w: u32,
    img_h: u32,
    rect: &Rect,
    scale: f32,
) -> Result<(u32, u32, Vec<u8>), PdfError> {
    let (left, top, right, bottom) = px_bounds(rect, scale, img_w, img_h);
    if right <= left || bottom <= top {
        return Err(PdfError::Write("empty slice crop".into()));
    }
    let cw = (right - left) as u32;
    let ch = (bottom - top) as u32;
    let mut cropped = Vec::with_capacity((cw * ch * 4) as usize);
    for y in top..bottom {
        let row = ((y as u32) * img_w + left as u32) as usize * 4;
        let n = (cw as usize) * 4;
        cropped.extend_from_slice(&rgba[row..row + n]);
    }
    Ok((cw, ch, cropped))
}

fn px_bounds(rect: &Rect, scale: f32, img_w: u32, img_h: u32) -> (i32, i32, i32, i32) {
    let mut left = millipt_to_px(rect.x.0, scale);
    let mut top = millipt_to_px(rect.y.0, scale);
    let mut right = millipt_to_px(rect.x.0 + rect.width.0, scale);
    let mut bottom = millipt_to_px(rect.y.0 + rect.height.0, scale);
    let w = img_w as i32;
    let h = img_h as i32;
    left = left.clamp(0, w.saturating_sub(1).max(0));
    top = top.clamp(0, h.saturating_sub(1).max(0));
    right = right.clamp(left + 1, w.max(left + 1));
    bottom = bottom.clamp(top + 1, h.max(top + 1));
    (left, top, right, bottom)
}

fn millipt_to_px(millipt: i128, scale: f32) -> i32 {
    let pt = millipt as f64 / 1000.0;
    (pt * f64::from(scale)).round() as i32
}
