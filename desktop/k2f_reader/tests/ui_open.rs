mod common;

use common::{invoice_bytes, published_invoice_bytes};
use k2f_reader::ui::{
    accept_key, key_action, open_hit, path_from_open_string, Action, KeyBind, Session, HUD_HEIGHT,
};
use std::path::PathBuf;

#[test]
fn file_url_and_plain_path_parse() {
    assert_eq!(
        path_from_open_string("file:///Users/me/doc.K2F"),
        Some(PathBuf::from("/Users/me/doc.K2F"))
    );
    assert_eq!(
        path_from_open_string("file://localhost/Users/me/My%20Doc.k2f"),
        Some(PathBuf::from("/Users/me/My Doc.k2f"))
    );
    assert_eq!(
        path_from_open_string("/tmp/invoice.K2F"),
        Some(PathBuf::from("/tmp/invoice.K2F"))
    );
    assert_eq!(path_from_open_string("   "), None);
}

#[test]
fn cmd_o_is_open_and_does_not_repeat() {
    assert_eq!(
        key_action(KeyBind::Char('o'), true, false, false),
        Some(Action::Open)
    );
    assert_eq!(
        key_action(KeyBind::Char('O'), false, false, true),
        Some(Action::Open)
    );
    assert_eq!(key_action(KeyBind::Char('o'), false, false, false), None);
    assert!(
        !accept_key(true, Action::Open),
        "held Cmd+O must not spam the Open dialog"
    );
}

#[test]
fn empty_session_paints_open_then_loads_published_invoice() {
    let mut session = Session::empty();
    assert!(session.app().is_none());
    assert_eq!(session.window_title(), "K2F Reader");
    let (w, h) = (1280u32, 820u32);
    let frame = session.compose_frame(w, h);
    assert_eq!(frame.len(), (w * h) as usize);
    assert!(
        open_hit(w, HUD_HEIGHT, 28.0, 24.0),
        "toolbar Open sits on the left"
    );

    session
        .load(&published_invoice_bytes())
        .expect("open published invoice lock");
    let app = session.app().expect("document after load");
    assert!(app.page_count() >= 1);
    assert!(!app.title().is_empty());
    let frame = session.compose_frame(w, h);
    assert_eq!(frame.len(), (w * h) as usize);
}

#[test]
fn failed_open_keeps_previous_document() {
    let mut session = Session::empty();
    assert!(session.load(&[0, 1, 2, 3]).is_err());
    assert!(session.app().is_none());
    session.load(&invoice_bytes()).unwrap();
    let title = session.app().unwrap().title().to_string();
    assert!(session.load(&[0, 1, 2, 3]).is_err());
    assert_eq!(session.app().unwrap().title(), title);
}

#[test]
fn load_replaces_with_real_invoice_bytes() {
    let mut session = Session::empty();
    session.load(&invoice_bytes()).unwrap();
    assert_eq!(session.app().unwrap().title(), "STATEMENT");
    session.load(&published_invoice_bytes()).unwrap();
    assert_ne!(session.app().unwrap().title(), "STATEMENT");
}

#[test]
fn toolbar_open_does_not_steal_export() {
    use k2f_reader::ui::export_hit;
    let w = 800u32;
    let h = HUD_HEIGHT;
    assert!(open_hit(w, h, 28.0, 24.0));
    assert!(!export_hit(w, h, 28.0, 24.0));
    assert!(!open_hit(w, h, 780.0, 24.0));
}
