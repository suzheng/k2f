//! Edit / Done and the node popover, on a published invoice and a real form.

mod common;

use k2f_paint::OpenedDocument;
use k2f_reader::ui::{ChromeHit, Session};
use k2f_reader::AppState;
use k2f_sdk::Editor;

fn hash_of(bytes: &[u8]) -> String {
    OpenedDocument::open(bytes)
        .unwrap()
        .content_hash()
        .unwrap()
        .to_string()
}

fn doc_hash(session: &Session) -> String {
    session
        .app()
        .unwrap()
        .doc()
        .content_hash()
        .unwrap()
        .to_string()
}

fn click(session: &mut Session, x: f64, y: f64) {
    session.pointer_down(x, y);
    session.pointer_up(x, y);
}

/// Published invoice on disk, which Save and relock replaces.
fn on_disk(name: &str) -> (Session, std::path::PathBuf) {
    let bytes = common::published_invoice_bytes();
    let path = common::write_k2f(&common::scratch(name), &bytes);
    let mut session = Session::new(AppState::open(&bytes).unwrap()).unwrap();
    session.set_source_path(Some(path.clone()));
    (session, path)
}

fn find_hit(session: &Session, want: ChromeHit) -> bool {
    let (w, _) = session.scaled_size();
    (0..w).any(|x| session.chrome_hit(x as f64 + 0.5, 28.0) == Some(want))
}

fn box_center(session: &mut Session, id: &str) -> (f64, f64) {
    let b = session
        .app()
        .unwrap()
        .doc()
        .boxes_for(id)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{id} has no box"));
    session.jump_to_page(b.page);
    let (w, h) = session.scaled_size();
    let view = session.page_view(w, h);
    let cx = (b.x as f64 + b.width as f64 / 2.0) / 1000.0;
    let cy = (b.y as f64 + b.height as f64 / 2.0) / 1000.0;
    view.pt_to_window(cx, cy)
}

#[test]
fn invoice_toolbar_has_edit_not_save() {
    let mut session =
        Session::new(AppState::open(&common::published_invoice_bytes()).unwrap()).unwrap();
    assert!(find_hit(&session, ChromeHit::Edit));
    assert!(!find_hit(&session, ChromeHit::Save));
    assert!(!session.is_editing());
    session.toggle_edit();
    assert!(session.is_editing());
    assert!(find_hit(&session, ChromeHit::Edit));
    assert!(!find_hit(&session, ChromeHit::Save));
}

#[test]
fn view_mode_click_does_not_open_popover() {
    let bytes = common::published_invoice_bytes();
    let mut session = Session::new(AppState::open(&bytes).unwrap()).unwrap();
    let (x, y) = box_center(&mut session, "invoice.total");
    click(&mut session, x, y);
    assert!(session.popover_id().is_none());
    assert_eq!(doc_hash(&session), hash_of(&bytes));
}

#[test]
fn text_field_click_places_caret_and_accepts_input() {
    let (mut session, _) = on_disk("edit-caret");
    session.toggle_edit();
    let (x, y) = box_center(&mut session, "invoice.total");
    click(&mut session, x, y);
    assert!(
        session.ime_cursor_area().is_some(),
        "the open text box must be a text input"
    );
    let (tx, ty) = session.popover_text_point().expect("text");
    click(&mut session, tx, ty);
    session.type_text("Q");
    let (sx, sy) = session.popover_save_point().expect("save and relock");
    click(&mut session, sx, sy);
    let text = session
        .app()
        .unwrap()
        .doc()
        .selection("invoice.total")
        .unwrap()
        .text
        .unwrap();
    assert!(text.starts_with('Q'), "{text}");
}

