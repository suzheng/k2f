//! Word-style vertical stack of lock pages.

use super::coords::{scaled_png_size, PageView};
use super::hud::HUD_HEIGHT;

/// Gutter between sheets, matching `.k2f-stack { gap: 24px }`.
pub const PAGE_GAP: f64 = 24.0;

/// Last page whose top is at or above viewportTop + 25% of viewport height.
pub fn page_at_scroll(scroll_top: f64, viewport_height: f64, page_tops: &[f64]) -> usize {
    if page_tops.is_empty() {
        return 0;
    }
    let probe = scroll_top + viewport_height * 0.25;
    for i in (0..page_tops.len()).rev() {
        if page_tops[i] <= probe {
            return i;
        }
    }
    0
}

pub fn page_tops(heights: &[u32]) -> Vec<f64> {
    let mut tops = Vec::with_capacity(heights.len());
    let mut y = 0.0;
    for (i, &h) in heights.iter().enumerate() {
        tops.push(y);
        y += f64::from(h);
        if i + 1 < heights.len() {
            y += PAGE_GAP;
        }
    }
    tops
}

pub fn content_height(heights: &[u32]) -> f64 {
    if heights.is_empty() {
        return 0.0;
    }
    heights.iter().map(|&h| f64::from(h)).sum::<f64>()
        + PAGE_GAP * heights.len().saturating_sub(1) as f64
}

pub fn origin_y(page: usize, tops: &[f64], scroll_y: f64) -> f64 {
    f64::from(HUD_HEIGHT) + tops.get(page).copied().unwrap_or(0.0) - scroll_y
}

pub fn page_view(win_w: u32, png_w: u32, png_h: u32, zoom: f32, origin_y: f64) -> PageView {
    let (sw, _) = scaled_png_size(png_w, png_h, zoom);
    PageView {
        origin_x: (f64::from(win_w) - f64::from(sw)) * 0.5,
        origin_y,
        zoom,
        png_w,
        png_h,
    }
}

pub fn hit_index(x: f64, y: f64, views: &[PageView]) -> Option<usize> {
    views.iter().position(|v| {
        let (sw, sh) = v.scaled_size();
        x >= v.origin_x
            && x < v.origin_x + f64::from(sw)
            && y >= v.origin_y
            && y < v.origin_y + f64::from(sh)
    })
}
