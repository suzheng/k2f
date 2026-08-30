use std::path::Path;

use super::repo_root;

pub fn read_repo(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| {
        panic!("missing {rel}: {e}");
    })
}

/// `(label, href)` pairs from `[label](href)` markdown.
pub fn markdown_links(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(mid) = rest.find("](") {
        let before = &rest[..mid];
        let label = before.rsplit('[').next().unwrap_or("").to_string();
        rest = &rest[mid + 2..];
        let Some(end) = rest.find(')') else { break };
        out.push((label, rest[..end].to_string()));
        rest = &rest[end + 1..];
    }
    out
}

pub fn markdown_section<'a>(text: &'a str, heading: &str) -> Option<&'a str> {
    let start = text.find(heading)?;
    let after = &text[start + heading.len()..];
    Some(after.split("\n## ").next().unwrap_or(after))
}

pub fn resolves(from: &str, link: &str) -> bool {
    let href = link.split('#').next().unwrap_or(link);
    if href.is_empty()
        || href.starts_with("http://")
        || href.starts_with("https://")
        || href.starts_with("mailto:")
    {
        return true;
    }
    repo_root().join(from).parent().unwrap().join(href).exists()
}

pub fn desktop_readme_links(from: &str, text: &str) -> Vec<(String, String)> {
    let hits: Vec<_> = markdown_links(text)
        .into_iter()
        .filter(|(_, href)| href.contains("desktop/k2f_reader"))
        .collect();
    for (label, href) in &hits {
        assert!(
            href.contains("README.md") || href.ends_with("k2f_reader/"),
            "{from}: desktop link should be the crate README, got [{label}]({href})"
        );
        assert!(
            !href.contains("docs/plans"),
            "{from}: public docs must not link internal plans"
        );
        assert!(
            resolves(from, href),
            "broken public link in {from} -> {href}"
        );
    }
    hits
}

pub fn assert_desktop_readme_linked(from: &str, text: &str) {
    let hits = desktop_readme_links(from, text);
    assert!(
        !hits.is_empty(),
        "{from} must link to desktop/k2f_reader/README.md"
    );
}

pub fn assert_not_packaged_binary(from: &str, text: &str) {
    let lower = text.to_ascii_lowercase();
    for claim in [
        "notarized",
        "signed dmg",
        "signed msi",
        "appimage",
        "github releases",
    ] {
        assert!(
            !lower.contains(claim),
            "{from} must not claim a packaged desktop binary ({claim})"
        );
    }
    assert!(
        !text.contains("--verify examples/published"),
        "{from} must not gate on --verify of committed published locks"
    );
}

pub fn walk_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if !dir.is_dir() {
        return;
    }
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk_files(&path, out);
        } else {
            out.push(path);
        }
    }
}
