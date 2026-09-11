//! GUI export (Ctrl+Shift+S / HUD). Dialogs are not opened; writes go to temp paths.

mod common;

use common::{
    invoice_bytes, pack_with_tampered_lock, published_invoice_bytes, scratch, unlocked_bytes,
};
use k2f_reader::export::ExportFormat;
use k2f_reader::ui::{
    accept_key, copy_hit, default_pdf_path, ensure_pdf_path, export_hit, export_label_x,
    export_menu_hit, export_menu_item_hit, key_action, overlay_label, pdf_file_name, Action, KeyBind,
    Session, COPY_ALL_TOOLTIP, EXPORT_ACTION_LABEL, HUD_HEIGHT,
};
use k2f_reader::AppState;
use std::path::{Path, PathBuf};

const EXPORT_BG: u32 = 0x007AFF;
const EXPORT_FG: u32 = 0xFFFFFF;

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
    assert_eq!(EXPORT_ACTION_LABEL, "Export as K2F");
    let x0 = f64::from(export_label_x(w));
    assert!(
        export_hit(w, h, x0 + 8.0, 24.0),
        "right-side Export as button matches the web Export control"
    );
    assert!(
        !export_hit(w, h, 12.0, 24.0),
        "left title is not the export control"
    );
    assert!(
        !export_hit(w, h, x0 + 8.0, f64::from(HUD_HEIGHT) + 2.0),
        "page pixels must not trigger export"
    );
    assert!(
        !export_hit(w, 10, x0 + 8.0, 12.0),
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
    let mut hits = 0u32;
    for row in 6..42 {
        for col in 2..20 {
            let p = frame[(row * w + x0 + col) as usize];
            if p == EXPORT_BG || p == EXPORT_FG {
                hits += 1;
            }
        }
    }
    assert!(
        hits > 0,
        "Export must be painted in the toolbar, not only hit-tested"
    );
}

#[test]
fn copy_hit_is_icon_only_left_of_export() {
    assert_eq!(COPY_ALL_TOOLTIP, "Copy all as Markdown");
    let w = 1280u32;
    let h = 100u32;
    let export_x = export_label_x(w) as f64;
    assert!(
        copy_hit(w, h, EXPORT_ACTION_LABEL, export_x - 20.0, 20.0),
        "copy icon sits just left of export"
    );
    assert!(
        !copy_hit(w, h, EXPORT_ACTION_LABEL, export_x + 10.0, 20.0),
        "export rect is not the copy control"
    );
}

#[test]
fn hud_export_does_not_overlap_banner_on_narrow_window() {
    let bytes = pack_with_tampered_lock(&published_invoice_bytes(), |lock| {
        lock.engine_version = "9.9.9".into();
    });
    let session = Session::new(AppState::open(&bytes).unwrap()).unwrap();
    let overlay = overlay_label(session.app().unwrap());
    assert!(
        overlay.len() * 8 + 16 > 320,
        "fixture must be long enough to collide without clipping, got {overlay:?}"
    );
    let w = 320u32;
    let h = HUD_HEIGHT + 40;
    let frame = session.compose_frame(w, h);
    let x0 = export_label_x(w);
    assert_eq!(
        frame[(24 * w + x0 + 2) as usize],
        EXPORT_BG,
        "title/banner chrome must clip before the Export button"
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
    assert_eq!(ExportFormat::Pptx.action_label(), "Export as PowerPoint");
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
    assert_eq!(ExportFormat::Docx.action_label(), "Export as Word");
    assert_eq!(ExportFormat::Docx.dialog_filter(), ("Word", &["docx"][..]));
}

#[test]
fn export_action_labels_match_web_menu() {
    let labels: Vec<&str> = ExportFormat::ALL.iter().map(|f| f.action_label()).collect();
    assert_eq!(
        labels,
        [
            "Export as K2F",
            "Export as PDF",
            "Export as PowerPoint",
            "Export as Word",
            "Export as Markdown",
            "Export as PNG",
            "Export as JPG",
        ]
    );
}

#[test]
fn export_caret_opens_export_as_menu() {
    let w = 800u32;
    let h = HUD_HEIGHT;
    let x0 = f64::from(export_label_x(w));
    assert!(
        export_hit(w, h, x0 + 8.0, 24.0),
        "main control exports the last format"
    );
    assert!(
        !export_menu_hit(w, h, x0 + 8.0, 24.0),
        "the label is not the caret"
    );
    assert!(
        export_menu_hit(w, h, f64::from(w) - 24.0, 24.0),
        "caret sits on the right of the split button"
    );
    assert!(
        export_menu_item_hit(w, 0, f64::from(w) - 48.0, 70.0),
        "first menu row is Export as K2F, click exports immediately"
    );
    assert!(
        !export_menu_item_hit(w, 0, f64::from(w) - 48.0, 24.0),
        "toolbar clicks are not menu rows"
    );
}

#[test]
fn session_export_menu_paints_over_the_page() {
    let mut session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    assert!(!session.export_menu_open());
    session.toggle_export_menu();
    assert!(session.export_menu_open());
    let (w, h) = session.scaled_size();
    session.set_window_size(w, h);
    let frame = session.compose_frame(w, h);
    let mut white = 0u32;
    for row in 60..78 {
        for col in (w.saturating_sub(240))..w.saturating_sub(24) {
            let p = frame[(row * w + col) as usize];
            if p == 0xFFFFFF {
                white += 1;
            }
        }
    }
    assert!(
        white > 100,
        "open Export as menu must paint a light panel, got {white} white pixels"
    );
    session.close_export_menu();
    let closed = session.compose_frame(w, h);
    let mut white_closed = 0u32;
    for row in 60..78 {
        for col in (w.saturating_sub(240))..w.saturating_sub(24) {
            if closed[(row * w + col) as usize] == 0xFFFFFF {
                white_closed += 1;
            }
        }
    }
    assert!(
        white_closed < white / 4,
        "closing the menu must drop the panel, open={white} closed={white_closed}"
    );
}

fn is_primary_blue(p: u32) -> bool {
    let r = (p >> 16) & 0xff;
    let g = (p >> 8) & 0xff;
    let b = p & 0xff;
    r < 32 && g > 64 && g < 180 && b > 200
}

#[test]
fn export_menu_check_icon_paints_primary_blue() {
    let mut session = Session::new(AppState::open(&invoice_bytes()).unwrap()).unwrap();
    session.toggle_export_menu();
    let (w, h) = session.scaled_size();
    session.set_window_size(w, h);
    let frame = session.compose_frame(w, h);
    let mut blue = 0u32;
    for row in 62..100 {
        for col in (w.saturating_sub(260))..w.saturating_sub(8) {
            let p = frame[(row * w + col) as usize];
            if is_primary_blue(p) {
                blue += 1;
            }
        }
    }
    assert!(
        blue >= 4,
        "selected export row must paint a vector check in PRIMARY blue, got {blue} pixels"
    );
}