#[test]
fn edit_click_save_relocks_invoice_total() {
    let bytes = common::published_invoice_bytes();
    let before = hash_of(&bytes);
    let (mut session, path) = on_disk("edit-save");
    session.toggle_edit();
    let (x, y) = box_center(&mut session, "invoice.total");
    click(&mut session, x, y);
    assert_eq!(session.popover_id(), Some("invoice.total"));
    session.type_text("!");
    let (sx, sy) = session.popover_save_point().expect("save and relock");
    click(&mut session, sx, sy);
    assert!(session.popover_id().is_none());
    assert!(!session.is_editing());
    let doc = session.app().unwrap().doc();
    assert_ne!(doc.content_hash().unwrap(), before);
    let text = doc.selection("invoice.total").unwrap().text.unwrap();
    assert!(text.contains("7,047.00"), "{text}");
    assert!(text.ends_with('!'), "{text}");
    let reopened = AppState::open(&std::fs::read(&path).unwrap()).unwrap();
    let again = reopened
        .doc()
        .selection("invoice.total")
        .unwrap()
        .text
        .unwrap();
    assert!(again.ends_with('!'), "{again}");
    assert_ne!(reopened.doc().content_hash().unwrap(), before);
    let pkg = k2f_package::unpack_bytes(&std::fs::read(&path).unwrap()).unwrap();
    let files = pkg.to_file_map().unwrap();
    let content: String = files
        .iter()
        .filter(|(p, _)| p.starts_with("content/") && p.ends_with(".json"))
        .map(|(_, b)| String::from_utf8_lossy(b).into_owned())
        .collect();
    assert!(
        content.contains("7,047.00!"),
        "content JSON must carry the saved text, got {content}"
    );
}

#[test]
fn save_onto_a_directory_keeps_the_popover() {
    let bytes = common::published_invoice_bytes();
    let before = hash_of(&bytes);
    let dir = common::scratch("edit-dir").join("doc.K2F");
    std::fs::create_dir_all(&dir).unwrap();
    let mut session = Session::new(AppState::open(&bytes).unwrap()).unwrap();
    session.set_source_path(Some(dir.clone()));
    session.toggle_edit();
    let (x, y) = box_center(&mut session, "invoice.total");
    click(&mut session, x, y);
    session.type_text("!");
    let (sx, sy) = session.popover_save_point().expect("save and relock");
    click(&mut session, sx, sy);
    assert_eq!(session.popover_id(), Some("invoice.total"));
    assert!(session.edit_error().is_some());
    assert_eq!(doc_hash(&session), before);
    assert!(dir.is_dir());
    assert!(!dir.parent().unwrap().join(".doc.K2F.tmp").exists());
}

#[test]
fn done_discards_popover_without_relock() {
    let bytes = common::published_invoice_bytes();
    let before = hash_of(&bytes);
    let mut session = Session::new(AppState::open(&bytes).unwrap()).unwrap();
    session.toggle_edit();
    let (x, y) = box_center(&mut session, "invoice.total");
    click(&mut session, x, y);
    session.type_text("Z");
    session.toggle_edit();
    assert!(session.popover_id().is_none());
    assert!(!session.is_editing());
    assert_eq!(doc_hash(&session), before);
}

#[test]
fn copy_node_writes_clipboard_json() {
    let mut session =
        Session::new(AppState::open(&common::published_invoice_bytes()).unwrap()).unwrap();
    session.toggle_edit();
    let (x, y) = box_center(&mut session, "invoice.total");
    click(&mut session, x, y);
    assert!(
        session.popover_copy_point().is_none(),
        "copy stays off the popover"
    );
    session.copy_open_node().expect("clipboard path");
    let json = session.take_node_clipboard().expect("clipboard");
    assert!(json.contains("invoice.total"), "{json}");
    assert!(session.is_editing(), "copy does not relock or leave edit");
}

#[test]
fn cancel_without_edits_closes_popover() {
    let bytes = common::published_invoice_bytes();
    let before = hash_of(&bytes);
    let mut session = Session::new(AppState::open(&bytes).unwrap()).unwrap();
    session.toggle_edit();
    let (x, y) = box_center(&mut session, "invoice.total");
    click(&mut session, x, y);
    let (cx, cy) = session.popover_cancel_point().expect("cancel");
    click(&mut session, cx, cy);
    assert!(!session.discard_prompt_open());
    assert!(session.popover_id().is_none());
    assert!(session.is_editing());
    assert_eq!(doc_hash(&session), before);
}

