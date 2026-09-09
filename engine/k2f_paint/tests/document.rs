mod common;

use common::contract_k2f_bytes;
use k2f_package::{appearance_hash_for_lock, unpack_bytes, VerifyStatus};
use k2f_paint::{Banner, OpenedDocument, PaintError, OFFICIAL_PNG_SCALE};

#[test]
fn committed_contract_opens_valid_and_paints() {
    let doc = OpenedDocument::open(&contract_k2f_bytes()).unwrap();
    assert_eq!(doc.banner(), Banner::Unsigned);
    assert!(
        matches!(
            doc.status(),
            VerifyStatus::Valid | VerifyStatus::EngineMismatch
        ),
        "published contract must be self-consistent, got {:?}",
        doc.status()
    );
    assert_eq!(doc.status_code(), "UNSIGNED");
    assert!(doc.page_count() >= 1);
    let png = doc.render_page(0, OFFICIAL_PNG_SCALE).unwrap();
    assert!(png.starts_with(b"\x89PNG"));
    let png2 = doc.render_page(0, OFFICIAL_PNG_SCALE).unwrap();
    assert_eq!(png, png2);
}

#[test]
fn theme_change_without_recompile_is_broken_but_paints_old_lock() {
    let bytes = contract_k2f_bytes();
    let valid = OpenedDocument::open(&bytes).unwrap();
    let old_png = valid.render_page(0, OFFICIAL_PNG_SCALE).unwrap();

    let mut pkg = unpack_bytes(&bytes).unwrap();
    assert!(
        pkg.theme_json.contains("#FFFFFF") || pkg.theme_json.contains("#ffffff"),
        "fixture theme must contain white so the appearance hash changes"
    );
    pkg.theme_json = pkg.theme_json.replace("#FFFFFF", "#000000");
    pkg.theme_json = pkg.theme_json.replace("#ffffff", "#000000");

    let broken = OpenedDocument::from_package(pkg).unwrap();
    assert_eq!(broken.banner(), Banner::BrokenIntegrity);
    assert_eq!(broken.status(), VerifyStatus::AppearanceChanged);
    assert_eq!(broken.status_code(), "APPEARANCE_CHANGED");
    assert_eq!(
        broken.render_page(0, OFFICIAL_PNG_SCALE).unwrap(),
        old_png,
        "broken files must still show the published lock, not a reflow"
    );
}

#[test]
fn content_change_without_recompile_is_broken_but_paints_old_lock() {
    let bytes = contract_k2f_bytes();
    let valid = OpenedDocument::open(&bytes).unwrap();
    let old_png = valid.render_page(0, OFFICIAL_PNG_SCALE).unwrap();

    let mut pkg = unpack_bytes(&bytes).unwrap();
    let node = k2f_core::find_node_mut(&mut pkg.root, "contract.clause_2").unwrap();
    match &mut node.content {
        k2f_core::NodeContent::Text(t) => t.push_str(" TAMPERED"),
        other => panic!("expected contract.clause_2 text, got {other:?}"),
    }

    let broken = OpenedDocument::from_package(pkg).unwrap();
    assert_eq!(broken.banner(), Banner::BrokenIntegrity);
    assert_eq!(broken.status(), VerifyStatus::ContentChanged);
    assert_eq!(broken.status_code(), "CONTENT_CHANGED");
    assert_eq!(
        broken.render_page(0, OFFICIAL_PNG_SCALE).unwrap(),
        old_png,
        "tampered content must still show the published lock"
    );
}

#[test]
fn engine_mismatch_is_quiet_and_paints_old_lock() {
    let bytes = contract_k2f_bytes();
    let valid = OpenedDocument::open(&bytes).unwrap();
    let old_png = valid.render_page(0, OFFICIAL_PNG_SCALE).unwrap();

    let mut pkg = unpack_bytes(&bytes).unwrap();
    let mut lock: k2f_core::LockFile =
        serde_json::from_str(pkg.lock_json.as_ref().unwrap()).unwrap();
    lock.engine_version = "9.9.9".to_string();
    lock.appearance_hash = appearance_hash_for_lock(&pkg, &lock).unwrap();
    pkg.set_lock(&lock).unwrap();

    let opened = OpenedDocument::from_package(pkg).unwrap();
    assert_eq!(opened.banner(), Banner::Unsigned);
    assert_eq!(opened.status(), VerifyStatus::EngineMismatch);
    assert_eq!(opened.status_code(), "UNSIGNED");
    assert_eq!(opened.hash_code(), "ENGINE_MISMATCH");
    assert_eq!(opened.render_page(0, OFFICIAL_PNG_SCALE).unwrap(), old_png);
}

