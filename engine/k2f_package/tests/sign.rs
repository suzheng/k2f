mod common;

use common::{compile_pkg, repo_root};
use k2f_core::canonical_json_string;
use k2f_package::{
    generate_secret_key, inspect_package, pack_bytes, sign_package, unpack_bytes, verify_package,
    IntegrityStatus, SecretKey, VerifyStatus, CODE_SIGNED, CODE_SIGNED_BUT_BROKEN, CODE_UNSIGNED,
};
use serde_json::Value;
use std::fs;

fn compiled_contract() -> k2f_package::Package {
    let bytes = fs::read(repo_root().join("examples/published/contract.K2F")).unwrap();
    unpack_bytes(&bytes).unwrap()
}

fn tamper_clause_2(pkg: &mut k2f_package::Package) {
    let node = k2f_core::find_node_mut(&mut pkg.root, "contract.clause_2").unwrap();
    match &mut node.content {
        k2f_core::NodeContent::Text(t) => t.push_str(" 110"),
        other => panic!("expected contract.clause_2 text, got {other:?}"),
    }
}

fn sign_with_fixed_time(pkg: &mut k2f_package::Package, key: &SecretKey) {
    sign_package(pkg, key, Some("Acme Legal"), 1_704_067_200).unwrap();
}

#[test]
fn committed_contract_is_unsigned_not_signed() {
    let pkg = compiled_contract();
    let hash = verify_package(&pkg).unwrap();
    assert!(
        matches!(hash, VerifyStatus::Valid | VerifyStatus::EngineMismatch),
        "committed contract must be self-consistent, got {hash:?}"
    );
    let report = inspect_package(&pkg).unwrap();
    assert_eq!(report.status, IntegrityStatus::Unsigned);
    assert_eq!(report.status.code(), CODE_UNSIGNED);
    assert!(report.fingerprint.is_none());
}

#[test]
fn sign_lock_bytes_is_signed() {
    let mut pkg = compiled_contract();
    let key = generate_secret_key().unwrap();
    sign_with_fixed_time(&mut pkg, &key);
    let report = inspect_package(&pkg).unwrap();
    assert_eq!(report.status, IntegrityStatus::Signed);
    assert_eq!(report.status.code(), CODE_SIGNED);
    assert_eq!(report.signed_by.as_deref(), Some("Acme Legal"));
    assert_eq!(report.signed_at, Some(1_704_067_200));
    assert_eq!(report.fingerprint, Some(key.fingerprint()));
    assert!(pkg.signatures_json.is_some());
}

#[test]
fn same_key_and_timestamp_packs_identically() {
    let mut pkg = compiled_contract();
    let key = SecretKey::from_hex(&generate_secret_key().unwrap().to_hex()).unwrap();
    sign_with_fixed_time(&mut pkg, &key);
    let a = pack_bytes(&pkg).unwrap();
    let b = pack_bytes(&pkg).unwrap();
    assert_eq!(a, b);
    let round = unpack_bytes(&a).unwrap();
    assert_eq!(
        inspect_package(&round).unwrap().status,
        IntegrityStatus::Signed
    );
}

#[test]
fn changing_amount_without_resign_is_signed_but_broken() {
    let mut pkg = compiled_contract();
    let key = generate_secret_key().unwrap();
    sign_with_fixed_time(&mut pkg, &key);
    tamper_clause_2(&mut pkg);
    let report = inspect_package(&pkg).unwrap();
    assert_eq!(report.status, IntegrityStatus::SignedButBroken);
    assert_eq!(report.status.code(), CODE_SIGNED_BUT_BROKEN);
    assert_eq!(report.hash, VerifyStatus::ContentChanged);
}

#[test]
fn changing_theme_on_signed_file_drops_the_green_check() {
    let mut pkg = compiled_contract();
    let key = generate_secret_key().unwrap();
    sign_with_fixed_time(&mut pkg, &key);
    assert_eq!(
        inspect_package(&pkg).unwrap().status,
        IntegrityStatus::Signed
    );
    pkg.theme_json = pkg.theme_json.replace("#FFFFFF", "#000000");
    pkg.theme_json = pkg.theme_json.replace("#ffffff", "#000000");
    let report = inspect_package(&pkg).unwrap();
    assert_eq!(report.status, IntegrityStatus::SignedButBroken);
    assert_eq!(report.hash, VerifyStatus::AppearanceChanged);
}

#[test]
fn recompile_and_resign_makes_old_signature_invalid() {
    let mut pkg = compiled_contract();
    let key = generate_secret_key().unwrap();
    sign_with_fixed_time(&mut pkg, &key);
    let old_sig = pkg.signatures_json.clone().unwrap();

    tamper_clause_2(&mut pkg);
    compile_pkg(&mut pkg);
    assert_eq!(
        inspect_package(&pkg).unwrap().status,
        IntegrityStatus::SignedButBroken,
        "old signature must not green-check a new lock"
    );

    sign_package(&mut pkg, &key, Some("Acme Legal"), 1_704_067_201).unwrap();
    let report = inspect_package(&pkg).unwrap();
    assert_eq!(report.status, IntegrityStatus::Signed);
    assert_ne!(pkg.signatures_json.as_ref().unwrap(), &old_sig);

    pkg.signatures_json = Some(old_sig);
    assert_eq!(
        inspect_package(&pkg).unwrap().status,
        IntegrityStatus::SignedButBroken
    );
}

