//! Getting Started, docs index, and the web viewer must point at the native reader.

mod common;

use common::docs::{
    assert_desktop_readme_linked, assert_not_packaged_binary, markdown_links, read_repo,
};

#[test]
fn getting_started_links_native_reader() {
    let text = read_repo("docs/guide/getting-started.md");
    assert_desktop_readme_linked("docs/guide/getting-started.md", &text);
    assert_not_packaged_binary("docs/guide/getting-started.md", &text);
    assert!(
        text.contains("cargo run -p k2f_reader --"),
        "getting-started must show the real cargo run (clone, no PATH install)"
    );
    assert!(
        text.contains("examples/published/invoice.K2F"),
        "getting-started must open the committed invoice, not a fake path"
    );
}

#[test]
fn docs_index_links_native_reader() {
    let text = read_repo("docs/README.md");
    assert_desktop_readme_linked("docs/README.md", &text);
    let hits = markdown_links(&text);
    let desktop = hits
        .iter()
        .find(|(_, href)| href.contains("desktop/k2f_reader"))
        .expect("desktop link");
    assert!(
        desktop.0.to_ascii_lowercase().contains("in development")
            || text.to_ascii_lowercase().contains("in development"),
        "docs index must present the native reader as in development"
    );
    assert_not_packaged_binary("docs/README.md", &text);
}

#[test]
fn web_viewer_readme_links_native_reader() {
    let text = read_repo("viewer/README.md");
    assert_desktop_readme_linked("viewer/README.md", &text);
    assert_not_packaged_binary("viewer/README.md", &text);
    let lower = text.to_ascii_lowercase();
    assert!(
        lower.contains("lock executor") || lower.contains("does not recompile"),
        "viewer README must say the native reader is the same lock-executor contract"
    );
}
