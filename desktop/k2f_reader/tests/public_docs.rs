//! Public docs must link the in-tree native reader and match shipped/preview claims.

mod common;

use common::docs::{
    assert_desktop_readme_linked, assert_not_packaged_binary, desktop_readme_links,
    markdown_section, read_repo,
};

#[test]
fn status_lists_k2f_reader_in_preview() {
    let status = read_repo("docs/guide/status.md");
    let preview = markdown_section(&status, "## Preview")
        .expect("status.md must have a Preview section");
    assert!(
        preview.contains("k2f_reader"),
        "k2f_reader belongs under Preview (packaged builds on /download)"
    );
    assert!(
        preview.to_ascii_lowercase().contains("native"),
        "Preview must describe the native reader"
    );
    assert!(
        preview.contains("cargo run -p k2f_reader --"),
        "Preview must show the real cargo run (clone, no PATH install)"
    );
}

#[test]
fn status_links_desktop_readme() {
    let status = read_repo("docs/guide/status.md");
    assert_desktop_readme_linked("docs/guide/status.md", &status);
}

#[test]
fn status_does_not_claim_signed_desktop_binary() {
    let status = read_repo("docs/guide/status.md");
    assert_not_packaged_binary("docs/guide/status.md", &status);
    let shipped =
        markdown_section(&status, "## Shipped in 0.3.x").expect("status.md must list shipped 0.3.x");
    assert!(
        !shipped.contains("k2f_reader") && !shipped.contains("k2f-reader"),
        "native reader is preview-quality installers, not listed under Shipped in 0.3.x body"
    );
    let roadmap = markdown_section(&status, "## Roadmap (not shipped)")
        .expect("status.md must keep a Roadmap (not shipped) section");
    assert!(
        !roadmap.contains("k2f_reader") && !roadmap.contains("k2f-reader"),
        "the source crate is Preview; Roadmap keeps signing / notarization"
    );
    assert!(
        roadmap.to_ascii_lowercase().contains("notarization")
            || roadmap.to_ascii_lowercase().contains("signing"),
        "desktop signing / notarization stay not shipped"
    );
}

#[test]
fn root_readme_links_desktop_reader() {
    let readme = read_repo("README.md");
    let hits = desktop_readme_links("README.md", &readme);
    assert!(
        !hits.is_empty(),
        "root README must link desktop/k2f_reader/README.md"
    );
    let lower = readme.to_ascii_lowercase();
    assert!(
        lower.contains("k2f.dev/download") || lower.contains("/download"),
        "root README must point readers at packaged desktop downloads"
    );
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
