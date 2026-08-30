//! Integrity / error paths for the headless binary. Must not open a window.

mod common;

use common::{
    assert_ok, invoice_bytes, run_reader, scratch, signed_bytes, unlocked_bytes, write_k2f,
};

#[test]
fn headless_verify_signed_exits_0() {
    let dir = scratch("verify-signed");
    let k2f = write_k2f(&dir, &signed_bytes(&invoice_bytes()));
    let out = run_reader(&["--verify", k2f.to_str().unwrap()]);
    assert_ok(&out);
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "SIGNED");
}

#[test]
fn headless_verify_unlocked_exits_1() {
    let dir = scratch("verify-unlocked");
    let k2f = write_k2f(&dir, &unlocked_bytes(&invoice_bytes()));
    let out = run_reader(&["--verify", k2f.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "UNLOCKED");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("UNLOCKED"), "{stderr}");
}

#[test]
fn headless_export_unlocked_writes_nothing() {
    let dir = scratch("export-unlocked");
    let k2f = write_k2f(&dir, &unlocked_bytes(&invoice_bytes()));
    let pdf = dir.join("out.pdf");
    let out = run_reader(&["--export-pdf", pdf.to_str().unwrap(), k2f.to_str().unwrap()]);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("UNLOCKED"), "got {err}");
    assert!(!pdf.exists(), "must not write a PDF without a lock");
}

#[test]
fn headless_missing_file_exits_nonzero() {
    let missing = scratch("missing").join("no-such.K2F");
    let verify = run_reader(&["--verify", missing.to_str().unwrap()]);
    assert!(!verify.status.success());
    assert_ne!(verify.status.code(), Some(0));
    let err = String::from_utf8_lossy(&verify.stderr);
    assert!(
        err.contains("no-such.K2F") || err.contains("No such file"),
        "CI must see the path, got {err}"
    );

    let pdf = scratch("missing-export").join("out.pdf");
    let export = run_reader(&[
        "--export-pdf",
        pdf.to_str().unwrap(),
        missing.to_str().unwrap(),
    ]);
    assert!(!export.status.success());
    assert!(!pdf.exists());
}
