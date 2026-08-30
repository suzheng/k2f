use k2f_paint::OFFICIAL_PNG_SCALE;
use k2f_reader::ui::PageView;

#[test]
fn window_pixels_divide_by_zoom_times_official_scale() {
    let view = PageView::fitted(1190, 1684, 1190, 1684, 1.0);
    assert_eq!(view.origin_x, 0.0);
    assert_eq!(view.origin_y, 0.0);

    let (x, y) = view.window_to_pt(0.0, 0.0);
    assert_eq!((x, y), (0.0, 0.0));

    let (x, y) = view.window_to_pt(
        100.0 * OFFICIAL_PNG_SCALE as f64,
        40.0 * OFFICIAL_PNG_SCALE as f64,
    );
    assert!((x - 100.0).abs() < 1e-9, "{x}");
    assert!((y - 40.0).abs() < 1e-9, "{y}");
}

#[test]
fn zoom_scales_the_bitmap_not_the_lock() {
    let view = PageView::fitted(2380, 3368, 1190, 1684, 2.0);
    let (x, y) = view.window_to_pt(
        100.0 * 2.0 * OFFICIAL_PNG_SCALE as f64,
        40.0 * 2.0 * OFFICIAL_PNG_SCALE as f64,
    );
    assert!((x - 100.0).abs() < 1e-9, "{x}");
    assert!((y - 40.0).abs() < 1e-9, "{y}");
}

#[test]
fn letterbox_origin_is_subtracted_before_pt_conversion() {
    let view = PageView::fitted(1400, 2000, 1190, 1684, 1.0);
    assert!(view.origin_x > 0.0);
    assert!(view.origin_y > 0.0);
    let (x, y) = view.window_to_pt(view.origin_x, view.origin_y);
    assert!((x.abs() + y.abs()) < 1e-9, "{x},{y}");
}

#[test]
fn pt_roundtrip_through_window_space() {
    let view = PageView::fitted(1300, 1800, 1190, 1684, 1.25);
    let (wx, wy) = view.pt_to_window(72.0, 36.0);
    let (x, y) = view.window_to_pt(wx, wy);
    assert!((x - 72.0).abs() < 1e-6, "{x}");
    assert!((y - 36.0).abs() < 1e-6, "{y}");
}
