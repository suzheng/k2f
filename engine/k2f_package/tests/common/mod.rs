#![allow(dead_code)]

use k2f_core::Manifest;
use k2f_layout::compile_manifest;
use k2f_package::{unpack_bytes, Package};
use std::fs;
use std::path::PathBuf;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn load_font() -> Vec<u8> {
    fs::read(repo_root().join("assets/fonts/NotoSansSC-Regular.otf")).expect("NotoSansSC-Regular.otf")
}

pub fn packed_contract() -> Vec<u8> {
    fs::read(repo_root().join("examples/published/contract.K2F")).unwrap()
}

pub fn load_contract_engine() -> (Manifest, String) {
    let pkg = unpack_bytes(&packed_contract()).unwrap();
    (pkg.engine_manifest(), pkg.theme_json.clone())
}

pub fn compile_pkg(pkg: &mut Package) {
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
