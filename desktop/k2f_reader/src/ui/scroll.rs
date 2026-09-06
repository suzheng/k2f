//! Scroll offset for the stacked page viewport.

pub fn clamp_scroll(scroll: f64, content_h: f64, viewport_h: f64) -> f64 {
    let max = (content_h - viewport_h).max(0.0);
    scroll.clamp(0.0, max)
}

/// Mouse-wheel line delta → window pixels. Sign matches winit: positive Y
/// means the content should move down.
pub const LINE_PX: f64 = 40.0;

pub fn line_delta_px(lines: f32) -> f64 {
    f64::from(lines) * LINE_PX
}

/// Map a winit wheel Y into a `scroll_y` delta.
///
/// winit (and macOS) report positive Y when the page should move down — the
/// same direction Preview and other system apps use, including Natural
/// Scrolling. Our `scroll_y` grows as we move down the document, which moves
/// content *up*, so the sign is inverted here.
pub fn wheel_y_to_scroll(wheel_y: f64) -> f64 {
    -wheel_y
}
