//! Display paint LOD: continuous UI zoom, quantized `render_page` scale.
//! Official 2× stays the baseline; screen may use a higher bucket.

use k2f_paint::OFFICIAL_PNG_SCALE;
use std::time::Duration;

/// Idle time after the last zoom change before upgrading display rasters.
pub const DISPLAY_PAINT_DEBOUNCE: Duration = Duration::from_millis(120);

/// Paint-scale buckets (pixels per document pt). Max matches PDF stamp cap.
pub const PAINT_BUCKETS: [f32; 7] = [1.0, 1.25, 1.5, 2.0, 2.5, 3.0, 4.0];

/// Desktop dest size is `official_png × ui_zoom`, so needed paint is `OFFICIAL × zoom`.
pub fn needed_paint_scale(ui_zoom: f32) -> f32 {
    if !ui_zoom.is_finite() || ui_zoom <= 0.0 {
        return OFFICIAL_PNG_SCALE;
    }
    OFFICIAL_PNG_SCALE * ui_zoom
}

/// Smallest bucket `>= needed`, clamped to the last bucket.
pub fn quantize_paint_scale(needed: f32) -> f32 {
    if !needed.is_finite() || needed <= 0.0 {
        return OFFICIAL_PNG_SCALE;
    }
    let max = *PAINT_BUCKETS.last().unwrap();
    if needed >= max {
        return max;
    }
    for &b in &PAINT_BUCKETS {
        if b + f32::EPSILON >= needed {
            return b;
        }
    }
    max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needed_is_official_times_zoom() {
        assert!((needed_paint_scale(1.0) - 2.0).abs() < 1e-6);
        assert!((needed_paint_scale(1.5) - 3.0).abs() < 1e-6);
        assert!((needed_paint_scale(2.0) - 4.0).abs() < 1e-6);
    }

    #[test]
    fn quantize_picks_ceiling_bucket() {
        assert_eq!(quantize_paint_scale(2.0), 2.0);
        assert_eq!(quantize_paint_scale(2.01), 2.5);
        assert_eq!(quantize_paint_scale(2.5), 2.5);
        assert_eq!(quantize_paint_scale(2.51), 3.0);
        assert_eq!(quantize_paint_scale(3.0), 3.0);
        assert_eq!(quantize_paint_scale(3.1), 4.0);
        assert_eq!(quantize_paint_scale(9.0), 4.0);
        assert_eq!(quantize_paint_scale(0.5), 1.0);
    }

    #[test]
    fn zoom_one_point_five_needs_bucket_three() {
        assert_eq!(quantize_paint_scale(needed_paint_scale(1.5)), 3.0);
    }
}
