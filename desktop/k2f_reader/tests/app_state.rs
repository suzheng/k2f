mod common;

use common::invoice_bytes;
use k2f_paint::Banner;
use k2f_reader::{AppState, VerifyStatus};

#[test]
fn opens_invoice_and_paints() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    assert_eq!(app.title(), "STATEMENT");
    assert!(matches!(app.banner(), Banner::Unsigned | Banner::Signed));
    assert_eq!(app.banner_str(), app.banner().as_str());
    assert_eq!(app.status(), VerifyStatus::Valid);
    assert_eq!(app.status_code(), app.banner().as_str());
    assert!(app.page_count() >= 1);
    let png = app.render_current_png().unwrap();
    assert!(png.starts_with(b"\x89PNG"));
    assert!(!app.text_layer().is_empty());
}

#[test]
fn export_pdf_magic() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let pdf = app.export_pdf_bytes().unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
}

#[test]
fn pdf_is_not_a_source() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let pdf = app.export_pdf_bytes().unwrap();
    let err = AppState::open(&pdf).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("PDF_IS_NOT_A_SOURCE"),
        "expected PDF_IS_NOT_A_SOURCE, got {msg}"
    );
}

#[test]
fn export_docx_magic() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let docx = app.export_docx_bytes().unwrap();
    assert!(docx.starts_with(b"PK"));
}

#[test]
fn docx_is_not_a_source() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let docx = app.export_docx_bytes().unwrap();
    let err = AppState::open(&docx).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("UNEXPECTED_PATH") || msg.contains("DOCX_IS_NOT_A_SOURCE"),
        "exported DOCX must not open as K2F, got {msg}"
    );
}

#[test]
fn export_pptx_magic() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let pptx = app.export_pptx_bytes().unwrap();
    assert!(pptx.starts_with(b"PK"));
}

#[test]
fn pptx_is_not_a_source() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let pptx = app.export_pptx_bytes().unwrap();
    let err = AppState::open(&pptx).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("PPTX_IS_NOT_A_SOURCE") || msg.contains("UNEXPECTED_PATH"),
        "exported PPTX must not open as K2F, got {msg}"
    );
}

#[test]
fn rejects_non_package_bytes() {
    let err = AppState::open(b"not-a-k2f-package").unwrap_err();
    let msg = format!("{err}");
    assert!(
        !msg.is_empty(),
        "garbage input must fail open, got empty error"
    );
}

#[test]
fn pager_and_zoom_clamp() {
    let mut app = AppState::open(&invoice_bytes()).unwrap();
    assert_eq!(app.page(), 0);
    assert_eq!(app.zoom(), 1.0);

    app.next_page();
    assert_eq!(app.page(), 0, "single-page invoice must not wrap");
    app.set_page(99);
    assert_eq!(app.page(), 0);
    app.prev_page();
    assert_eq!(app.page(), 0);

    app.set_zoom(0.1);
    assert_eq!(app.zoom(), 0.1);
    app.set_zoom(9.0);
    assert_eq!(app.zoom(), 3.0);
    app.set_zoom(1.5);
    assert_eq!(app.zoom(), 1.5);

    app.set_zoom(f32::NAN);
    assert_eq!(
        app.zoom(),
        1.5,
        "non-finite zoom must not poison session state"
    );
    app.set_zoom(f32::INFINITY);
    assert_eq!(app.zoom(), 3.0);
    app.set_zoom(f32::NEG_INFINITY);
    assert_eq!(app.zoom(), 0.1);
}

#[test]
fn zoom_does_not_change_official_raster() {
    let mut app = AppState::open(&invoice_bytes()).unwrap();
    let at_default = app.render_current_png().unwrap();
    app.set_zoom(0.5);
    let at_min = app.render_current_png().unwrap();
    app.set_zoom(3.0);
    let at_max = app.render_current_png().unwrap();
    assert_eq!(at_default, at_min);
    assert_eq!(at_default, at_max);
}
