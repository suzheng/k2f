mod common;

use common::{compile_pkg, packed_contract, rebind_lock_engine};
use k2f_package::{
    generate_secret_key, inspect_package, sign_package, unpack_bytes, verify_package,
    IntegrityStatus, VerifyStatus,
};

fn compiled() -> k2f_package::Package {
    let mut pkg = unpack_bytes(&packed_contract()).unwrap();
    compile_pkg(&mut pkg);
    pkg
}

#[test]
fn rebound_engine_version_is_mismatch_not_broken() {
    let mut pkg = compiled();
    rebind_lock_engine(&mut pkg, |lock| {
        lock.engine_version = "9.9.9".into();
    });
    assert_eq!(verify_package(&pkg).unwrap(), VerifyStatus::EngineMismatch);
    let report = inspect_package(&pkg).unwrap();
    assert_eq!(report.status, IntegrityStatus::Unsigned);
    assert!(report.status.is_ok());
    assert_eq!(report.hash, VerifyStatus::EngineMismatch);
}

#[test]
fn rebound_engine_commit_sha_is_mismatch_not_broken() {
    let mut pkg = compiled();
    rebind_lock_engine(&mut pkg, |lock| {
        lock.engine_commit_sha = "deadbeef".into();
    });
    assert_eq!(verify_package(&pkg).unwrap(), VerifyStatus::EngineMismatch);
    assert_eq!(
        inspect_package(&pkg).unwrap().status,
        IntegrityStatus::Unsigned
    );
}

#[test]
fn committed_contract_is_self_consistent() {
    let pkg = unpack_bytes(&packed_contract()).unwrap();
    let hash = verify_package(&pkg).unwrap();
    assert!(
        matches!(hash, VerifyStatus::Valid | VerifyStatus::EngineMismatch),
        "published lock must verify against its own engine identity, got {hash:?}"
    );
    let report = inspect_package(&pkg).unwrap();
    assert_eq!(report.status, IntegrityStatus::Unsigned);
    assert!(report.status.is_ok());
}

#[test]
fn sign_after_rebind_stays_signed() {
    let mut pkg = compiled();
    rebind_lock_engine(&mut pkg, |lock| {
        lock.engine_commit_sha = "deadbeef".into();
    });
    let key = generate_secret_key().unwrap();
    sign_package(&mut pkg, &key, Some("Acme Legal"), 1_704_067_200).unwrap();
    let report = inspect_package(&pkg).unwrap();
    assert_eq!(report.status, IntegrityStatus::Signed);
    assert_eq!(report.hash, VerifyStatus::EngineMismatch);
    assert_eq!(report.fingerprint, Some(key.fingerprint()));
}

#[test]
fn sign_then_engine_tamper_without_rebind_is_signed_but_broken() {
    let mut pkg = compiled();
    let key = generate_secret_key().unwrap();
    sign_package(&mut pkg, &key, Some("Acme Legal"), 1_704_067_200).unwrap();
    let mut lock: k2f_core::LockFile =
        serde_json::from_str(pkg.lock_json.as_ref().unwrap()).unwrap();
    lock.engine_version = "9.9.9".into();
    pkg.set_lock(&lock).unwrap();
    let report = inspect_package(&pkg).unwrap();
    assert_eq!(report.status, IntegrityStatus::SignedButBroken);
    assert_eq!(report.hash, VerifyStatus::AppearanceChanged);
}
