//! Trust / signing smoke for the demo key used by the site portal.
//! Site demo fixtures are written by `k2f-site/scripts/write-trust-fixtures.mjs` (JS SDK only).

use k2f_core::find_node_mut;
use k2f_package::{
    inspect_package, sign_package, unpack_bytes, IntegrityStatus, SecretKey,
};
use k2f_sdk::Editor;
use std::fs;
use std::path::PathBuf;

const DEMO_SECRET_HEX: &str = "6fd6bd821ea700ab1399e0902257c068a139567df487ce515825a527f04ec90f";
const DEMO_FINGERPRINT: &str = "a82d7f9b60155ac1c0b6add8ffe932b63be554fdf53749a4add709ee128129ca";
const DEMO_SIGNED_BY: &str = "Demo Legal";
const DEMO_SIGNED_AT: i64 = 1_704_067_200;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn relocked_contract() -> Vec<u8> {
    let raw = fs::read(repo_root().join("examples/published/contract.K2F")).unwrap();
    let mut editor = Editor::open(&raw).unwrap();
    editor.save_bytes().unwrap()
}

fn tamper_clause_2(pkg: &mut k2f_package::Package) {
    let node = find_node_mut(&mut pkg.root, "contract.clause_2").unwrap();
    match &mut node.content {
        k2f_core::NodeContent::Text(t) => t.push_str(" [tampered]"),
        other => panic!("expected contract.clause_2 text, got {other:?}"),
    }
}

#[test]
fn demo_signing_key_matches_fingerprint() {
    let key = SecretKey::from_hex(DEMO_SECRET_HEX).unwrap();
    assert_eq!(key.fingerprint(), DEMO_FINGERPRINT);
}

#[test]
fn relocked_contract_signs_and_tampers() {
    let relocked = relocked_contract();
    assert_eq!(
        inspect_package(&unpack_bytes(&relocked).unwrap())
            .unwrap()
            .status,
        IntegrityStatus::Unsigned
    );
    let key = SecretKey::from_hex(DEMO_SECRET_HEX).unwrap();
    let mut pkg = unpack_bytes(&relocked).unwrap();
    sign_package(&mut pkg, &key, Some(DEMO_SIGNED_BY), DEMO_SIGNED_AT).unwrap();
    tamper_clause_2(&mut pkg);
    assert_eq!(
        inspect_package(&pkg).unwrap().status,
        IntegrityStatus::SignedButBroken
    );
}
