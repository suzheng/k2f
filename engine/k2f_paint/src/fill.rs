use k2f_core::{BoxDecoration, Fill, FillRef, LinearGradient, Shadow, ShadowRef};
use tiny_skia::{FillRule, Paint, Path, Pixmap, Transform};

use crate::error::PaintError;
use crate::geom::{parse_hex_color_rgba8, RectPx};

pub fn resolve_fill(decoration: &BoxDecoration) -> Result<Option<Fill>, PaintError> {
    match decoration.background.as_ref() {
        None => Ok(None),
        Some(FillRef::Inline(f)) => Ok(Some(f.clone())),
        Some(FillRef::Ref(name)) => Err(PaintError::UnresolvedRef(name.clone())),
    }
}

pub(crate) fn resolve_shadow(decoration: &BoxDecoration) -> Result<Option<Shadow>, PaintError> {
    match decoration.shadow.as_ref() {
        None => Ok(None),
        Some(ShadowRef::Inline(s)) => Ok(Some(s.clone())),
        Some(ShadowRef::Ref(name)) => Err(PaintError::UnresolvedRef(name.clone())),
    }
}

pub(crate) fn fill_path_with_fill(
    pixmap: &mut Pixmap,
    path: &Path,
    rect_px: RectPx,
    fill: &Fill,
) -> Result<(), PaintError> {
    match fill {
        Fill::Solid { color } => {
            let c =
                parse_hex_color_rgba8(color).unwrap_or(tiny_skia::Color::from_rgba8(0, 0, 0, 0));
            if c.alpha() == 0.0 {
                return Ok(());
            }
            let mut paint = Paint::default();
            paint.set_color(c);
            pixmap
                .as_mut()
                .fill_path(path, &paint, FillRule::Winding, Transform::identity(), None);
            Ok(())
        }
        Fill::LinearGradient { value } => {
            fill_path_with_linear_gradient(pixmap, path, rect_px, value)
        }
    }
}

fn fill_path_with_linear_gradient(
    pixmap: &mut Pixmap,
    path: &Path,
    rect_px: RectPx,
    grad: &LinearGradient,
) -> Result<(), PaintError> {
    use tiny_skia::{GradientStop, LinearGradient as TsLinearGradient, Point, SpreadMode};

    let LinearGradient::Linear {
        angle_degrees,
        stops,
    } = grad;

    let cx = (rect_px.left + rect_px.right) as f32 * 0.5;
    let cy = (rect_px.top + rect_px.bottom) as f32 * 0.5;
    let w = (rect_px.right - rect_px.left).max(1) as f32;
    let h = (rect_px.bottom - rect_px.top).max(1) as f32;

    let theta = (*angle_degrees as f32) * std::f32::consts::PI / 180.0;
    let dir_x = theta.cos();
    let dir_y = theta.sin();

    let corners = [
        (-w * 0.5, -h * 0.5),
        (w * 0.5, -h * 0.5),
        (-w * 0.5, h * 0.5),
        (w * 0.5, h * 0.5),
    ];
    let mut min_t = f32::INFINITY;
    let mut max_t = f32::NEG_INFINITY;
    for (dx, dy) in corners {
        let t = dx * dir_x + dy * dir_y;
        min_t = min_t.min(t);
        max_t = max_t.max(t);
    }

    let start = Point::from_xy(cx + dir_x * min_t, cy + dir_y * min_t);
    let end = Point::from_xy(cx + dir_x * max_t, cy + dir_y * max_t);

    let ts_stops: Vec<GradientStop> = stops
        .iter()
        .filter_map(|s| {
            let c = parse_hex_color_rgba8(&s.color)?;
            Some(GradientStop::new(
                (s.pos as f32 / 1000.0).clamp(0.0, 1.0),
                c,
            ))
        })
        .collect();
    if ts_stops.is_empty() {
        return Ok(());
    }

    let shader =
        TsLinearGradient::new(start, end, ts_stops, SpreadMode::Pad, Transform::identity())
            .ok_or(PaintError::Gradient)?;

    let mut paint = Paint::default();
    paint.shader = shader;
    pixmap
        .as_mut()
        .fill_path(path, &paint, FillRule::Winding, Transform::identity(), None);
    Ok(())
}
