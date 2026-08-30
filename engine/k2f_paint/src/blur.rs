use k2f_core::Rect;
use tiny_skia::Pixmap;

use crate::geom::{pt_i64_to_px_i32, rect_to_px};

pub(crate) fn apply_backdrop_blur(
    pixmap: &mut Pixmap,
    rect: &Rect,
    radius_pt: i64,
    corner_radius_pt: Option<i64>,
    scale: f32,
) {
    let rect_px = rect_to_px(rect, scale);
    if rect_px.left >= rect_px.right || rect_px.top >= rect_px.bottom {
        return;
    }

    let w = pixmap.width() as usize;
    let h = pixmap.height() as usize;
    let rx = rect_px.left.max(0) as usize;
    let ry = rect_px.top.max(0) as usize;
    let rw = (rect_px.right - rect_px.left).max(1) as usize;
    let rh = (rect_px.bottom - rect_px.top).max(1) as usize;
    let radius_px = pt_i64_to_px_i32(radius_pt, scale).max(0) as usize;
    let corner_radius_px = corner_radius_pt
        .map(|r| pt_i64_to_px_i32(r, scale).max(0) as usize)
        .filter(|r| *r > 0);

    k2f_core::effects::backdrop_blur::apply_backdrop_blur_premul_rgba(
        pixmap.data_mut(),
        w,
        h,
        rx,
        ry,
        rw,
        rh,
        radius_px,
        3,
        corner_radius_px,
    );
}
