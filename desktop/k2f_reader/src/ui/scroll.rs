//! Scroll offset for the stacked page viewport.

pub fn clamp_scroll(scroll: f64, content_h: f64, viewport_h: f64) -> f64 {
    let max = (content_h - viewport_h).max(0.0);
    scroll.clamp(0.0, max)
}

/// Mouse-wheel line delta → window pixels. Positive is scroll down.
pub const LINE_PX: f64 = 40.0;

pub fn line_delta_px(lines: f32) -> f64 {
    f64::from(lines) * LINE_PX
}
