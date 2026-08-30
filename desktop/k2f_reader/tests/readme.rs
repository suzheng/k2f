//! README must document real lock-executor behavior — not a GUI smoke that CI cannot run.

mod common;

use common::LINUX_WINDOW_PACKAGES;
use std::path::PathBuf;

fn readme() -> String {
    std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("README.md"))
        .expect("desktop/k2f_reader/README.md")
}

#[test]
fn documents_lock_executor_rules() {
    let text = readme();
    for needle in [
        "does not recompile",
        "document.K2F.lock",
        "PDF export draws the lock",
        "PDF_IS_NOT_A_SOURCE",
        "not a default-member",
    ] {
        assert!(text.contains(needle), "README must state {needle:?}");
    }
}

#[test]
fn documents_build_run_and_headless() {
    let text = readme();
    for needle in [
        "cargo build -p k2f_reader --release",
        "target/release/k2f-reader",
        "cargo run -p k2f_reader --",
        "cargo run -p k2f_reader -- --verify",
        "cargo run -p k2f_reader -- --export-pdf",
        "cargo test -p k2f_reader",
        "examples/published/invoice.K2F",
    ] {
        assert!(text.contains(needle), "README must include `{needle}`");
    }
}

#[test]
fn documents_linux_window_packages() {
    let text = readme();
    for pkg in LINUX_WINDOW_PACKAGES {
        assert!(
            text.contains(pkg),
            "README Debian/Ubuntu list must include {pkg} (same as CI)"
        );
    }
}

#[test]
fn does_not_tell_ci_to_verify_published_invoice() {
    let text = readme();
    assert!(
        text.contains("ENGINE_MISMATCH"),
        "README must warn that a committed published lock can already be ENGINE_MISMATCH"
    );
    assert!(
        !text.contains("--verify examples/published/invoice.K2F"),
        "gating CI on --verify of the committed invoice fails after an engine bump"
    );
}