#[test]
fn engine_field_tamper_without_rebind_is_broken_but_paints_old_lock() {
    let bytes = contract_k2f_bytes();
    let valid = OpenedDocument::open(&bytes).unwrap();
    let old_png = valid.render_page(0, OFFICIAL_PNG_SCALE).unwrap();

    let mut pkg = unpack_bytes(&bytes).unwrap();
    let mut lock: k2f_core::LockFile =
        serde_json::from_str(pkg.lock_json.as_ref().unwrap()).unwrap();
    lock.engine_version = "9.9.9".to_string();
    pkg.set_lock(&lock).unwrap();

    let broken = OpenedDocument::from_package(pkg).unwrap();
    assert_eq!(broken.banner(), Banner::BrokenIntegrity);
    assert_eq!(broken.status(), VerifyStatus::AppearanceChanged);
    assert_eq!(broken.status_code(), "APPEARANCE_CHANGED");
    assert_eq!(broken.render_page(0, OFFICIAL_PNG_SCALE).unwrap(), old_png);
}

#[test]
fn unlocked_draft_has_no_pages() {
    let mut pkg = unpack_bytes(&contract_k2f_bytes()).unwrap();
    pkg.lock_json = None;
    let doc = OpenedDocument::from_package(pkg).unwrap();
    assert_eq!(doc.banner(), Banner::Unlocked);
    assert_eq!(doc.page_count(), 0);
    assert!(matches!(
        doc.render_page(0, OFFICIAL_PNG_SCALE),
        Err(PaintError::Unlocked)
    ));
}

#[test]
fn signed_contract_banner_is_signed_and_still_paints() {
    let mut pkg = unpack_bytes(&contract_k2f_bytes()).unwrap();
    let key = k2f_package::generate_secret_key().unwrap();
    k2f_package::sign_package(&mut pkg, &key, Some("Acme Legal"), 1_704_067_200).unwrap();
    let bytes = k2f_package::pack_bytes(&pkg).unwrap();
    let doc = OpenedDocument::open(&bytes).unwrap();
    assert_eq!(doc.banner(), Banner::Signed);
    assert_eq!(doc.status_code(), "SIGNED");
    assert_eq!(doc.signed_by(), Some("Acme Legal"));
    assert_eq!(doc.fingerprint().unwrap(), key.fingerprint());
    let png = doc.render_page(0, OFFICIAL_PNG_SCALE).unwrap();
    assert!(png.starts_with(b"\x89PNG"));
}

#[test]
fn signed_then_amount_change_is_signed_but_broken_and_paints_old_lock() {
    let bytes = contract_k2f_bytes();
    let mut pkg = unpack_bytes(&bytes).unwrap();
    let key = k2f_package::generate_secret_key().unwrap();
    k2f_package::sign_package(&mut pkg, &key, Some("Acme Legal"), 1_704_067_200).unwrap();
    let signed = k2f_package::pack_bytes(&pkg).unwrap();
    let valid = OpenedDocument::open(&signed).unwrap();
    let old_png = valid.render_page(0, OFFICIAL_PNG_SCALE).unwrap();

    let mut pkg = unpack_bytes(&signed).unwrap();
    let node = k2f_core::find_node_mut(&mut pkg.root, "contract.clause_2").unwrap();
    match &mut node.content {
        k2f_core::NodeContent::Text(t) => t.push_str(" 110"),
        other => panic!("expected contract.clause_2 text, got {other:?}"),
    }
    let broken = OpenedDocument::from_package(pkg).unwrap();
    assert_eq!(broken.banner(), Banner::SignedButBroken);
    assert_eq!(broken.status_code(), "SIGNED_BUT_BROKEN");
    assert_eq!(
        broken.render_page(0, OFFICIAL_PNG_SCALE).unwrap(),
        old_png,
        "signed-but-broken must still show the published lock, not a reflow"
    );
}
