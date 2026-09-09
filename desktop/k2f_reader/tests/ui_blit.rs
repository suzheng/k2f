mod common;

use common::{invoice_bytes, published_invoice_bytes};
use k2f_reader::ui::{overlay_label, page_inset_y, Action, Session, HUD_HEIGHT};
use k2f_reader::AppState;

const TOOLBAR_BG: u32 = 0x3A3A3C;

#[test]
fn zoom_one_blits_official_png_pixels_below_hud() {
    let session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    let (w, h) = session.scaled_size();
    let view = session.page_view(w, h);
    let inset = f64::from(page_inset_y(session.app().unwrap()));
    assert!(
        (view.origin_y - inset).abs() < 1e-9,
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
    let inset = f64::from(page_inset_y(session.app().unwrap()));
    assert!(
        (view.origin_y - inset).abs() < 1e-9,
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
    assert!(session.official_pixel(0, 0).is_some());
    assert!(overlay_label(session.app().unwrap()).contains("1 /"));
    assert!(h > HUD_HEIGHT);
}

#[test]
fn unsigned_hud_matches_neutral_toolbar_color() {
    let session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    assert_eq!(session.app().unwrap().banner_str(), "UNSIGNED");
    let (w, h) = session.scaled_size();
    let frame = session.compose_frame(w, h);
    assert_eq!(
        frame[4], TOOLBAR_BG,
        "quiet unsigned uses a Preview-style dark toolbar, not a VGA strip"
    );
    assert!(h > 0);
}

#[test]
fn zoom_scales_the_bitmap_not_lock_bytes() {
    let mut session = Session::new(AppState::open(&published_invoice_bytes()).unwrap()).unwrap();
    let png0 = session.app().unwrap().render_current_png().unwrap();
    let (w0, h0) = session.scaled_size();
    session.apply(Action::ZoomIn);
    let png1 = session.app().unwrap().render_current_png().unwrap();
    let (w1, h1) = session.scaled_size();
    assert_eq!(png0, png1, "zoom is UI scale of the official bitmap");
    assert!(
        w1 > w0 && h1 > h0,
        "intrinsic page size grows with zoom, {w0}x{h0} -> {w1}x{h1}"
    );
    assert!((session.app().unwrap().zoom() - 1.25).abs() < 1e-6);
    assert_page_interior_is_not_chrome(&session);
}

#[test]
fn zoom_out_samples_the_official_bitmap() {
    let mut session = Session::new(AppState::open(&published_invoice_bytes()).unwrap()).unwrap();
    let before = session.official_pixel(16, 16);
    session.apply(Action::ZoomOut);
    assert!((session.app().unwrap().zoom() - 0.75).abs() < 1e-6);
    assert_eq!(
        session.official_pixel(16, 16),
        before,
        "zoom must not rebuild lock pixels"
    );
    assert_page_interior_is_not_chrome(&session);
    session.apply(Action::ZoomIn);
    assert!((session.app().unwrap().zoom() - 1.0).abs() < 1e-6);
    let (w, h) = session.scaled_size();
    let view = session.page_view(w, h);
    let frame = session.compose_frame(w, h);
    let png_y = 16u32;
    let x = w / 2;
    let expected = session
        .official_pixel(x, png_y)
        .expect("sample point is inside the official PNG");
    let frame_y = view.origin_y.round() as u32 + png_y;
    assert_eq!(
        frame[(frame_y * w + x) as usize],
        expected,
        "returning to zoom 1 must 1:1 blit again"
    );
}

const LETTERBOX: u32 = 0xFBFBFD;
const PAGE_SHADOW: u32 = 0xE4E4E8;

fn assert_page_interior_is_not_chrome(session: &Session) {
    let (w, h) = session.scaled_size();
    let view = session.page_view(w, h);
    let (sw, sh) = view.scaled_size();
    let frame = session.compose_frame(w, h);
    let x = (view.origin_x.round() as u32 + sw / 2).min(w.saturating_sub(1));
    let y = (view.origin_y.round() as u32 + sh / 2).min(h.saturating_sub(1));
    let px = frame[(y * w + x) as usize];
    assert_ne!(px, LETTERBOX, "zoomed page interior must not be the stage");
    assert_ne!(
        px, TOOLBAR_BG,
        "zoomed page interior must not be the toolbar"
    );
    assert_ne!(
        px, PAGE_SHADOW,
        "zoomed page interior must not be the drop shadow"
    );
}
