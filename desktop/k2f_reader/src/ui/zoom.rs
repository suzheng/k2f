//! Continuous UI zoom from trackpad pinch and Ctrl+wheel.
//!
//! View-scale only — does not recompile or re-layout the lock.

use super::stack::{content_height, page_tops};

/// AppKit / winit pinch: `delta` is relative magnification (`0` = no change).
pub fn zoom_after_pinch(zoom: f32, delta: f64) -> f32 {
    if !delta.is_finite() {
        return zoom;
    }
    zoom * (1.0 + delta as f32)
}

/// Ctrl+wheel: positive wheel Y (scroll-up) zooms in. ~10% per `LINE_PX` line.
pub fn zoom_after_ctrl_wheel(zoom: f32, wheel_y: f64) -> f32 {
    if !wheel_y.is_finite() {
        return zoom;
    }
    let factor = (wheel_y * 0.0025).clamp(-0.35, 0.35);
    zoom * (1.0 + factor as f32)
}

pub fn content_y_under(scroll_y: f64, focus_y: f64, inset_y: f64) -> f64 {
    scroll_y + (focus_y - inset_y)
}

/// `(page, fraction within page height)`. Gap after a page pins to that page's bottom.
pub fn stack_anchor(content_y: f64, heights: &[u32]) -> (usize, f64) {
    if heights.is_empty() {
        return (0, 0.0);
    }
    let tops = page_tops(heights);
    for i in 0..heights.len() {
        let top = tops[i];
        let h = f64::from(heights[i]).max(1.0);
        let bottom = top + h;
        let next_top = tops.get(i + 1).copied().unwrap_or(f64::INFINITY);
        if content_y < next_top || i + 1 == heights.len() {
            if content_y <= bottom {
                return (i, ((content_y - top) / h).clamp(0.0, 1.0));
            }
            return (i, 1.0);
        }
    }
    (heights.len() - 1, 1.0)
}

pub fn content_y_from_anchor(page: usize, frac: f64, heights: &[u32]) -> f64 {
    let tops = page_tops(heights);
    let top = tops.get(page).copied().unwrap_or(0.0);
    let h = f64::from(heights.get(page).copied().unwrap_or(0));
    top + frac.clamp(0.0, 1.0) * h
}

/// New `scroll_y` so the stack point under `focus_y` stays fixed after heights change.
///
/// If the focus is above/below the document stack (letterbox), leave `old_scroll`
/// unchanged so zoom-out-to-fit does not pin the page bottom to the viewport center.
pub fn scroll_to_keep_anchor(
    old_scroll: f64,
    focus_y: f64,
    inset_y: f64,
    old_heights: &[u32],
    new_heights: &[u32],
) -> f64 {
    let cy = content_y_under(old_scroll, focus_y, inset_y);
    let old_h = content_height(old_heights);
    if cy < 0.0 || cy > old_h {
        return old_scroll;
    }
    let (page, frac) = stack_anchor(cy, old_heights);
    content_y_from_anchor(page, frac, new_heights) - (focus_y - inset_y)
}
