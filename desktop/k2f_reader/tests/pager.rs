mod common;

use common::published_invoice_bytes;
use k2f_reader::AppState;

#[test]
fn published_invoice_pages_and_paints_each_lock_page() {
    let mut app = AppState::open(&published_invoice_bytes()).unwrap();
    assert!(
        app.page_count() >= 2,
        "published invoice must paginate so pager can change rasters"
    );
    assert_eq!(app.page(), 0);

    let page0 = app.render_current_png().unwrap();
    assert!(page0.starts_with(b"\x89PNG"));
    let layer0 = app.text_layer();
    assert!(!layer0.is_empty());

    app.next_page();
    assert_eq!(app.page(), 1);
    let page1 = app.render_current_png().unwrap();
    assert!(page1.starts_with(b"\x89PNG"));
    assert_ne!(
        page0, page1,
        "render_current_png must follow the current page, not always page 0"
    );

    app.prev_page();
    assert_eq!(app.page(), 0);
    assert_eq!(app.render_current_png().unwrap(), page0);

    app.set_page(1);
    assert_eq!(app.page(), 1);
    assert_eq!(app.render_current_png().unwrap(), page1);

    app.set_page(app.page_count());
    assert_eq!(app.page(), 1, "set_page must ignore out-of-range indices");
}
