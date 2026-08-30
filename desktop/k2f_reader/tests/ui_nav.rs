mod common;

use common::{invoice_bytes, published_invoice_bytes};
use k2f_paint::TextSpan;
use k2f_reader::copy::RectPt;
use k2f_reader::ui::{accept_key, key_action, Action, KeyBind, Session, HUD_HEIGHT};
use k2f_reader::AppState;

fn title_span(spans: &[TextSpan]) -> &TextSpan {
    spans
        .iter()
        .find(|s| s.text.contains("STATEMENT"))
        .expect("invoice heading is on the text layer")
}

#[test]
fn arrows_and_zoom_keys_map_to_actions() {
    assert_eq!(
        key_action(KeyBind::Left, false, false, false),
        Some(Action::PrevPage)
    );
    assert_eq!(
        key_action(KeyBind::Right, false, false, false),
        Some(Action::NextPage)
    );
    assert_eq!(
        key_action(KeyBind::Plus, false, false, false),
        Some(Action::ZoomIn)
    );
    assert_eq!(
        key_action(KeyBind::Minus, false, false, false),
        Some(Action::ZoomOut)
    );
    assert_eq!(
        key_action(KeyBind::Char('c'), true, false, false),
        Some(Action::Copy)
    );
    assert_eq!(
        key_action(KeyBind::Char('c'), false, false, true),
        Some(Action::Copy)
    );
    assert_eq!(key_action(KeyBind::Char('c'), false, false, false), None);
    assert!(accept_key(true, Action::NextPage), "held arrows page");
    assert!(accept_key(true, Action::ZoomIn), "held +/- zoom");
    assert!(
        !accept_key(true, Action::Copy),
        "key-repeat must not spam the clipboard"
    );
}

#[test]
fn arrow_keys_page_the_published_invoice() {
    let mut session = Session::new(AppState::open(&published_invoice_bytes()).unwrap()).unwrap();
    assert!(session.app().page_count() >= 2);
    assert_eq!(session.app().page(), 0);
    let page0 = session.app().render_current_png().unwrap();

    session.apply(Action::NextPage);
    assert_eq!(session.app().page(), 1);
    let page1 = session.app().render_current_png().unwrap();
    assert_ne!(page0, page1);

    session.apply(Action::PrevPage);
    assert_eq!(session.app().page(), 0);
    assert_eq!(session.app().render_current_png().unwrap(), page0);
}

#[test]
fn drag_on_page_1_copies_that_page_lock_text() {
    let mut session = Session::new(AppState::open(&published_invoice_bytes()).unwrap()).unwrap();
    session.apply(Action::NextPage);
    assert_eq!(session.app().page(), 1);
    let spans = session.app().text_layer();
    let span = spans
        .iter()
        .find(|s| !s.text.is_empty())
        .expect("page 1 has a text layer")
        .clone();
    let (w, h) = session.scaled_size();
    let view = session.page_view(w, h);
    let (x0, y0) = view.pt_to_window(
        span.x_pt + span.width_pt * 0.25,
        span.y_pt + span.height_pt * 0.25,
    );
    let (x1, y1) = view.pt_to_window(
        span.x_pt + span.width_pt * 0.75,
        span.y_pt + span.height_pt * 0.75,
    );
    session.pointer_down(x0, y0);
    let payload = session
        .pointer_up(x1, y1)
        .expect("drag on page 1 must copy that page");
    assert!(
        payload.plain.contains(&span.text)
            || payload.nodes.iter().any(|n| n.node_id == span.node_id),
        "page-1 drag must map through the stacked page view, got {:?}",
        payload.plain
    );
}

#[test]
fn drag_in_window_pixels_copies_lock_text() {
    let mut session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    let title = title_span(&session.app().text_layer()).clone();
    let (w, h) = session.scaled_size();
    let view = session.page_view(w, h);
    let (x0, y0) = view.pt_to_window(
        title.x_pt + title.width_pt * 0.25,
        title.y_pt + title.height_pt * 0.25,
    );
    let (x1, y1) = view.pt_to_window(
        title.x_pt + title.width_pt * 0.75,
        title.y_pt + title.height_pt * 0.75,
    );

    session.pointer_down(x0, y0);
    let payload = session
        .pointer_up(x1, y1)
        .expect("drag over title must copy");
    assert!(
        payload.plain.contains("STATEMENT"),
        "window-space drag must map through zoom * OFFICIAL_PNG_SCALE, got {:?}",
        payload.plain
    );
    assert!(payload.nodes.iter().any(|n| n.node_id == title.node_id));
}

#[test]
fn collapsed_click_does_not_copy() {
    let mut session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    let title = title_span(&session.app().text_layer()).clone();
    let (w, h) = session.scaled_size();
    let (x, y) = session.page_view(w, h).pt_to_window(
        title.x_pt + title.width_pt * 0.5,
        title.y_pt + title.height_pt * 0.5,
    );
    session.pointer_down(x, y);
    assert!(
        session.pointer_up(x, y).is_none(),
        "zero-area drag matches a collapsed web selection"
    );
}

#[test]
fn ctrl_c_copies_active_selection() {
    let mut session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    let spans = session.app().text_layer();
    let title = title_span(&spans);
    let sel = RectPt::from_drag(
        title.x_pt,
        title.y_pt,
        title.x_pt + title.width_pt,
        title.y_pt + title.height_pt,
    );
    session.set_selection(sel);
    let payload = session.apply(Action::Copy).expect("Ctrl+C with selection");
    assert!(payload.plain.contains("STATEMENT"));
}

#[test]
fn hud_is_excluded_from_page_coordinates() {
    let session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    let (w, h) = session.scaled_size();
    let view = session.page_view(w, h);
    let (x, y) = view.window_to_pt(0.0, HUD_HEIGHT as f64);
    assert!(x.abs() < 1e-9, "{x}");
    assert!(
        y.abs() < 1e-9,
        "window y=HUD_HEIGHT is document pt 0, got {y}"
    );
}

#[test]
fn drag_starting_on_hud_does_not_copy() {
    let mut session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    let title = title_span(&session.app().text_layer()).clone();
    let (w, h) = session.scaled_size();
    let (x1, y1) = session.page_view(w, h).pt_to_window(
        title.x_pt + title.width_pt * 0.5,
        title.y_pt + title.height_pt * 0.5,
    );
    session.pointer_down(8.0, 4.0);
    assert!(
        session.pointer_up(x1, y1).is_none(),
        "banner chrome is not a text-layer drag, matching the web header"
    );
}
