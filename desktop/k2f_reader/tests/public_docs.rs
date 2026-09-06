//! Public docs must link the in-tree native reader without claiming a shipped binary.

mod common;

use common::docs::{
    assert_desktop_readme_linked, assert_not_packaged_binary, desktop_readme_links,
    markdown_section, read_repo,
};

#[test]
fn status_lists_k2f_reader_in_development() {
    let status = read_repo("docs/guide/status.md");
    let in_dev = markdown_section(&status, "## In development")
        .expect("status.md must have an In development section");
    assert!(
        in_dev.contains("k2f_reader"),
        "k2f_reader belongs under In development, not only Roadmap (not shipped)"
    );
    assert!(
        in_dev.to_ascii_lowercase().contains("native"),
        "In development must describe the native reader"
    );
    assert!(
        in_dev.contains("cargo run -p k2f_reader --"),
        "In development must show the real cargo run (clone, no PATH install)"
    );
}

#[test]
fn status_links_desktop_readme() {
    let status = read_repo("docs/guide/status.md");
    assert_desktop_readme_linked("docs/guide/status.md", &status);
}

#[test]
fn status_does_not_claim_a_shipped_desktop_binary() {
    let status = read_repo("docs/guide/status.md");
    assert_not_packaged_binary("docs/guide/status.md", &status);
    let shipped =
        markdown_section(&status, "## Shipped").expect("status.md must have a Shipped section");
    assert!(
        !shipped.contains("k2f_reader") && !shipped.contains("k2f-reader"),
        "native reader is in development, not shipped in v0.1"
    );
    let roadmap = markdown_section(&status, "## Roadmap")
        .expect("status.md must keep a Roadmap (not shipped) section");
    assert!(
        !roadmap.contains("k2f_reader") && !roadmap.contains("k2f-reader"),
        "the source crate is In development; Roadmap keeps file association / installers"
    );
    assert!(
        roadmap.to_ascii_lowercase().contains("file association")
            || roadmap.to_ascii_lowercase().contains("installer"),
        "packaged binaries / file association stay not shipped"
    );
}

#[test]
fn root_readme_links_desktop_reader() {
    let readme = read_repo("README.md");
    let hits = desktop_readme_links("README.md", &readme);
    assert!(
        !hits.is_empty(),
        "root README must add a Roadmap link to desktop/k2f_reader/README.md"
    );
    for (label, href) in &hits {
        let blob = format!("{label} {href}").to_ascii_lowercase();
        assert!(
            blob.contains("in development"),
            "desktop README link must be labeled in development, got [{label}]({href})"
        );
    }
    assert_not_packaged_binary("README.md", &readme);
}

#[test]
fn desktop_readme_exists() {
    assert!(
        common::repo_root()
            .join("desktop/k2f_reader/README.md")
            .is_file(),
        "public links target desktop/k2f_reader/README.md"
    );
}
