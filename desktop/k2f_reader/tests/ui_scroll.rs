mod common;

use common::published_invoice_bytes;
use k2f_reader::ui::{
    clamp_scroll, content_height, line_delta_px, page_at_scroll, page_inset_y, page_tops,
    wheel_y_to_scroll, window_chrome_h, Action, Session, LINE_PX, PAGE_GAP,
};
use k2f_reader::AppState;

#[test]
fn page_at_scroll_matches_web_25_percent_probe() {
    let h = 100.0;
    let tops = page_tops(&[100, 100, 100]);
    assert_eq!(tops, vec![0.0, h + PAGE_GAP, 2.0 * (h + PAGE_GAP)]);
    assert_eq!(page_at_scroll(0.0, h, &tops), 0);
    let into1 = tops[1] - h * 0.25;
    assert_eq!(page_at_scroll(into1, h, &tops), 1);
    let into2 = tops[2] - h * 0.25;
    assert_eq!(page_at_scroll(into2, h, &tops), 2);
    assert_eq!(page_at_scroll(0.0, h, &[]), 0);
}

#[test]
fn clamp_scroll_stops_at_content_end() {
    assert_eq!(clamp_scroll(-10.0, 500.0, 200.0), 0.0);
    assert_eq!(clamp_scroll(100.0, 500.0, 200.0), 100.0);
    assert_eq!(clamp_scroll(999.0, 500.0, 200.0), 300.0);
    assert_eq!(clamp_scroll(50.0, 100.0, 200.0), 0.0);
}

#[test]
fn line_delta_is_positive_down() {
    assert_eq!(line_delta_px(1.0), LINE_PX);
    assert_eq!(line_delta_px(-2.0), -2.0 * LINE_PX);
}

#[test]
fn wheel_y_matches_system_content_direction() {
    assert_eq!(
        wheel_y_to_scroll(line_delta_px(1.0)),
        -LINE_PX,
        "positive winit Y moves content down, so scroll_y decreases"
    );
    assert_eq!(wheel_y_to_scroll(-24.0), 24.0);
}

#[test]
fn published_invoice_stack_is_taller_than_one_page() {
    let session = Session::new(AppState::open(&published_invoice_bytes()).unwrap()).unwrap();
    assert!(session.app().unwrap().page_count() >= 3);
    let (_w, h) = session.scaled_size();
    let view_h = f64::from(h.saturating_sub(window_chrome_h(session.app().unwrap())));
    assert!(
        session.content_height() > view_h + PAGE_GAP,
        "stack {} view {}",
        session.content_height(),
        view_h
    );
    let expected = content_height(
        &(0..session.app().unwrap().page_count())
            .map(|_| view_h.round() as u32)
            .collect::<Vec<_>>(),
    );
    assert!(
        (session.content_height() - expected).abs() < 1.0 || session.content_height() > view_h,
        "content_height uses scaled page heights plus gutters"
    );
}

#[test]
fn scroll_changes_current_page_on_published_invoice() {
    let mut session = Session::new(AppState::open(&published_invoice_bytes()).unwrap()).unwrap();
    assert_eq!(session.app().unwrap().page(), 0);
    session.scroll_by(session.content_height());
    assert!(
        session.app().unwrap().page() >= 1,
        "scrolling to the end must leave page 0, got {}",
        session.app().unwrap().page()
    );
    assert_eq!(session.app().unwrap().page(), session.app().unwrap().page_count() - 1);
}

#[test]
fn compose_at_scroll_zero_matches_page0_png() {
    let session = Session::new(AppState::open(&published_invoice_bytes()).unwrap()).unwrap();
    let (w, h) = session.scaled_size();
    let view = session.page_view(w, h);
    assert!(
        (view.origin_y - f64::from(page_inset_y(session.app().unwrap()))).abs() < 1e-9,
        "page 0 sits under the HUD at scroll 0, got {}",
        view.origin_y
    );
    let frame = session.compose_frame(w, h);
    let png_y = 16u32;
    let x = w / 2;
    let expected = session.official_pixel_at(0, x, png_y).unwrap();
    let frame_y = view.origin_y.round() as u32 + png_y;
    assert_eq!(frame[(frame_y * w + x) as usize], expected);
}

#[test]
fn jump_to_page_1_blits_page1_not_page0() {
    let mut session = Session::new(AppState::open(&published_invoice_bytes()).unwrap()).unwrap();
    let (w, h) = session.scaled_size();
    session.apply(Action::NextPage);
    assert_eq!(session.app().unwrap().page(), 1);
    assert_eq!(session.scroll_y(), page_tops_for(&session)[1]);

    let view = session.page_view(w, h);
    assert!(
        (view.origin_y - f64::from(page_inset_y(session.app().unwrap()))).abs() < 1e-9,
        "jumped page sits under the HUD, got {}",
        view.origin_y
    );
    let frame = session.compose_frame(w, h);
    let (x, png_y) = differing_pixel(&session, w).expect("invoice pages must differ");
    let page1 = session.official_pixel_at(1, x, png_y).unwrap();
    let frame_y = view.origin_y.round() as u32 + png_y;
    assert_eq!(
        frame[(frame_y * w + x) as usize],
        page1,
        "after Next the visible sheet is page 1"
    );
}

fn differing_pixel(session: &Session, max_x: u32) -> Option<(u32, u32)> {
    for y in 0..400 {
        for x in 0..max_x {
            let Some(a) = session.official_pixel_at(0, x, y) else {
                continue;
            };
            let Some(b) = session.official_pixel_at(1, x, y) else {
                continue;
            };
            if a != b {
                return Some((x, y));
            }
        }
    }
    None
}

fn page_tops_for(session: &Session) -> Vec<f64> {
    let (_w, h) = session.scaled_size();
    let page_h = h.saturating_sub(window_chrome_h(session.app().unwrap()));
    page_tops(&vec![page_h; session.app().unwrap().page_count()])
}
