mod common;

use common::{compile_pkg, repo_root};
use k2f_core::hash_manifest_semantic;
use k2f_package::{load_dir, pack_bytes, unpack_bytes};
use std::fs;

#[test]
fn contract_with_includes_loads_and_maps_paths() {
    let pkg = load_dir(&repo_root().join("examples/contract")).expect("load contract");
    assert_eq!(pkg.include_map.len(), 3);
    assert_eq!(
        pkg.include_map.get("contract.clauses_1_3").map(String::as_str),
        Some("content/clauses_1_3.json")
    );
    assert_eq!(
        pkg.include_map.get("contract.signatures").map(String::as_str),
        Some("content/signatures.json")
    );
}

#[test]
fn contract_include_pack_unpack_roundtrip() {
    let pkg = load_dir(&repo_root().join("examples/contract")).expect("load");
    let bytes = pack_bytes(&pkg).unwrap();
    let pkg2 = unpack_bytes(&bytes).unwrap();
    assert_eq!(pack_bytes(&pkg2).unwrap(), bytes);
    assert_eq!(pkg2.include_map, pkg.include_map);
}

#[test]
fn contract_include_hash_matches_expanded_tree() {
    let pkg = load_dir(&repo_root().join("examples/contract")).expect("load");
    let hash_with_includes = hash_manifest_semantic(&pkg.engine_manifest());

    let mut flat = pkg.clone();
    flat.include_map.clear();
    let hash_flat = hash_manifest_semantic(&flat.engine_manifest());
    assert_eq!(hash_with_includes, hash_flat);
}

#[test]
fn contract_include_compiles() {
    let mut pkg = load_dir(&repo_root().join("examples/contract")).expect("load");
    compile_pkg(&mut pkg);
    assert!(pkg.lock_json.is_some());
}

#[test]
fn rejects_unreferenced_content_file() {
    let base = std::env::temp_dir().join(format!("k2f_include_orphan_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(base.join("content")).unwrap();
    fs::create_dir_all(base.join("assets/fonts")).unwrap();

    let contract = repo_root().join("examples/contract");
    for name in [
        "manifest.json",
        "styles/theme.json",
        "changelog.json",
        "content/root.json",
        "content/clauses_1_3.json",
        "content/clauses_4_7.json",
        "content/signatures.json",
    ] {
        let src = contract.join(name);
        if src.is_file() {
            if let Some(parent) = base.join(name).parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::copy(&src, base.join(name)).unwrap();
        }
    }
    if contract.join("assets/fonts").is_dir() {
        for entry in fs::read_dir(contract.join("assets/fonts")).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name();
            fs::copy(
                entry.path(),
                base.join("assets/fonts").join(name),
            )
            .unwrap();
        }
    }
    fs::write(
        base.join("content/orphan.json"),
        r#"{"id":"orphan","role":"body","content":{"type":"text","value":"x"}}"#,
    )
    .unwrap();

    let err = load_dir(&base).unwrap_err();
    assert!(
        err.to_string().contains("UNEXPECTED_PATH"),
        "got {err}"
    );
    let _ = fs::remove_dir_all(&base);
}
