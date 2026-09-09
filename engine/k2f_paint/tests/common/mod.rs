#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use k2f_core::LockFile;
use k2f_layout::LayoutEngine;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn contract_k2f_bytes() -> Vec<u8> {
    fs::read(repo_root().join("examples/published/contract.K2F"))
        .expect("examples/published/contract.K2F")
}

pub fn font_bytes() -> Vec<u8> {
    fs::read(repo_root().join("assets/fonts/Roboto-Regular.ttf")).expect("Roboto-Regular.ttf")
}

pub fn compile_case(name: &str) -> LockFile {
    let dir = repo_root().join("tests/fixtures/cases").join(name);
    let content = fs::read_to_string(dir.join("content.json")).unwrap();
    let theme = fs::read_to_string(dir.join("theme.json")).unwrap();
    let json = LayoutEngine::compile_chunk(&content, &theme, &font_bytes()).unwrap();
    serde_json::from_str(&json).unwrap()
}

pub fn case_images(name: &str) -> std::collections::BTreeMap<String, Vec<u8>> {
    let dir = repo_root()
        .join("tests/fixtures/cases")
        .join(name)
        .join("assets");
    let mut out = std::collections::BTreeMap::new();
    let Ok(_) = fs::read_dir(&dir) else {
        return out;
    };
    fn rec(
        dir: &std::path::Path,
        prefix: &str,
        out: &mut std::collections::BTreeMap<String, Vec<u8>>,
    ) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                rec(&path, &format!("{prefix}{name}/"), out);
            } else if path.is_file() {
                out.insert(format!("{prefix}{name}"), fs::read(&path).unwrap());
            }
        }
    }
    rec(&dir, "assets/", &mut out);
    out
}

pub fn assert_png_golden(rel: &str, bytes: &[u8]) {
    assert!(
        bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "paint output is not a PNG: {rel}"
    );
    let path = repo_root().join(rel);
    if std::env::var_os("UPDATE_PAINT_GOLDENS").is_some() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, bytes).unwrap();
        return;
    }
    if std::env::var_os("CHECK_PAINT_GOLDENS").is_none() {
        return;
    }
    let expected = fs::read(&path).unwrap_or_else(|_| {
        panic!(
            "missing paint golden {:?}; run with UPDATE_PAINT_GOLDENS=1",
            path
        )
    });
    if expected != bytes {
        panic!(
            "paint golden mismatch: {rel} (expected {} bytes, got {} bytes); re-run with UPDATE_PAINT_GOLDENS=1",
            expected.len(),
            bytes.len()
        );
    }
}
