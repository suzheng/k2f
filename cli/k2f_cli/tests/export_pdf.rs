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
            repo_root()
                .join("examples/published/invoice.K2F")
                .to_str()
                .unwrap(),
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
    assert!(
        pdf.windows(14).any(|w| w == b"K2F PDF bridge"),
        "default export must embed Producer K2F PDF bridge"
    );
    assert!(
        !pdf.windows(9).any(|w| w == b"/AcroForm"),
        "invoice has no form fields so default PDF must not write /AcroForm"
    );
    let checker = repo_root().join("skills/k2f/scripts/check-pdf.py");
    let check = Command::new("python3")
        .args([checker.to_str().unwrap(), out.to_str().unwrap()])
        .output()
        .expect("python3 to run check-pdf.py");
    assert!(
        check.status.success(),
        "check-pdf.py must accept a clean default export: {}",
        String::from_utf8_lossy(&check.stderr)
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
            repo_root()
                .join("examples/published/invoice.K2F")
                .to_str()
                .unwrap(),
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
    assert!(err.contains("PDF_IS_NOT_A_SOURCE"), "got {err}");
}

fn pack_compiled_contract(dir: &std::path::Path) -> PathBuf {
    let pkg = dir.join("contract.K2F");
    let pack = k2f()
        .args([
            "pack",
            repo_root().join("examples/contract").to_str().unwrap(),
            "-o",
            pkg.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        pack.status.success(),
        "pack: {}",
        String::from_utf8_lossy(&pack.stderr)
    );
    let compile = k2f()
        .args(["compile", pkg.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "compile: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    pkg
}

#[test]
fn export_pdf_form_fields_default_writes_acroform() {
    let dir = std::env::temp_dir().join(format!("k2f-pdf-form-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let pkg = pack_compiled_contract(&dir);
    let out = dir.join("fillable.pdf");
    let export = k2f()
        .args([
            "export-pdf",
            pkg.to_str().unwrap(),
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
    assert!(
        pdf.windows(9).any(|w| w == b"/AcroForm"),
        "default export of a form document must write /AcroForm"
    );
}

#[test]
fn export_pdf_flatten_omits_acroform() {
    let dir = std::env::temp_dir().join(format!("k2f-pdf-flat-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let pkg = pack_compiled_contract(&dir);
    let out = dir.join("flat.pdf");
    let export = k2f()
        .args([
            "export-pdf",
            "--flatten",
            pkg.to_str().unwrap(),
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
        !pdf.windows(9).any(|w| w == b"/AcroForm"),
        "--flatten must not write /AcroForm"
    );
}

#[test]
fn export_pdf_trust_pack_form_document_errors_fillable_exclusive() {
    let dir = std::env::temp_dir().join(format!("k2f-pdf-trust-form-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let pkg = pack_compiled_contract(&dir);
    let out = dir.join("trust.pdf");
    let export = k2f()
        .args([
            "export-pdf",
            "--trust-pack",
            pkg.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!export.status.success());
    let err = String::from_utf8_lossy(&export.stderr);
    assert!(
        err.contains("FILLABLE_EXCLUSIVE"),
        "trust-pack + fillable fields must fail, got {err}"
    );
}

#[test]
fn export_pdf_help_lists_flatten() {
    let out = k2f().args(["export-pdf", "--help"]).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
    assert!(
        text.contains("flatten"),
        "export-pdf --help must list --flatten, got {text}"
    );
}
