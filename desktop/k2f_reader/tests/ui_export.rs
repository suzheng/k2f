//! GUI export (Ctrl+Shift+S / HUD). Dialogs are not opened; writes go to temp paths.

mod common;

use common::{
    invoice_bytes, pack_with_tampered_lock, published_invoice_bytes, scratch, unlocked_bytes,
};
use k2f_reader::export::ExportFormat;
use k2f_reader::ui::{
    accept_key, default_pdf_path, ensure_pdf_path, export_hit, export_label_x, key_action,
    overlay_label, pdf_file_name, Action, KeyBind, Session, EXPORT_ACTION_LABEL, HUD_HEIGHT,
};
use k2f_reader::AppState;
use std::path::{Path, PathBuf};

const HUD_FG: u32 = 0xF8FAFC;

#[test]
fn ctrl_shift_s_maps_to_export() {
    assert_eq!(
        key_action(KeyBind::Char('s'), true, true, false),
        Some(Action::Export)
    );
    assert_eq!(
        key_action(KeyBind::Char('S'), true, true, false),
        Some(Action::Export)
    );
    assert_eq!(
        key_action(KeyBind::Char('s'), false, true, true),
        Some(Action::Export),
        "Cmd+Shift+S on macOS"
    );
    assert_eq!(
        key_action(KeyBind::Char('s'), true, false, false),
        None,
        "Ctrl+S is not save — the reader is lock-only"
    );
    assert_eq!(key_action(KeyBind::Char('s'), false, true, false), None);
    assert!(
        !accept_key(true, Action::Export),
        "key-repeat must not spam the save dialog"
    );
}

#[test]
fn pdf_file_name_is_title_dot_pdf() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    assert_eq!(pdf_file_name(app.title()), "STATEMENT.pdf");
    assert_eq!(pdf_file_name("a/b:c"), "a_b_c.pdf");
    assert_eq!(pdf_file_name("   "), "document.pdf");
    assert_eq!(
        pdf_file_name("Invoice.pdf"),
        "Invoice.pdf",
        "dialog default must not become Invoice.pdf.pdf"
    );
    assert_eq!(pdf_file_name("Invoice.PDF"), "Invoice.pdf");
}

#[test]
fn default_pdf_path_is_title_next_to_source() {
    let src = Path::new("/tmp/docs/invoice.K2F");
    assert_eq!(
        default_pdf_path(src, "STATEMENT"),
        PathBuf::from("/tmp/docs/STATEMENT.pdf")
    );
}

#[test]
fn dialog_path_keeps_or_gains_pdf_extension() {
    assert_eq!(
        ensure_pdf_path(PathBuf::from("/tmp/STATEMENT")),
        PathBuf::from("/tmp/STATEMENT.pdf"),
        "rfd does not always append the filter extension"
    );
    assert_eq!(
        ensure_pdf_path(PathBuf::from("/tmp/STATEMENT.pdf")),
        PathBuf::from("/tmp/STATEMENT.pdf")
    );
    assert_eq!(
        ensure_pdf_path(PathBuf::from("/tmp/STATEMENT.PDF")),
        PathBuf::from("/tmp/STATEMENT.PDF")
    );
}

#[test]
fn export_pdf_to_writes_the_same_lock_bytes_as_headless() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let out = scratch("gui-export-sdk").join(pdf_file_name(app.title()));
    app.export_pdf_to(&out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert!(bytes.starts_with(b"%PDF-"), "GUI export must draw the lock");
    assert_eq!(
        bytes,
        app.export_pdf_bytes().unwrap(),
        "menu action must call the same export_pdf_bytes path as --export-pdf"
    );
}

#[test]
fn export_pdf_to_draws_published_invoice_lock() {
    let app = AppState::open(&published_invoice_bytes()).unwrap();
    let out = scratch("gui-export-published").join("invoice.pdf");
    app.export_pdf_to(&out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
    assert_eq!(bytes, app.export_pdf_bytes().unwrap());
    assert!(
        !bytes.windows(16).any(|w| w == b"appearance_hash="),
        "default export must not add trust-pack captions"
    );
}

#[test]
fn broken_integrity_still_exports_the_published_lock() {
    let app = AppState::open(&pack_with_tampered_lock(&invoice_bytes(), |lock| {
        lock.engine_version = "9.9.9".into();
    }))
    .unwrap();
    assert_eq!(app.banner_str(), "BROKEN_INTEGRITY");
    let out = scratch("gui-export-broken").join("broken.pdf");
    app.export_pdf_to(&out).unwrap();
    assert!(std::fs::read(&out).unwrap().starts_with(b"%PDF-"));
}

#[test]
fn hud_export_hit_is_the_right_edge() {
    let w = 800u32;
    let h = HUD_HEIGHT;
    assert_eq!(EXPORT_ACTION_LABEL, "Export");
    assert!(
        export_hit(w, h, f64::from(w) - 4.0, 8.0),
        "right-side HUD control matches the web Export button"
    );
    assert!(
        !export_hit(w, h, 12.0, 8.0),
        "left banner text is not the export control"
    );
    assert!(
        !export_hit(w, h, f64::from(w) - 4.0, f64::from(HUD_HEIGHT) + 2.0),
        "page pixels must not trigger export"
    );
    assert!(
        !export_hit(w, 10, f64::from(w) - 4.0, 12.0),
        "a short window clips the HUD; clicks below it are not Export"
    );
}

#[test]
fn session_apply_export_does_not_copy() {
    let mut session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    assert!(session.apply(Action::Export).is_none());
}

#[test]
fn export_markdown_writes_semantic_text() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let out = scratch("gui-export-md").join("invoice.md");
    app.export_to(ExportFormat::Markdown, &out).unwrap();
    let md = std::fs::read_to_string(&out).unwrap();
    assert!(md.contains('#'), "markdown export must include headings");
    assert!(!md.contains("<!--"), "export markdown must omit k2f hints");
}

