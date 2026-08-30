mod common;

use common::{invoice_bytes, pack_with_tampered_lock, published_invoice_bytes};
use k2f_paint::TextSpan;
use k2f_reader::copy::{plain_text_from_spans, RectPt};
use k2f_reader::AppState;

fn span_bounds(spans: &[TextSpan]) -> RectPt {
    let mut x0 = f64::INFINITY;
    let mut y0 = f64::INFINITY;
    let mut x1 = f64::NEG_INFINITY;
    let mut y1 = f64::NEG_INFINITY;
    for s in spans {
        x0 = x0.min(s.x_pt);
        y0 = y0.min(s.y_pt);
        x1 = x1.max(s.x_pt + s.width_pt);
        y1 = y1.max(s.y_pt + s.height_pt);
    }
    RectPt { x0, y0, x1, y1 }
}

#[test]
fn copy_from_live_invoice_title() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let spans = app.text_layer();
    let title = spans
        .iter()
        .find(|s| s.text.contains("STATEMENT"))
        .expect("invoice heading is on the text layer");
    let sel = RectPt::from_drag(
        title.x_pt + title.width_pt * 0.25,
        title.y_pt + title.height_pt * 0.25,
        title.x_pt + title.width_pt * 0.75,
        title.y_pt + title.height_pt * 0.75,
    );
    let plain = plain_text_from_spans(&spans, sel);
    assert!(
        plain.contains("STATEMENT"),
        "drag over title box must copy lock text, got {plain:?}"
    );
    let payload = app
        .copy_selection(sel)
        .expect("drag over title must produce a clipboard payload");
    assert!(payload.plain.contains("STATEMENT"));
    assert!(
        payload.plain.trim_start().starts_with("# "),
        "default copy format is Markdown heading, got {:?}",
        payload.plain
    );
    assert!(
        payload.nodes.iter().any(|n| n.node_id == title.node_id),
        "JSON MIME must carry the lock node id, got {:?}",
        payload.nodes
    );
}

#[test]
fn copy_format_plain_keeps_span_text() {
    let mut app = AppState::open(&invoice_bytes()).unwrap();
    app.set_copy_format(k2f_reader::copy::CopyFormat::Plain);
    let spans = app.text_layer();
    let title = spans
        .iter()
        .find(|s| s.text.contains("STATEMENT"))
        .expect("invoice heading");
    let sel = RectPt::from_drag(
        title.x_pt + title.width_pt * 0.25,
        title.y_pt + title.height_pt * 0.25,
        title.x_pt + title.width_pt * 0.75,
        title.y_pt + title.height_pt * 0.75,
    );
    let payload = app.copy_selection(sel).expect("payload");
    assert!(!payload.plain.trim_start().starts_with("# "));
    assert!(payload.plain.contains("STATEMENT"));
}

#[test]
fn copy_table_cells_as_gfm_when_markdown() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let spans = app.text_layer();
    let cells: Vec<_> = spans
        .iter()
        .filter(|s| s.node_id.contains(".r") || s.node_id.contains(".h."))
        .take(4)
        .collect();
    assert!(cells.len() >= 2, "need table cell spans");
    let mut x0 = f64::INFINITY;
    let mut y0 = f64::INFINITY;
    let mut x1 = f64::NEG_INFINITY;
    let mut y1 = f64::NEG_INFINITY;
    for s in &cells {
        x0 = x0.min(s.x_pt);
        y0 = y0.min(s.y_pt);
        x1 = x1.max(s.x_pt + s.width_pt);
        y1 = y1.max(s.y_pt + s.height_pt);
    }
    let payload = app
        .copy_selection(RectPt { x0, y0, x1, y1 })
        .expect("table selection");
    assert!(
        payload.plain.contains('|') && payload.plain.contains("---"),
        "markdown table expected, got {:?}",
        payload.plain
    );
}

#[test]
fn copy_from_published_lock_without_recompile() {
    let app = AppState::open(&published_invoice_bytes()).unwrap();
    let spans = app.text_layer();
    assert!(!spans.is_empty());
    let plain = plain_text_from_spans(&spans, span_bounds(&spans));
    assert!(
        plain.contains("STATEMENT"),
        "full-page drag on published lock must copy title, got {plain:?}"
    );
    assert!(
        plain.contains('\n'),
        "stacked invoice spans must separate lines, got {plain:?}"
    );
}

#[test]
fn engine_mismatch_copy_matches_published_lock() {
    let intact = AppState::open(&published_invoice_bytes()).unwrap();
    let sel = span_bounds(&intact.text_layer());
    let plain = plain_text_from_spans(&intact.text_layer(), sel);
    assert!(plain.contains("STATEMENT"));

    let broken = AppState::open(&pack_with_tampered_lock(
        &published_invoice_bytes(),
        |lock| {
            lock.engine_version = "9.9.9".into();
        },
    ))
    .unwrap();
    assert_eq!(
        plain_text_from_spans(&broken.text_layer(), sel),
        plain,
        "copy must read the published lock, not recompile"
    );
}

#[test]
fn copy_is_page_local_on_published_invoice() {
    let mut app = AppState::open(&published_invoice_bytes()).unwrap();
    assert!(app.page_count() >= 2);
    let page0 = app.text_layer();
    let plain0 = plain_text_from_spans(&page0, span_bounds(&page0));
    app.next_page();
    let page1 = app.text_layer();
    let plain1 = plain_text_from_spans(&page1, span_bounds(&page1));
    assert!(!page1.is_empty(), "page 1 must have a text layer");
    let unique = page1
        .iter()
        .find(|s| !s.text.trim().is_empty() && !page0.iter().any(|p| p.text == s.text));
    if let Some(unique) = unique {
        assert!(
            !plain0.contains(&unique.text),
            "page 0 drag must not copy page 1 text {:?}",
            unique.text
        );
        assert!(
            plain1.contains(&unique.text),
            "page 1 drag must copy its own lock text {:?}",
            unique.text
        );
    } else {
        assert_ne!(
            plain0, plain1,
            "different pages must not copy identical full-page text"
        );
    }
}
