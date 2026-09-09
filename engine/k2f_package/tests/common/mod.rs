#![allow(dead_code)]

use k2f_core::{LockFile, Manifest};
use k2f_layout::compile_manifest;
use k2f_package::{appearance_hash_for_lock, unpack_bytes, Package};
use std::fs;
use std::path::PathBuf;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn load_font() -> Vec<u8> {
    fs::read(repo_root().join("assets/fonts/NotoSansSC-Regular.otf"))
        .expect("NotoSansSC-Regular.otf")
}

pub fn packed_contract() -> Vec<u8> {
    fs::read(repo_root().join("examples/published/contract.K2F")).unwrap()
}

pub fn load_contract_engine() -> (Manifest, String) {
    let pkg = unpack_bytes(&packed_contract()).unwrap();
    (pkg.engine_manifest(), pkg.theme_json.clone())
}

pub fn compile_pkg(pkg: &mut Package) {
    k2f_package::apply_coverage_subset(&mut pkg.fonts).unwrap();
    let assets: std::collections::HashMap<String, Vec<u8>> =
        pkg.assets.clone().into_iter().collect();
    let lock = compile_manifest(
        pkg.engine_manifest(),
        &pkg.theme_json,
        &pkg.fonts,
        if assets.is_empty() {
            None
        } else {
            Some(&assets)
        },
    )
    .unwrap();
    pkg.set_lock(&lock).unwrap();
}

pub fn parse_lock(pkg: &Package) -> LockFile {
    serde_json::from_str(pkg.lock_json.as_ref().expect("lock")).expect("lock json")
}

/// Change lock engine identity and refresh `appearance_hash` so the package stays self-consistent.
pub fn rebind_lock_engine(pkg: &mut Package, f: impl FnOnce(&mut LockFile)) {
    let mut lock = parse_lock(pkg);
    f(&mut lock);
    lock.appearance_hash = appearance_hash_for_lock(pkg, &lock).expect("appearance hash");
    pkg.set_lock(&lock).expect("set_lock");
}
