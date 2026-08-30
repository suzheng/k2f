mod common;

use common::{invoice_bytes, pack_with_tampered_lock, published_invoice_bytes};
use k2f_reader::ui::{overlay_label, window_title};
use k2f_reader::AppState;

#[test]
fn window_title_is_banner_and_document_title() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let title = window_title(&app);
    assert!(
        title.starts_with("K2F Reader — "),
        "plan title prefix, got {title}"
    );
    assert!(title.contains(app.banner_str()), "{title}");
    assert!(title.contains(app.title()), "{title}");
}

#[test]
fn overlay_shows_banner_and_one_indexed_pager() {
    let mut app = AppState::open(&published_invoice_bytes()).unwrap();
    assert!(app.page_count() >= 2);
    let label = overlay_label(&app);
    assert!(label.contains(app.banner_str()), "{label}");
    assert!(label.contains("1 /"), "web pager is 1-indexed, got {label}");
    assert!(label.contains(&format!("{}", app.page_count())), "{label}");

    app.next_page();
    let next = overlay_label(&app);
    assert!(
        next.contains("2 /"),
        "Right-arrow page change must update the HUD, got {next}"
    );
}

#[test]
fn broken_integrity_chrome_shows_status_code() {
    let app = AppState::open(&pack_with_tampered_lock(&invoice_bytes(), |lock| {
        lock.engine_version = "9.9.9".into();
    }))
    .unwrap();
    assert_eq!(app.banner_str(), "BROKEN_INTEGRITY");
    assert_eq!(app.status_code(), "ENGINE_MISMATCH");

    let title = window_title(&app);
    assert!(title.contains("BROKEN_INTEGRITY"), "{title}");
    assert!(
        title.contains("ENGINE_MISMATCH"),
        "web banner subtitle is status_code, got {title}"
    );

    let label = overlay_label(&app);
    assert!(label.contains("BROKEN_INTEGRITY"), "{label}");
    assert!(label.contains("ENGINE_MISMATCH"), "{label}");
}
