//! Binary tests for `--export-pdf` / `--verify`. Must not open a window.

mod common;

use common::{
    assert_ok, invoice_bytes, pack_with_tampered_lock, published_invoice_bytes, run_reader,
    scratch, write_k2f,
};
use k2f_paint::Banner;
use k2f_reader::AppState;
use std::path::Path;

fn run(args: &[&str]) -> std::process::Output {
    run_reader(args)
}

#[test]
fn headless_export_pdf() {
    let dir = scratch("export-sdk");
    let k2f_bytes = invoice_bytes();
    let k2f = write_k2f(&dir, &k2f_bytes);
    let pdf = dir.join("out.pdf");
    let out = run(&["--export-pdf", pdf.to_str().unwrap(), k2f.to_str().unwrap()]);
    assert_ok(&out);
    assert!(
        out.stdout.is_empty(),
        "export-only stdout must stay empty for CI; got {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
    let bytes = std::fs::read(&pdf).unwrap();
    assert!(
        bytes.starts_with(b"%PDF-"),
        "export must write a PDF without opening a window"
    );
    let app = AppState::open(&k2f_bytes).unwrap();
    assert_eq!(
        bytes,
        app.export_pdf_bytes().unwrap(),
        "CLI must draw the same lock as AppState::export_pdf_bytes"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("wrote"), "{stderr}");
}

#[test]
fn headless_export_pdf_published_invoice() {
    let dir = scratch("export-published");
    let k2f = write_k2f(&dir, &published_invoice_bytes());
    let pdf = dir.join("invoice.pdf");
    let out = run(&["--export-pdf", pdf.to_str().unwrap(), k2f.to_str().unwrap()]);
    assert_ok(&out);
    let bytes = std::fs::read(&pdf).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
    assert!(
        !bytes.windows(16).any(|w| w == b"appearance_hash="),
        "default export must not add trust-pack captions"
    );
}

#[test]
fn headless_verify_prints_banner() {
    let dir = scratch("verify-ok");
    let k2f = write_k2f(&dir, &invoice_bytes());
    let out = run(&["--verify", k2f.to_str().unwrap()]);
    assert_ok(&out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.trim(),
        "UNSIGNED",
        "CI greps banner on stdout, got {stdout:?}"
    );
}

#[test]
fn headless_verify_published_invoice_matches_appstate() {
    let bytes = published_invoice_bytes();
    let app = AppState::open(&bytes).unwrap();
    let dir = scratch("verify-published");
    let k2f = write_k2f(&dir, &bytes);
    let out = run(&["--verify", k2f.to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.trim(),
        app.banner_str(),
        "CLI must report the same banner as AppState"
    );
    if matches!(app.banner(), Banner::Unsigned | Banner::Signed) {
        assert_ok(&out);
    } else {
        assert_eq!(out.status.code(), Some(1));
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(app.status_code()),
            "got {stderr:?}, want {}",
            app.status_code()
        );
    }
}

#[test]
fn headless_verify_exits_1_on_broken() {
    let dir = scratch("verify-broken");
    let tampered = pack_with_tampered_lock(&invoice_bytes(), |lock| {
        lock.engine_version = "9.9.9".into();
    });
    let k2f = write_k2f(&dir, &tampered);
    let out = run(&["--verify", k2f.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1), "broken must fail CI");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "BROKEN_INTEGRITY", "{stdout:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("ENGINE_MISMATCH"),
        "status_code on stderr under BROKEN_INTEGRITY, got {stderr:?}"
    );
}

#[test]
fn headless_verify_and_export_together_still_fails_ci() {
    let dir = scratch("both-broken");
    let tampered = pack_with_tampered_lock(&invoice_bytes(), |lock| {
        lock.engine_version = "9.9.9".into();
    });
    let k2f = write_k2f(&dir, &tampered);
    let pdf = dir.join("out.pdf");
    let out = run(&[
        "--verify",
        "--export-pdf",
        pdf.to_str().unwrap(),
        k2f.to_str().unwrap(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "--verify must still fail CI when --export-pdf is set; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "BROKEN_INTEGRITY"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("ENGINE_MISMATCH"), "{stderr}");
    assert!(
        std::fs::read(&pdf).unwrap().starts_with(b"%PDF-"),
        "broken lock must still export for forensics"
    );
}

#[test]
fn headless_verify_and_export_together_ok() {
    let dir = scratch("both-ok");
    let k2f = write_k2f(&dir, &invoice_bytes());
    let pdf = dir.join("out.pdf");
    let out = run(&[
        k2f.to_str().unwrap(),
        "--export-pdf",
        pdf.to_str().unwrap(),
        "--verify",
    ]);
    assert_ok(&out);
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "UNSIGNED");
    assert!(std::fs::read(&pdf).unwrap().starts_with(b"%PDF-"));
}

#[test]
fn headless_export_pdf_still_draws_broken_lock() {
    let dir = scratch("export-broken");
    let tampered = pack_with_tampered_lock(&published_invoice_bytes(), |lock| {
        lock.engine_version = "9.9.9".into();
    });
    let k2f = write_k2f(&dir, &tampered);
    let pdf = dir.join("broken.pdf");
    let out = run(&["--export-pdf", pdf.to_str().unwrap(), k2f.to_str().unwrap()]);
    assert_ok(&out);
    assert!(std::fs::read(&pdf).unwrap().starts_with(b"%PDF-"));
}

#[test]
fn headless_rejects_pdf_as_source() {
    let dir = scratch("pdf-source");
    let fake = dir.join("not.K2F");
    std::fs::write(&fake, b"%PDF-1.7\n").unwrap();
    let pdf = dir.join("out.pdf");
    let export = run(&[
        "--export-pdf",
        pdf.to_str().unwrap(),
        fake.to_str().unwrap(),
    ]);
    assert!(!export.status.success());
    let err = String::from_utf8_lossy(&export.stderr);
    assert!(err.contains("PDF_IS_NOT_A_SOURCE"), "got {err}");
    assert!(
        !pdf.exists(),
        "must not write a PDF when the input is a PDF"
    );

    let verify = run(&["--verify", fake.to_str().unwrap()]);
    assert!(!verify.status.success());
    let err = String::from_utf8_lossy(&verify.stderr);
    assert!(err.contains("PDF_IS_NOT_A_SOURCE"), "got {err}");
}

#[test]
fn headless_flags_require_file() {
    let verify = run(&["--verify"]);
    assert_eq!(
        verify.status.code(),
        Some(2),
        "clap requires FILE; must not no-op as success"
    );

    let export = run(&["--export-pdf", "/tmp/k2f-reader-missing.pdf"]);
    assert_eq!(export.status.code(), Some(2));
    assert!(
        !Path::new("/tmp/k2f-reader-missing.pdf").exists(),
        "must not write PDF without a source file"
    );
}
