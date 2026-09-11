mod common;

use common::published_invoice_bytes;
use k2f_reader::ui::{
    content_y_from_anchor, content_y_under, scroll_to_keep_anchor, stack_anchor,
    zoom_after_ctrl_wheel, zoom_after_pinch, Action, Session, LINE_PX, PAGE_GAP,
};
use k2f_reader::AppState;

#[test]
fn pinch_magnifies_and_shrinks() {
    assert!((zoom_after_pinch(1.0, 0.2) - 1.2).abs() < 1e-6);
    assert!((zoom_after_pinch(2.0, -0.25) - 1.5).abs() < 1e-6);
    assert_eq!(zoom_after_pinch(1.0, f64::NAN), 1.0);
}

#[test]
fn ctrl_wheel_up_zooms_in() {
    let z = zoom_after_ctrl_wheel(1.0, LINE_PX);
    assert!((z - 1.1).abs() < 1e-6, "one line ≈ +10%, got {z}");
    let out = zoom_after_ctrl_wheel(1.0, -LINE_PX);
    assert!((out - 0.9).abs() < 1e-6);
}

#[test]
fn stack_anchor_pins_gap_to_page_bottom() {
    let heights = [100u32, 100];
    assert_eq!(stack_anchor(0.0, &heights), (0, 0.0));
    assert_eq!(stack_anchor(50.0, &heights), (0, 0.5));
    assert_eq!(stack_anchor(100.0, &heights), (0, 1.0));
    let in_gap = 100.0 + PAGE_GAP * 0.5;
    assert_eq!(stack_anchor(in_gap, &heights), (0, 1.0));
    assert_eq!(stack_anchor(100.0 + PAGE_GAP + 25.0, &heights), (1, 0.25));
}

#[test]
fn scroll_anchor_keeps_fraction_under_focus() {
    let old = [200u32, 200];
    let new = [400u32, 400];
    let inset = 40.0;
    let focus = 140.0;
    let scroll = 80.0;
    let cy = content_y_under(scroll, focus, inset);
    let (page, frac) = stack_anchor(cy, &old);
    let kept = scroll_to_keep_anchor(scroll, focus, inset, &old, &new);
    let cy2 = content_y_under(kept, focus, inset);
    let (page2, frac2) = stack_anchor(cy2, &new);
    assert_eq!(page, page2);
    assert!((frac - frac2).abs() < 1e-9);
    assert!((content_y_from_anchor(page, frac, &new) - cy2).abs() < 1e-9);
}

#[test]
fn scroll_anchor_ignores_focus_below_stack() {
    let old = [100u32];
    let new = [200u32];
    let inset = 40.0;
    let kept = scroll_to_keep_anchor(0.0, inset + 150.0, inset, &old, &new);
    assert_eq!(kept, 0.0, "letterbox focus must not invent scroll");
}

#[test]
fn published_invoice_step_zoom_out_in_restores_scroll() {
    let mut session = Session::new(AppState::open(&published_invoice_bytes()).unwrap()).unwrap();
    session.apply(Action::ZoomOut);
    session.apply(Action::ZoomIn);
    assert!((session.app().unwrap().zoom() - 1.0).abs() < 1e-6);
    assert!(
        session.scroll_y().abs() < 1.0,
        "fit-height zoom cycle must not leave residual scroll, got {}",
        session.scroll_y()
    );
}

#[test]
fn published_invoice_pinch_keeps_focus_point() {
    let mut session = Session::new(AppState::open(&published_invoice_bytes()).unwrap()).unwrap();
    let (w, h) = session.scaled_size();
    session.set_window_size(w, h);
    session.scroll_by(120.0);

    let focus_y = session.viewport_focus_y();
    let before_scroll = session.scroll_y();
    assert!((session.app().unwrap().zoom() - 1.0).abs() < 1e-6);

    session.set_zoom_about(zoom_after_pinch(1.0, 0.5), Some(focus_y));
    assert!((session.app().unwrap().zoom() - 1.5).abs() < 1e-5);
    assert!(
        session.scroll_y() > before_scroll,
        "zooming in mid-document should increase scroll_y to hold the anchor"
    );

    session.set_zoom_about(1.0, Some(focus_y));
    assert!(
        (session.scroll_y() - before_scroll).abs() < 2.0,
        "zoom out to 1.0 should restore scroll (~{}), got {}",
        before_scroll,
        session.scroll_y()
    );
}