#[test]
fn cancel_with_edits_asks_before_discard() {
    let bytes = common::published_invoice_bytes();
    let before = hash_of(&bytes);
    let (mut session, _) = on_disk("edit-cancel");
    session.toggle_edit();
    let (x, y) = box_center(&mut session, "invoice.total");
    click(&mut session, x, y);
    session.type_text("Z");
    let (cx, cy) = session.popover_cancel_point().expect("cancel");
    click(&mut session, cx, cy);
    assert!(session.discard_prompt_open());
    assert_eq!(session.popover_id(), Some("invoice.total"));
    let (kx, ky) = session.discard_keep_point().expect("keep editing");
    click(&mut session, kx, ky);
    assert!(!session.discard_prompt_open());
    assert_eq!(session.popover_id(), Some("invoice.total"));
    let (sx, sy) = session.popover_save_point().expect("save and relock");
    click(&mut session, sx, sy);
    let text = session
        .app()
        .unwrap()
        .doc()
        .selection("invoice.total")
        .unwrap()
        .text
        .unwrap();
    assert!(text.ends_with('Z'), "{text}");

    session.toggle_edit();
    let (x, y) = box_center(&mut session, "invoice.total");
    click(&mut session, x, y);
    session.type_text("Q");
    let (cx, cy) = session.popover_cancel_point().expect("cancel");
    click(&mut session, cx, cy);
    let (dx, dy) = session.discard_discard_point().expect("discard");
    click(&mut session, dx, dy);
    assert!(!session.discard_prompt_open());
    assert!(session.popover_id().is_none());
    assert!(session.is_editing());
    let kept = session
        .app()
        .unwrap()
        .doc()
        .selection("invoice.total")
        .unwrap()
        .text
        .unwrap();
    assert!(kept.ends_with('Z'), "{kept}");
    assert!(!kept.contains('Q'), "{kept}");
    assert_ne!(doc_hash(&session), before);
}

#[test]
fn form_field_click_focuses_overlay_not_popover() {
    let bytes = Editor::open_dir(&common::repo_root().join("examples/form_application"))
        .unwrap()
        .save_bytes()
        .unwrap();
    let mut session = Session::new(AppState::open(&bytes).unwrap()).unwrap();
    assert!(find_hit(&session, ChromeHit::Edit));
    assert!(!find_hit(&session, ChromeHit::Save));
    session.toggle_edit();
    assert!(session.is_filling());
    assert!(find_hit(&session, ChromeHit::Save));
    let (x, y) = box_center(&mut session, "app.name");
    click(&mut session, x, y);
    assert!(session.popover_id().is_none());
    assert!(session.fill_editing());
}

#[test]
fn text_box_drag_grows_and_wheel_scrolls_inside_it() {
    let mut session =
        Session::new(AppState::open(&common::published_invoice_bytes()).unwrap()).unwrap();
    session.toggle_edit();
    let (x, y) = box_center(&mut session, "invoice.total");
    click(&mut session, x, y);
    let (_, sy) = session.popover_save_point().expect("save");
    let (rx, ry) = session.popover_resize_point().expect("resize edge");
    session.pointer_down(rx, ry);
    session.pointer_move(rx, ry + 80.0);
    session.pointer_up(rx, ry + 80.0);
    let (_, sy2) = session.popover_save_point().expect("save");
    assert!(
        sy2 > sy + 40.0,
        "dragging the bottom edge grows the text box ({sy} -> {sy2})"
    );

    session.type_text("\n\n\n\n\n\nbelow");
    let page = session.scroll_y();
    let before = session.popover_text_scroll();
    assert!(
        before > 1.0,
        "a long value scrolls the caret into view, got {before}"
    );
    let (tx, ty) = session.popover_text_point().expect("text");
    assert!(session.scroll_text_field(tx, ty, 60.0));
    assert!(
        session.popover_text_scroll() < before,
        "wheel over the text box moves through the lines"
    );
    assert!(
        (session.scroll_y() - page).abs() < 0.1,
        "the page stays put while the text box scrolls"
    );
}
