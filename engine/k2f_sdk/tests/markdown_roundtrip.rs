use k2f_core::{for_each_node, SemanticNode};
use k2f_package::unpack_bytes;
use k2f_sdk::{k2f_to_markdown, markdown_to_k2f, MarkdownOptions};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/markdown")
}

fn report_opts() -> MarkdownOptions {
    MarkdownOptions::new(
        "Document",
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates/report"),
    )
    .unwrap()
}

fn import(md: &str) -> k2f_sdk::MarkdownResult {
    markdown_to_k2f(md, report_opts()).unwrap()
}

fn root_of(bytes: &[u8]) -> SemanticNode {
    unpack_bytes(bytes).unwrap().root
}

fn texts(root: &SemanticNode, role: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for_each_node(root, &mut |n| {
        if n.role == role {
            if let k2f_core::NodeContent::Text(t) = &n.content {
                out.push((n.id.clone(), t.clone()));
            }
        }
    });
    out
}

fn tree_json(bytes: &[u8]) -> Value {
    serde_json::to_value(&unpack_bytes(bytes).unwrap().root).unwrap()
}

#[test]
fn import_heading_and_body() {
    let md = "# Hello\n\nWorld.";
    let result = import(md);
    let root = root_of(&result.bytes);
    let h = texts(&root, "h1");
    let p = texts(&root, "body");
    assert_eq!(h.len(), 1);
    assert!(h[0].0.starts_with("doc.h1_"));
    assert_eq!(h[0].1, "Hello");
    assert_eq!(p.len(), 1);
    assert!(p[0].0.starts_with("doc.p_"));
    assert_eq!(p[0].1, "World.");
}

#[test]
fn export_heading_body() {
    let md = "# Hi\n\nThere.";
    let bytes = import(md).bytes;
    let out = k2f_to_markdown(&bytes).unwrap();
    assert!(out.contains("# Hi"));
    assert!(out.contains("There."));
}

#[test]
fn default_theme_is_report() {
    let bytes = import("# T\n\nBody.").bytes;
    let pkg = unpack_bytes(&bytes).unwrap();
    let theme: Value = serde_json::from_str(&pkg.theme_json).unwrap();
    assert_eq!(theme["palette"]["accent"], "#0969da");
}

#[test]
fn footnote_and_html_warn() {
    let foot = fs::read_to_string(fixtures_dir().join("skip_footnote.md")).unwrap();
    let html = fs::read_to_string(fixtures_dir().join("skip_html.md")).unwrap();
    let f = import(&foot);
    let h = import(&html);
    assert!(
        f.report.warnings.iter().any(|w| w.contains("footnote")),
        "{:?}",
        f.report.warnings
    );
    assert!(
        h.report.warnings.iter().any(|w| w.contains("HTML")),
        "{:?}",
        h.report.warnings
    );
}

#[test]
fn markdown_roundtrip_fixtures() {
    let dir = fixtures_dir();
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("md"))
        .collect();
    files.sort();
    assert!(
        files.len() >= 20,
        "expected at least 20 markdown fixtures, got {}",
        files.len()
    );
    for path in &files {
        assert_roundtrip(path);
    }
}

fn assert_roundtrip(md_path: &Path) {
    let md = fs::read_to_string(md_path).unwrap();
    let opts = report_opts();
    let first = markdown_to_k2f(&md, opts.clone())
        .unwrap_or_else(|e| panic!("{}: import failed: {e}", md_path.display()));
    let out = k2f_to_markdown(&first.bytes)
        .unwrap_or_else(|e| panic!("{}: export failed: {e}", md_path.display()));
    let second = markdown_to_k2f(&out, opts)
        .unwrap_or_else(|e| panic!("{}: re-import failed: {e}", md_path.display()));
    if tree_json(&first.bytes) != tree_json(&second.bytes) {
        panic!(
            "{}: trees differ\n--- export ---\n{out}\n--- first ---\n{}\n--- second ---\n{}",
            md_path.display(),
            serde_json::to_string_pretty(&tree_json(&first.bytes)).unwrap(),
            serde_json::to_string_pretty(&tree_json(&second.bytes)).unwrap()
        );
    }
}
