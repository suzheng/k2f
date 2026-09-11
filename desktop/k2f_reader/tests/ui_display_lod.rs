mod common;

use common::invoice_bytes;
use k2f_paint::OFFICIAL_PNG_SCALE;
use k2f_reader::ui::display_scale::{needed_paint_scale, quantize_paint_scale};
use k2f_reader::ui::{Action, Session};
use k2f_reader::AppState;

#[test]
fn continuous_zoom_accepted() {
    let mut session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    session.set_zoom(1.13);
    assert!((session.app().unwrap().zoom() - 1.13).abs() < 1e-6);
}

#[test]
fn zoom_does_not_change_official_baseline_bytes() {
    let mut session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    let before = session.app().unwrap().render_page_png(0).unwrap();
    session.set_zoom(1.5);
    session.flush_display_lod();
    let after = session.app().unwrap().render_page_png(0).unwrap();
    assert_eq!(before, after, "official 2× bytes must stay the contract");
}

#[test]
fn display_lod_upgrades_visible_page_bucket() {
    let mut session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    let (w, h) = session.scaled_size();
    session.set_window_size(w, h);
    assert_eq!(session.display_bucket_at(0), None);

    session.set_zoom(1.5);
    assert_eq!(
        quantize_paint_scale(needed_paint_scale(1.5)),
        3.0,
        "1.5× UI zoom needs paint bucket 3"
    );
    assert!(session.flush_display_lod(), "visible page should gain a display raster");
    assert_eq!(session.display_bucket_at(0), Some(3.0));

    session.set_zoom(1.0);
    assert!(session.flush_display_lod());
    assert_eq!(
        session.display_bucket_at(0),
        None,
        "back to official 2× clears display overlay"
    );
}

#[test]
fn render_page_png_at_matches_bucket() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let official = app.render_page_png(0).unwrap();
    let at_official = app.render_page_png_at(0, OFFICIAL_PNG_SCALE).unwrap();
    assert_eq!(official, at_official);
    let denser = app.render_page_png_at(0, 3.0).unwrap();
    assert_ne!(denser.len(), 0);
    assert_ne!(denser, official, "3× paint must differ from official 2×");
}

#[test]
fn stepped_zoom_still_notes_lod() {
    let mut session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    let (w, h) = session.scaled_size();
    session.set_window_size(w, h);
    session.apply(Action::ZoomIn);
    assert!(session.app().unwrap().zoom() > 1.0);
    session.flush_display_lod();
}
