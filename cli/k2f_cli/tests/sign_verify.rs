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
fn compile_and_verify_report_pages() {
    let dir = std::env::temp_dir().join(format!("k2f-pages-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let pkg = dir.join("contract.K2F");
    fs::copy(repo_root().join("examples/published/contract.K2F"), &pkg).unwrap();

    let compile = k2f()
        .args(["compile", pkg.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let compile_err = String::from_utf8_lossy(&compile.stderr);
    assert!(
        compile_err.contains("pages="),
        "compile stderr missing pages=: {compile_err}"
    );

    let verify = k2f()
        .args(["verify", pkg.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        verify.status.success(),
        "{}",
        String::from_utf8_lossy(&verify.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&verify.stdout).trim(), "UNSIGNED");
    let verify_err = String::from_utf8_lossy(&verify.stderr);
    assert!(
        verify_err.contains("pages="),
        "verify stderr missing pages=: {verify_err}"
    );

    let png = dir.join("page0.png");
    let render = k2f()
        .args([
            "render",
            pkg.to_str().unwrap(),
            "--page",
            "0",
            "-o",
            png.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        render.status.success(),
        "{}",
        String::from_utf8_lossy(&render.stderr)
    );
    let render_err = String::from_utf8_lossy(&render.stderr);
    assert!(
        render_err.contains("rendering page 0 of "),
        "render stderr missing page count: {render_err}"
    );
}

#[test]
fn sign_and_verify_example_contract() {
    let dir = std::env::temp_dir().join(format!("k2f-sign-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let pkg = dir.join("contract.K2F");
    let key = dir.join("ed25519.hex");
    fs::copy(repo_root().join("examples/published/contract.K2F"), &pkg).unwrap();

    let compile = k2f()
        .args(["compile", pkg.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let keygen = k2f()
        .args(["keygen", "-o", key.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        keygen.status.success(),
        "{}",
        String::from_utf8_lossy(&keygen.stderr)
    );

    let unsigned = k2f()
        .args(["verify", pkg.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        unsigned.status.success(),
        "{}",
        String::from_utf8_lossy(&unsigned.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&unsigned.stdout).trim(), "UNSIGNED");

    let sign = k2f()
        .args([
            "sign",
            pkg.to_str().unwrap(),
            "--key",
            key.to_str().unwrap(),
            "--signed-by",
            "Acme Legal",
            "--signed-at",
            "1704067200",
        ])
        .output()
        .unwrap();
    assert!(
        sign.status.success(),
        "{}",
        String::from_utf8_lossy(&sign.stderr)
    );

    let signed = k2f()
        .args(["verify", pkg.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        signed.status.success(),
        "{}",
        String::from_utf8_lossy(&signed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&signed.stdout).trim(), "SIGNED");
    let err = String::from_utf8_lossy(&signed.stderr);
    assert!(err.contains("signed_by=Acme Legal"), "{err}");
    assert!(err.contains("signed_at=1704067200"), "{err}");
}

#[test]
fn hit_test_invoice_total_center() {
    let pkg = repo_root().join("examples/published/invoice.K2F");
    let doc = k2f_paint::OpenedDocument::open(&fs::read(&pkg).unwrap()).unwrap();
    let b = &doc.boxes_for("invoice.total")[0];
    let x = (b.x + b.width / 2) as f64 / 1000.0;
    let y = (b.y + b.height / 2) as f64 / 1000.0;
    let out = k2f()
        .args([
            "hit-test",
            pkg.to_str().unwrap(),
            "--page",
            &b.page.to_string(),
            "--x",
            &x.to_string(),
            "--y",
            &y.to_string(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.split_whitespace().next() == Some("invoice.total"),
        "got {stdout}"
    );
}

#[test]
fn hit_test_golden_invoice_pts() {
    let pkg = repo_root().join("examples/published/invoice.K2F");
    let out = k2f()
        .args([
            "hit-test",
            pkg.to_str().unwrap(),
            "--page",
            "2",
            "--x",
            "297.5",
            "--y",
            "258.2",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.split_whitespace().next(),
        Some("invoice.total"),
        "got {stdout}"
    );
}
