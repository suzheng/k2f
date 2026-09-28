mod common;

use common::{invoice_bytes, scratch};
use k2f_package::{pack_bytes, unpack_bytes};
use k2f_reader::copy::CopyFormat;
use k2f_reader::ui::{
    accept_key, key_action, reload_hit, Action, ExportFormat, KeyBind, Session, EXPORT_ACTION_LABEL,
};

fn retitled(bytes: &[u8], title: &str) -> Vec<u8> {
    let mut pkg = unpack_bytes(bytes).unwrap();
    pkg.manifest.title = title.into();
    pack_bytes(&pkg).unwrap()
}

#[test]
fn cmd_r_reloads_and_does_not_repeat() {
    assert_eq!(
        key_action(KeyBind::Char('r'), true, false, false),
        Some(Action::Reload)
    );
    assert_eq!(
        key_action(KeyBind::Char('R'), false, false, true),
        Some(Action::Reload)
    );
    assert_eq!(key_action(KeyBind::Char('r'), false, false, false), None);
    assert_eq!(
        key_action(KeyBind::Char('r'), true, true, false),
        None,
        "Ctrl+Shift+R stays export's neighbor, not reload"
    );
    assert!(!accept_key(true, Action::Reload));
}

#[test]
fn reload_icon_sits_left_of_the_zoom_cluster() {
    let w = 1280u32;
    let y = 28.0;
    let mut xs = Vec::new();
    for x in 0..w {
        if reload_hit(w, 80, EXPORT_ACTION_LABEL, "100%", x as f64 + 0.5, y) {
            xs.push(x);
        }
    }
    assert!(!xs.is_empty(), "toolbar must have a reload target");
    assert!(xs[0] > 600, "reload stays with zoom, not beside Open");
}

#[test]
fn reload_keeps_zoom_scroll_and_formats() {
    let mut session = Session::empty();
    session.load(&invoice_bytes()).unwrap();
    assert_eq!(session.app().unwrap().title(), "STATEMENT");
    session.set_window_size(480, 320);
    session.set_zoom(1.5);
    session.scroll_by(240.0);
    let scroll = session.scroll_y();
    assert!(scroll > 0.0, "invoice must scroll inside a short window");
    session
        .app_mut()
        .unwrap()
        .set_copy_format(CopyFormat::Plain);
    session
        .app_mut()
        .unwrap()
        .set_export_format(ExportFormat::Pdf);
    let page = session.app().unwrap().page();

    let next = retitled(&invoice_bytes(), "RELOADED");
    session.reload_bytes(&next).unwrap();

    let app = session.app().unwrap();
    assert_eq!(app.title(), "RELOADED");
    assert!((app.zoom() - 1.5).abs() < 1e-6);
    assert_eq!(app.page(), page);
    assert!((session.scroll_y() - scroll).abs() < 1.0);
    assert_eq!(app.copy_format(), CopyFormat::Plain);
    assert_eq!(app.export_format(), ExportFormat::Pdf);
    assert!(session.active_copy().is_none());
}

#[test]
fn failed_reload_keeps_the_open_invoice() {
    let mut session = Session::empty();
    session.load(&invoice_bytes()).unwrap();
    session.set_window_size(480, 320);
    session.scroll_by(180.0);
    let scroll = session.scroll_y();
    let title = session.app().unwrap().title().to_string();
    assert!(session.reload_bytes(&[0, 1, 2, 3, 4]).is_err());
    assert_eq!(session.app().unwrap().title(), title);
    assert!((session.scroll_y() - scroll).abs() < 1.0);
}

#[test]
fn reload_from_a_rewritten_invoice_file() {
    let dir = scratch("reload-file");
    let path = dir.join("invoice.K2F");
    let original = invoice_bytes();
    std::fs::write(&path, &original).unwrap();
    let mut session = Session::empty();
    session.load_path(&path).unwrap();
    session.set_zoom(1.25);
    std::fs::write(&path, retitled(&original, "FROM DISK")).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    session.reload_bytes(&bytes).unwrap();
    assert_eq!(session.app().unwrap().title(), "FROM DISK");
    assert!((session.app().unwrap().zoom() - 1.25).abs() < 1e-6);
}
