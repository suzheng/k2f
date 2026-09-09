mod common;

use common::{compile_pkg, repo_root};
use k2f_package::paths::FORMAT_SCHEMA_FILES;
use k2f_package::{load_dir, pack_bytes, unpack_bytes, verify_package, Package, VerifyStatus};
use std::fs;

#[test]
fn committed_contract_k2f_verifies_valid() {
    let bytes = fs::read(repo_root().join("examples/published/contract.K2F")).unwrap();
    let pkg = unpack_bytes(&bytes).unwrap();
    assert!(pkg
        .fonts
        .contains_key("assets/fonts/NotoSansSC-Regular.otf"));
    assert!(pkg.assets.contains_key("assets/images/logo.png"));
    assert!(pkg.lock_json.is_some());
    assert_format_schemas_only(&pkg);
    let hash = verify_package(&pkg).unwrap();
    assert!(
        matches!(hash, VerifyStatus::Valid | VerifyStatus::EngineMismatch),
        "committed contract must be self-consistent, got {hash:?}"
    );
    let lock: k2f_core::LockFile = serde_json::from_str(pkg.lock_json.as_ref().unwrap()).unwrap();
    assert!(
        lock.geometry.pages.len() >= 2,
        "formal contract must paginate, got {}",
        lock.geometry.pages.len()
    );
    let has_image = lock.render_plan.pages.iter().any(|p| {
        p.ops
            .iter()
            .any(|op| matches!(op, k2f_core::PaintOp::DrawImage { .. }))
    });
    assert!(has_image, "contract lock must paint the logo");
}

#[test]
fn committed_invoice_k2f_verifies_valid() {
    let bytes = fs::read(repo_root().join("examples/published/invoice.K2F")).unwrap();
    let pkg = unpack_bytes(&bytes).unwrap();
    assert!(pkg.fonts.contains_key("assets/fonts/Roboto-Regular.ttf"));
    assert!(pkg.assets.contains_key("assets/data/invoice_data.json"));
    assert!(pkg.assets.contains_key("assets/images/logo.png"));
    assert!(pkg.lock_json.is_some());
    assert_format_schemas_only(&pkg);
    let hash = verify_package(&pkg).unwrap();
    assert!(
        matches!(hash, VerifyStatus::Valid | VerifyStatus::EngineMismatch),
        "committed invoice must be self-consistent, got {hash:?}"
    );
    let lock: k2f_core::LockFile = serde_json::from_str(pkg.lock_json.as_ref().unwrap()).unwrap();
    assert!(
        lock.geometry.pages.len() >= 2,
        "statement table should repeat across pages, got {}",
        lock.geometry.pages.len()
    );
}

#[test]
fn unpack_pack_committed_contract_is_byte_identical() {
    let bytes = fs::read(repo_root().join("examples/published/contract.K2F")).unwrap();
    let pkg = unpack_bytes(&bytes).unwrap();
    assert_eq!(pack_bytes(&pkg).unwrap(), bytes);
}

#[test]
fn unpack_pack_committed_invoice_is_byte_identical() {
    let bytes = fs::read(repo_root().join("examples/published/invoice.K2F")).unwrap();
    let pkg = unpack_bytes(&bytes).unwrap();
    assert_eq!(pack_bytes(&pkg).unwrap(), bytes);
}

#[test]
fn load_dir_packs_and_verifies_contract_source() {
    let mut pkg = load_dir(&repo_root().join("examples/contract")).expect("load examples/contract");
    assert!(!pkg.fonts.is_empty());
    assert_format_schemas_only(&pkg);
    let bytes = pack_bytes(&pkg).unwrap();
    assert_eq!(bytes, pack_bytes(&pkg).unwrap());
    compile_pkg(&mut pkg);
    assert_eq!(verify_package(&pkg).unwrap(), VerifyStatus::Valid);
}

#[test]
fn pack_rejects_agent_schema_in_published_contract() {
    let bytes = fs::read(repo_root().join("examples/published/contract.K2F")).unwrap();
    let mut pkg = unpack_bytes(&bytes).unwrap();
    pkg.schemas
        .insert("schema/agent_v0.schema.json".into(), "{}".into());
    let err = pack_bytes(&pkg).unwrap_err();
    assert!(err.to_string().contains("UNEXPECTED_PATH"), "got {err}");
}

#[test]
fn load_dir_packs_and_verifies_invoice_source() {
    let mut pkg = load_dir(&repo_root().join("examples/invoice")).expect("load examples/invoice");
    assert!(!pkg.fonts.is_empty());
    assert_format_schemas_only(&pkg);
    compile_pkg(&mut pkg);
    assert_eq!(verify_package(&pkg).unwrap(), VerifyStatus::Valid);
}

fn assert_format_schemas_only(pkg: &Package) {
    let mut names: Vec<_> = pkg.schemas.keys().cloned().collect();
    names.sort();
    let mut expected: Vec<_> = FORMAT_SCHEMA_FILES.iter().map(|s| s.to_string()).collect();
    expected.sort();
    assert_eq!(names, expected);
    assert!(!names.iter().any(|p| p.contains("agent")));
}

#[test]
fn compile_on_committed_contract_is_valid_for_this_engine() {
    let bytes = fs::read(repo_root().join("examples/published/contract.K2F")).unwrap();
    let mut pkg = unpack_bytes(&bytes).unwrap();
    let before = verify_package(&pkg).unwrap();
    assert!(
        before.is_self_consistent(),
        "published contract must be self-consistent, got {before:?}"
    );
    let lock_before = pkg.lock_json.clone().unwrap();
    compile_pkg(&mut pkg);
    if before == VerifyStatus::Valid {
        assert_eq!(
            pkg.lock_json.as_ref().unwrap(),
            &lock_before,
            "same-engine compile must not change a valid lock"
        );
    }
    assert_eq!(verify_package(&pkg).unwrap(), VerifyStatus::Valid);
    compile_pkg(&mut pkg);
    assert_eq!(verify_package(&pkg).unwrap(), VerifyStatus::Valid);
}