#[test]
fn signature_is_over_canonical_lock_bytes_not_pretty_json() {
    let mut pkg = compiled_contract();
    let key = generate_secret_key().unwrap();
    sign_with_fixed_time(&mut pkg, &key);
    let lock: Value = serde_json::from_str(pkg.lock_json.as_ref().unwrap()).unwrap();
    let pretty = serde_json::to_string_pretty(&lock).unwrap();
    assert_ne!(&pretty, pkg.lock_json.as_ref().unwrap());
    pkg.lock_json = Some(pretty);
    assert_eq!(
        inspect_package(&pkg).unwrap().status,
        IntegrityStatus::SignedButBroken,
        "pretty-printed lock is different bytes and must not verify"
    );

    pkg.lock_json = Some(canonical_json_string(&lock).unwrap());
    assert_eq!(
        inspect_package(&pkg).unwrap().status,
        IntegrityStatus::Signed,
        "re-canonicalized lock bytes must match the signature"
    );
}

#[test]
fn refuse_to_sign_unlocked_or_broken_package() {
    let mut pkg = compiled_contract();
    pkg.lock_json = None;
    let key = generate_secret_key().unwrap();
    let err = sign_package(&mut pkg, &key, None, 0).unwrap_err();
    assert!(err.to_string().contains("UNLOCKED"), "got {err}");

    let mut pkg = compiled_contract();
    pkg.theme_json = pkg.theme_json.replace("#FFFFFF", "#000000");
    pkg.theme_json = pkg.theme_json.replace("#ffffff", "#000000");
    let err = sign_package(&mut pkg, &key, None, 0).unwrap_err();
    assert!(
        err.to_string().contains("APPEARANCE_CHANGED") || err.to_string().contains("not VALID"),
        "got {err}"
    );
}

#[test]
fn generated_by_is_not_signed_by() {
    let mut pkg = compiled_contract();
    pkg.manifest.generated_by = Some("agent.invoice-bot".into());
    let key = generate_secret_key().unwrap();
    sign_package(&mut pkg, &key, Some("Jane Doe"), 1_704_067_200).unwrap();
    let report = inspect_package(&pkg).unwrap();
    assert_eq!(
        pkg.manifest.generated_by.as_deref(),
        Some("agent.invoice-bot")
    );
    assert_eq!(report.signed_by.as_deref(), Some("Jane Doe"));
    assert_eq!(report.generated_by.as_deref(), Some("agent.invoice-bot"));
    assert_ne!(report.signed_by, pkg.manifest.generated_by);
}

fn rewrite_signatures(pkg: &mut k2f_package::Package, f: impl FnOnce(&mut Value)) {
    let mut v: Value = serde_json::from_str(pkg.signatures_json.as_ref().unwrap()).unwrap();
    f(&mut v);
    pkg.signatures_json = Some(canonical_json_string(&v).unwrap());
}

#[test]
fn flipped_signature_byte_is_signed_but_broken_while_hashes_still_match() {
    let mut pkg = compiled_contract();
    let key = generate_secret_key().unwrap();
    sign_with_fixed_time(&mut pkg, &key);
    let hash = verify_package(&pkg).unwrap();
    assert!(hash.is_self_consistent(), "got {hash:?}");
    rewrite_signatures(&mut pkg, |v| {
        let sig = v["signatures"][0]["signature"]
            .as_str()
            .unwrap()
            .to_string();
        let mut chars: Vec<char> = sig.chars().collect();
        chars[0] = if chars[0] == '0' { '1' } else { '0' };
        v["signatures"][0]["signature"] = Value::String(chars.into_iter().collect());
    });
    let report = inspect_package(&pkg).unwrap();
    assert_eq!(report.status, IntegrityStatus::SignedButBroken);
    assert_eq!(report.hash, hash);
}

#[test]
fn signature_version_two_cannot_green_check() {
    let mut pkg = compiled_contract();
    let key = generate_secret_key().unwrap();
    sign_with_fixed_time(&mut pkg, &key);
    rewrite_signatures(&mut pkg, |v| {
        v["version"] = Value::from(2);
    });
    assert_eq!(
        inspect_package(&pkg).unwrap().status,
        IntegrityStatus::SignedButBroken
    );
}

#[test]
fn extra_field_on_signature_cannot_green_check() {
    let mut pkg = compiled_contract();
    let key = generate_secret_key().unwrap();
    sign_with_fixed_time(&mut pkg, &key);
    rewrite_signatures(&mut pkg, |v| {
        v["signatures"][0]["note"] = Value::String("x".into());
    });
    assert_eq!(
        inspect_package(&pkg).unwrap().status,
        IntegrityStatus::SignedButBroken
    );
}

#[test]
fn signed_file_with_unbound_engine_field_is_signed_but_broken() {
    let mut pkg = compiled_contract();
    let key = generate_secret_key().unwrap();
    sign_with_fixed_time(&mut pkg, &key);
    let mut lock: k2f_core::LockFile =
        serde_json::from_str(pkg.lock_json.as_ref().unwrap()).unwrap();
    lock.engine_version = "9.9.9".into();
    pkg.set_lock(&lock).unwrap();
    let report = inspect_package(&pkg).unwrap();
    assert_eq!(report.status, IntegrityStatus::SignedButBroken);
    assert_eq!(report.hash, VerifyStatus::AppearanceChanged);
}

#[test]
fn refuse_negative_signed_at() {
    let mut pkg = compiled_contract();
    let key = generate_secret_key().unwrap();
    let err = sign_package(&mut pkg, &key, None, -1).unwrap_err();
    assert!(err.to_string().contains("signed_at"), "got {err}");
}

#[test]
fn empty_signatures_file_is_signed_but_broken_not_unsigned() {
    let mut pkg = compiled_contract();
    pkg.signatures_json = Some(r#"{"signatures":[],"version":1}"#.into());
    assert_eq!(
        inspect_package(&pkg).unwrap().status,
        IntegrityStatus::SignedButBroken
    );
}
