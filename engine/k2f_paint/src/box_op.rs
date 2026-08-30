use k2f_core::{Border, BorderEdge, BorderStyle, BoxDecoration, Rect};
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, StrokeDash, Transform};

use crate::error::PaintError;
use crate::fill::{fill_path_with_fill, resolve_fill, resolve_shadow};
use crate::geom::{parse_hex_color_rgba8, pt_i64_to_px_i32, rect_to_px, rounded_rect_path};
use crate::shadow::draw_shadow_layers;

pub(crate) fn draw_box(
    pixmap: &mut Pixmap,
    rect: &Rect,
    decoration: &BoxDecoration,
    scale: f32,
) -> Result<(), PaintError> {
    if let Some(shadow) = resolve_shadow(decoration)? {
        draw_shadow_layers(pixmap, rect, decoration.corner_radius_pt, &shadow, scale)?;
    }

    let rect_px = rect_to_px(rect, scale);
    let x = rect_px.left as f32;
    let y = rect_px.top as f32;
    let w = (rect_px.right - rect_px.left).max(1) as f32;
    let h = (rect_px.bottom - rect_px.top).max(1) as f32;
    let radius_px = decoration
        .corner_radius_pt
        .map(|r| pt_i64_to_px_i32(r, scale).max(0) as f32)
        .unwrap_or(0.0);

    let Some(path) = rounded_rect_path(x, y, w, h, radius_px) else {
        return Ok(());
    };

    if let Some(fill) = resolve_fill(decoration)? {
        fill_path_with_fill(pixmap, &path, rect_px, &fill)?;
    }

    if let Some(border) = &decoration.border {
        stroke_border(pixmap, x, y, w, h, border, scale)?;
    }

    Ok(())
}

fn stroke_border(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    border: &Border,
    scale: f32,
) -> Result<(), PaintError> {
    let color = parse_hex_color_rgba8(&border.color).unwrap_or(Color::from_rgba8(0, 0, 0, 0));
    let width_px = pt_i64_to_px_i32(border.width_pt, scale).max(0) as f32;
    if width_px <= 0.0 || color.alpha() == 0.0 || border.edges.is_empty() {
        return Ok(());
    }

    let mut paint = Paint::default();
    paint.set_color(color);
    let stroke = make_stroke(width_px, border.style, scale);

    if border.is_full_rect_stroke() {
        if let Some(path) = rounded_rect_path(x, y, w, h, 0.0) {
            pixmap
                .as_mut()
                .stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
        return Ok(());
    }

    // Partial edges / dashed: draw per-edge straight segments (corner radius ignored).
    for edge in &border.edges {
        let mut pb = PathBuilder::new();
        match edge {
            BorderEdge::Top => {
                pb.move_to(x, y);
                pb.line_to(x + w, y);
            }
            BorderEdge::Right => {
                pb.move_to(x + w, y);
                pb.line_to(x + w, y + h);
            }
            BorderEdge::Bottom => {
                pb.move_to(x, y + h);
                pb.line_to(x + w, y + h);
            }
            BorderEdge::Left => {
                pb.move_to(x, y);
                pb.line_to(x, y + h);
            }
        }
        if let Some(path) = pb.finish() {
            pixmap
                .as_mut()
                .stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
    Ok(())
}

fn make_stroke(width_px: f32, style: BorderStyle, scale: f32) -> Stroke {
    let mut stroke = Stroke::default();
    stroke.width = width_px;
    stroke.dash = match style {
        BorderStyle::Solid => None,
        BorderStyle::Dashed => {
            let on = pt_i64_to_px_i32(3000, scale).max(1) as f32;
            let off = pt_i64_to_px_i32(2000, scale).max(1) as f32;
            StrokeDash::new(vec![on, off], 0.0)
        }
        BorderStyle::Dotted => {
            let on = pt_i64_to_px_i32(1000, scale).max(1) as f32;
            let off = pt_i64_to_px_i32(1000, scale).max(1) as f32;
            StrokeDash::new(vec![on, off], 0.0)
        }
    };
    stroke
}
