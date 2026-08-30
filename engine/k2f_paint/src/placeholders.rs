use k2f_core::Rect;
use tiny_skia::{Color, FillRule, Paint, Pixmap, Stroke, Transform};

use crate::geom::{rect_to_px, rounded_rect_path};

pub(crate) fn draw_image_placeholder(pixmap: &mut Pixmap, rect: &Rect, scale: f32) {
    let rect_px = rect_to_px(rect, scale);
    let x = rect_px.left as f32;
    let y = rect_px.top as f32;
    let w = (rect_px.right - rect_px.left).max(1) as f32;
    let h = (rect_px.bottom - rect_px.top).max(1) as f32;

    let Some(path) = rounded_rect_path(x, y, w, h, 0.0) else {
        return;
    };

    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(230, 233, 238, 255));
    pixmap.as_mut().fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );

    let mut stroke = Stroke::default();
    stroke.width = 1.0;
    let mut stroke_paint = Paint::default();
    stroke_paint.set_color(Color::from_rgba8(0, 0, 0, 32));
    pixmap
        .as_mut()
        .stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
}

pub(crate) fn draw_table_placeholder(pixmap: &mut Pixmap, rect: &Rect, scale: f32) {
    draw_image_placeholder(pixmap, rect, scale);
}