#[test]
fn export_k2f_repackages_lock() {
    let bytes = invoice_bytes();
    let app = AppState::open(&bytes).unwrap();
    let out = scratch("gui-export-k2f").join("invoice.K2F");
    app.export_to(ExportFormat::K2f, &out).unwrap();
    let repacked = std::fs::read(&out).unwrap();
    assert!(repacked.starts_with(b"PK"));
    assert_eq!(repacked, app.export_k2f_bytes().unwrap());
}

#[test]
fn hud_paints_export_pdf_on_the_right() {
    let session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    let (w, h) = session.scaled_size();
    let frame = session.compose_frame(w, h);
    let x0 = export_label_x(w);
    let mut fg = 0u32;
    for row in 8..16 {
        for col in 0..8 {
            if frame[(row * w + x0 + col) as usize] == HUD_FG {
                fg += 1;
            }
        }
    }
    assert!(
        fg > 0,
        "Export must be painted in the HUD, not only hit-tested"
    );
}

#[test]
fn hud_export_does_not_overlap_banner_on_narrow_window() {
    // Broken integrity overlays include the status code and are long enough to
    // collide with Export on a narrow window; a short UNSIGNED label would not.
    let bytes = pack_with_tampered_lock(&published_invoice_bytes(), |lock| {
        lock.engine_version = "9.9.9".into();
    });
    let session = Session::new(AppState::open(&bytes).unwrap()).unwrap();
    let overlay = overlay_label(session.app());
    assert!(
        overlay.len() * 8 + 16 > 320,
        "fixture must be long enough to collide without clipping, got {overlay:?}"
    );
    let w = 320u32;
    let h = HUD_HEIGHT + 40;
    let frame = session.compose_frame(w, h);
    let gap_x = export_label_x(w).saturating_sub(4);
    assert_eq!(
        frame[(8 * w + gap_x) as usize],
        frame[0],
        "banner chrome must clip before Export on a resized window"
    );
}

#[test]
fn export_pdf_to_error_includes_path() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let path = scratch("gui-export-noparent")
        .join("missing")
        .join("out.pdf");
    let err = format!("{}", app.export_pdf_to(&path).unwrap_err());
    assert!(
        err.contains("out.pdf") || err.contains("missing"),
        "GUI stderr must show the path, got {err}"
    );
}

#[test]
fn export_pdf_to_unlocked_writes_nothing() {
    let app = AppState::open(&unlocked_bytes(&invoice_bytes())).unwrap();
    let out = scratch("gui-export-unlocked").join("out.pdf");
    let err = format!("{}", app.export_pdf_to(&out).unwrap_err());
    assert!(err.contains("UNLOCKED"), "got {err}");
    assert!(!out.exists(), "must not write a PDF without a lock");
}

#[test]
fn export_pptx_to_writes_zip() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let out = scratch("gui-export-pptx").join("invoice.pptx");
    app.export_to(ExportFormat::Pptx, &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert!(bytes.starts_with(b"PK"));
    assert_eq!(bytes, app.export_pptx_bytes().unwrap());
}

#[test]
fn export_pptx_to_unlocked_writes_nothing() {
    let app = AppState::open(&unlocked_bytes(&invoice_bytes())).unwrap();
    let out = scratch("gui-export-pptx-unlocked").join("out.pptx");
    let err = format!("{}", app.export_to(ExportFormat::Pptx, &out).unwrap_err());
    assert!(err.contains("UNLOCKED"), "got {err}");
    assert!(!out.exists(), "must not write a PPTX without a lock");
}

#[test]
fn export_format_cycle_includes_pptx() {
    assert!(ExportFormat::ALL.contains(&ExportFormat::Pptx));
    assert_eq!(ExportFormat::Pptx.extension(), "pptx");
    assert_eq!(ExportFormat::Pptx.hud_label(), "PPTX");
    assert_eq!(
        ExportFormat::Pdf.toggle(),
        ExportFormat::Pptx,
        "HUD format cycle must reach PPTX after PDF"
    );
    assert_eq!(
        ExportFormat::Pptx.toggle(),
        ExportFormat::Docx,
        "HUD format cycle must reach DOCX after PPTX"
    );
    assert_eq!(
        ExportFormat::Docx.toggle(),
        ExportFormat::Markdown,
        "HUD format cycle must continue after DOCX"
    );
}

#[test]
fn export_docx_to_writes_zip() {
    let app = AppState::open(&invoice_bytes()).unwrap();
    let out = scratch("gui-export-docx").join("invoice.docx");
    app.export_to(ExportFormat::Docx, &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert!(bytes.starts_with(b"PK"));
    assert_eq!(bytes, app.export_docx_bytes().unwrap());
}

#[test]
fn export_docx_to_unlocked_writes_nothing() {
    let app = AppState::open(&unlocked_bytes(&invoice_bytes())).unwrap();
    let out = scratch("gui-export-docx-unlocked").join("out.docx");
    let err = format!("{}", app.export_to(ExportFormat::Docx, &out).unwrap_err());
    assert!(err.contains("UNLOCKED"), "got {err}");
    assert!(!out.exists(), "must not write a DOCX without a lock");
}

#[test]
fn export_format_cycle_includes_docx() {
    assert!(ExportFormat::ALL.contains(&ExportFormat::Docx));
    assert_eq!(ExportFormat::Docx.extension(), "docx");
    assert_eq!(ExportFormat::Docx.hud_label(), "DOCX");
    assert_eq!(ExportFormat::Docx.dialog_filter(), ("Word", &["docx"][..]));
}
