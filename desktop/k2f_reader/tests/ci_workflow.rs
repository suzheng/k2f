//! CI must run `cargo test -p k2f_reader` without a display or Save dialog.

mod common;

use common::{repo_root, LINUX_WINDOW_PACKAGES};

fn workflow() -> String {
    std::fs::read_to_string(repo_root().join(".github/workflows/golden-suite.yml"))
        .expect(".github/workflows/golden-suite.yml")
}

fn workspace_toml() -> String {
    std::fs::read_to_string(repo_root().join("Cargo.toml")).expect("workspace Cargo.toml")
}

#[test]
fn k2f_reader_is_a_member_but_not_default() {
    let toml = workspace_toml();
    assert!(
        toml.contains("\"desktop/k2f_reader\""),
        "workspace members must include desktop/k2f_reader"
    );
    let start = toml
        .find("default-members")
        .expect("workspace must declare default-members");
    let block = &toml[start..];
    let end = block.find(']').expect("default-members list");
    assert!(
        !block[..end].contains("desktop/k2f_reader"),
        "k2f_reader must stay out of default-members so root cargo test does not pull GUI crates"
    );
}

#[test]
fn golden_suite_runs_desktop_reader_tests() {
    let yaml = workflow();
    assert!(
        yaml.contains("desktop-reader:"),
        "GUI crate is not a default-member; it needs its own job"
    );
    assert!(
        yaml.contains("timeout-minutes:"),
        "job must time out if a test ever opens a blocking Save dialog"
    );
    assert!(
        yaml.contains("runs-on: ubuntu-latest"),
        "desktop-reader job must run on ubuntu-latest"
    );
    assert!(
        yaml.lines()
            .any(|l| l.trim() == "run: cargo test -p k2f_reader"),
        "the test step must be exactly `cargo test -p k2f_reader`"
    );
}

#[test]
fn ci_installs_linux_window_packages() {
    let yaml = workflow();
    for pkg in LINUX_WINDOW_PACKAGES {
        assert!(
            yaml.contains(pkg),
            "ubuntu-latest desktop-reader job must apt-get {pkg}"
        );
    }
}

#[test]
fn ci_does_not_open_gui_or_verify_published_invoice() {
    let yaml = workflow();
    assert!(
        !yaml.contains("--verify examples/published"),
        "do not add a one-off published --verify step; cargo test covers integrity"
    );
    assert!(
        !yaml.contains("pick_save_path"),
        "CI must never call pick_save_path (native Save dialog blocks)"
    );
    assert!(
        !yaml.contains("cargo run -p k2f_reader"),
        "CI must not launch the GUI via cargo run"
    );
    assert!(
        !yaml.contains("k2f-reader --"),
        "CI must use cargo test, not a GUI launch of k2f-reader"
    );
    assert!(
        !yaml.contains("xvfb"),
        "headless cargo test must not need xvfb / a display"
    );
}
