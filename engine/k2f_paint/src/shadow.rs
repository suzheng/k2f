use k2f_core::effects::box_blur::box_blur_premul_rgba;
use k2f_core::{Rect, Shadow, ShadowLayer};
use tiny_skia::{Color, FillRule, Paint, Pixmap, Transform};

use crate::blit::blit_premul;
use crate::error::PaintError;
use crate::geom::{parse_hex_color_rgba8, pt_i64_to_px_i32, rect_to_px, rounded_rect_path};

pub(crate) fn draw_shadow_layers(
    dst: &mut Pixmap,
    rect: &Rect,
    corner_radius_pt: Option<i64>,
    shadow: &Shadow,
    scale: f32,
) -> Result<(), PaintError> {
    for layer in &shadow.layers {
        draw_shadow_layer(dst, rect, corner_radius_pt, layer, scale)?;
    }
    Ok(())
}

fn draw_shadow_layer(
    dst: &mut Pixmap,
    rect: &Rect,
    corner_radius_pt: Option<i64>,
    layer: &ShadowLayer,
    scale: f32,
) -> Result<(), PaintError> {
    let blur_radius_px = pt_i64_to_px_i32(layer.blur_radius_pt, scale).max(0);
    let spread_px = pt_i64_to_px_i32(layer.spread_radius_pt, scale);
    let offset_x_px = pt_i64_to_px_i32(layer.offset_x_pt, scale);
    let offset_y_px = pt_i64_to_px_i32(layer.offset_y_pt, scale);
    let passes = 3i32;
    let blur_margin_px = blur_radius_px.saturating_mul(passes);

    let rect_px = rect_to_px(rect, scale);
    let shape_left = rect_px.left + offset_x_px - spread_px;
    let shape_top = rect_px.top + offset_y_px - spread_px;
    let shape_right = rect_px.right + offset_x_px + spread_px;
    let shape_bottom = rect_px.bottom + offset_y_px + spread_px;

    let color = parse_hex_color_rgba8(&layer.color).unwrap_or(Color::from_rgba8(0, 0, 0, 0));
    if color.alpha() == 0.0 {
        return Ok(());
    }

    let img_left = shape_left - blur_margin_px;
    let img_top = shape_top - blur_margin_px;
    let img_right = shape_right + blur_margin_px;
    let img_bottom = shape_bottom + blur_margin_px;
    let img_w = (img_right - img_left).max(1) as u32;
    let img_h = (img_bottom - img_top).max(1) as u32;
    let mut tmp = Pixmap::new(img_w, img_h).ok_or(PaintError::Pixmap)?;
    tmp.fill(Color::from_rgba8(0, 0, 0, 0));

    let local_left = (shape_left - img_left) as f32;
    let local_top = (shape_top - img_top) as f32;
    let local_w = (shape_right - shape_left).max(1) as f32;
    let local_h = (shape_bottom - shape_top).max(1) as f32;
    let radius_px = corner_radius_pt
        .map(|r| pt_i64_to_px_i32(r, scale).max(0) as f32)
        .unwrap_or(0.0);

    if let Some(path) = rounded_rect_path(local_left, local_top, local_w, local_h, radius_px) {
        let mut paint = Paint::default();
        paint.set_color(color);
        tmp.as_mut().fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }

    let radius_usize = blur_radius_px as usize;
    if radius_usize > 0 {
        box_blur_premul_rgba(
            tmp.data_mut(),
            img_w as usize,
            img_h as usize,
            radius_usize,
            passes as usize,
        );
    }

    blit_premul(dst, img_left, img_top, &tmp);
    Ok(())
}
