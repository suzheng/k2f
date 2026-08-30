mod common;

use common::{invoice_bytes, published_invoice_bytes};
use k2f_reader::ui::{overlay_label, Action, Session, HUD_HEIGHT};
use k2f_reader::AppState;

/// Web `--banner-unsigned` (`sdk/js/viewer/styles.js`).
const WEB_UNSIGNED: u32 = 0x3D5A80;

#[test]
fn zoom_one_blits_official_png_pixels_below_hud() {
    let session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    let (w, h) = session.scaled_size();
    let view = session.page_view(w, h);
    assert!(
        (view.origin_y - f64::from(HUD_HEIGHT)).abs() < 1e-9,
        "page origin must sit below the reserved banner, got {}",
        view.origin_y
    );
    let frame = session.compose_frame(w, h);
    assert_eq!(frame.len(), w as usize * h as usize);

    let png_y = 16u32;
    let x = w / 2;
    let expected = session
        .official_pixel(x, png_y)
        .expect("sample point is inside the official PNG");
    let frame_y = view.origin_y.round() as u32 + png_y;
    assert_eq!(
        frame[(frame_y * w + x) as usize],
        expected,
        "zoom 1 must 1:1 blit the lock PNG under the banner, not a re-paint"
    );
}

#[test]
fn hud_is_a_reserved_strip_not_an_overlay() {
    let session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    let (w, h) = session.scaled_size();
    let view = session.page_view(w, h);
    assert_eq!(view.origin_x, 0.0);
    assert!(
        (view.origin_y - f64::from(HUD_HEIGHT)).abs() < 1e-9,
        "web banner is above the page, got origin_y={}",
        view.origin_y
    );

    let frame = session.compose_frame(w, h);
    let page00 = session
        .official_pixel(0, 0)
        .expect("lock PNG has a top-left pixel");
    assert_ne!(
        frame[0], page00,
        "banner strip must not sit on top of lock pixel (0,0)"
    );
    let y0 = view.origin_y.round() as u32;
    assert_eq!(
        frame[(y0 * w) as usize],
        page00,
        "lock (0,0) must be fully visible just below the HUD"
    );
    assert!(session.official_pixel(w / 2, h.saturating_sub(1)).is_none());
    assert!(session
        .official_pixel(w / 2, h.saturating_sub(HUD_HEIGHT + 1))
        .is_some());
    assert!(overlay_label(session.app()).contains(session.app().banner_str()));
    assert!(h > HUD_HEIGHT);
}

#[test]
fn unsigned_hud_matches_web_banner_color() {
    let session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    assert_eq!(session.app().banner_str(), "UNSIGNED");
    let (w, h) = session.scaled_size();
    let frame = session.compose_frame(w, h);
    assert_eq!(
        frame[w as usize / 2],
        WEB_UNSIGNED,
        "HUD must use web --banner-unsigned, not the unlocked amber"
    );
    assert!(h > 0);
}

#[test]
fn zoom_changes_window_size_not_lock_bytes() {
    let mut session = Session::new(AppState::open(&published_invoice_bytes()).unwrap()).unwrap();
    let png0 = session.app().render_current_png().unwrap();
    let (w0, h0) = session.scaled_size();
    session.apply(Action::ZoomIn);
    let png1 = session.app().render_current_png().unwrap();
    let (w1, h1) = session.scaled_size();
    assert_eq!(png0, png1, "zoom is UI scale of the official bitmap");
    assert!(
        w1 > w0 && h1 > h0,
        "window grows with zoom, {w0}x{h0} -> {w1}x{h1}"
    );
    assert!((session.app().zoom() - 1.25).abs() < 1e-6);
}
