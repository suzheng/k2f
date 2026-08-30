use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn k2f() -> Command {
    Command::new(env!("CARGO_BIN_EXE_k2f"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn export_pdf_draws_invoice_lock_not_a_second_layout() {
    let dir = std::env::temp_dir().join(format!("k2f-pdf-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("invoice.pdf");
    let export = k2f()
        .args([
            "export-pdf",
            repo_root().join("examples/published/invoice.K2F").to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        export.status.success(),
        "{}",
        String::from_utf8_lossy(&export.stderr)
    );
    let pdf = fs::read(&out).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    assert!(
        !pdf.windows(16).any(|w| w == b"appearance_hash="),
        "default export must not add trust-pack captions"
    );
    // Page count vs lock is covered by k2f_pdf::tests::verify_page and export_opened tests.
}

#[test]
fn export_pdf_accepts_scale_4() {
    let dir = std::env::temp_dir().join(format!("k2f-pdf-4x-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("invoice-4x.pdf");
    let export = k2f()
        .args([
            "export-pdf",
            repo_root().join("examples/published/invoice.K2F").to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--scale",
            "4",
        ])
        .output()
        .unwrap();
    assert!(
        export.status.success(),
        "{}",
        String::from_utf8_lossy(&export.stderr)
    );
    assert!(fs::read(&out).unwrap().starts_with(b"%PDF-"));
}

#[test]
fn export_pdf_rejects_invalid_scale() {
    let dir = std::env::temp_dir().join(format!("k2f-pdf-bad-scale-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("out.pdf");
    let export = k2f()
        .args([
            "export-pdf",
            repo_root()
                .join("examples/published/invoice.K2F")
                .to_str()
                .unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--scale",
            "1.5",
        ])
        .output()
        .unwrap();
    assert!(!export.status.success());
}

#[test]
fn export_pdf_refuses_pdf_as_input() {
    let dir = std::env::temp_dir().join(format!("k2f-pdf-bad-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let src = dir.join("not.K2F");
    let out = dir.join("out.pdf");
    fs::write(&src, b"%PDF-1.7\n").unwrap();
    let export = k2f()
        .args([
            "export-pdf",
            src.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!export.status.success());
    let err = String::from_utf8_lossy(&export.stderr);
    assert!(
        err.contains("PDF_IS_NOT_A_SOURCE"),
        "got {err}"
    );
}
