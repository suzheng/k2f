mod common;

use common::{compile_pkg, load_contract_engine, load_font, packed_contract, repo_root};
use k2f_core::{CanvasMode, Pt};
use k2f_package::{
    pack_bytes, package_from_engine, unpack_bytes, verify_package, VerifyStatus, CODE_NODE_ID,
    CODE_SCHEMA_INVALID,
};
use std::fs;

#[test]
fn pack_twice_is_byte_identical() {
    let (engine, theme) = load_contract_engine();
    let pkg = package_from_engine(
        engine,
        theme,
        "assets/fonts/NotoSansSC-Regular.otf",
        load_font(),
    )
    .unwrap();
    assert_eq!(pack_bytes(&pkg).unwrap(), pack_bytes(&pkg).unwrap());
}

#[test]
fn unpack_roundtrip_preserves_title() {
    let pkg = unpack_bytes(&packed_contract()).unwrap();
    assert_eq!(pkg.manifest.title, "独立顾问协议");
    assert_eq!(pkg.manifest.canvas_mode, CanvasMode::Paged);
    let lock: k2f_core::LockFile = serde_json::from_str(pkg.lock_json.as_ref().unwrap()).unwrap();
    assert_eq!(pkg.manifest.engine_version, lock.engine_version);
    assert!(!lock.engine_version.is_empty());
    assert!(pkg
        .fonts
        .contains_key("assets/fonts/NotoSansSC-Regular.otf"));
}

#[test]
fn verify_without_lock_is_unlocked() {
    let mut pkg = unpack_bytes(&packed_contract()).unwrap();
    pkg.lock_json = None;
    assert_eq!(verify_package(&pkg).unwrap(), VerifyStatus::Unlocked);
}

#[test]
fn compile_then_verify_is_valid() {
    let mut pkg = unpack_bytes(&packed_contract()).unwrap();
    compile_pkg(&mut pkg);
    assert_eq!(verify_package(&pkg).unwrap(), VerifyStatus::Valid);
}

#[test]
fn changing_text_breaks_content_hash() {
    let mut pkg = unpack_bytes(&packed_contract()).unwrap();
    compile_pkg(&mut pkg);
    let node = k2f_core::find_node_mut(&mut pkg.root, "contract.clause_1").unwrap();
    match &mut node.content {
        k2f_core::NodeContent::Text(t) => t.push_str(" CHANGED"),
        other => panic!("expected clause_1 text, got {other:?}"),
    }
    assert_eq!(verify_package(&pkg).unwrap(), VerifyStatus::ContentChanged);
}

#[test]
fn changing_theme_breaks_appearance_not_content() {
    let mut pkg = unpack_bytes(&packed_contract()).unwrap();
    compile_pkg(&mut pkg);
    let mut manifest = pkg.engine_manifest();
    k2f_core::normalize_manifest_nfc(&mut manifest);
    let content_hash = k2f_core::hash_manifest_semantic(&manifest);
    pkg.theme_json = pkg.theme_json.replace("#FFFFFF", "#000000");
    assert_eq!(
        verify_package(&pkg).unwrap(),
        VerifyStatus::AppearanceChanged
    );
    assert_eq!(k2f_core::hash_manifest_semantic(&manifest), content_hash);
}

#[test]
fn changing_page_margin_breaks_appearance_not_content() {
    let mut pkg = unpack_bytes(&packed_contract()).unwrap();
    compile_pkg(&mut pkg);
    let mut manifest = pkg.engine_manifest();
    k2f_core::normalize_manifest_nfc(&mut manifest);
    let content_hash = k2f_core::hash_manifest_semantic(&manifest);
    pkg.manifest.page_config.margin = [Pt(10000); 4];
    assert_eq!(
        verify_package(&pkg).unwrap(),
        VerifyStatus::AppearanceChanged
    );
    assert_eq!(k2f_core::hash_manifest_semantic(&manifest), content_hash);
}

#[test]
fn extra_json_field_fails_schema() {
    let (engine, theme) = load_contract_engine();
    let mut pkg = package_from_engine(
        engine,
        theme,
        "assets/fonts/NotoSansSC-Regular.otf",
        load_font(),
    )
    .unwrap();
    pkg.theme_json = r#"{"palette":{},"roles":{},"not_a_real_field":true}"#.to_string();
    let err = pack_bytes(&pkg).unwrap_err();
    assert!(err.to_string().contains(CODE_SCHEMA_INVALID), "got {err}");
}

#[test]
fn duplicate_ids_rejected_on_construct() {
    let (mut engine, theme) = load_contract_engine();
    engine.root.id = "contract.title".to_string();
    let err = package_from_engine(
        engine,
        theme,
        "assets/fonts/NotoSansSC-Regular.otf",
        load_font(),
    )
    .unwrap_err();
    assert!(err.to_string().contains(CODE_NODE_ID), "got {err}");
    assert!(err.to_string().contains("duplicate"), "got {err}");
}

#[test]
fn pack_rejects_duplicate_ids_after_mutation() {
    let (engine, theme) = load_contract_engine();
    let mut pkg = package_from_engine(
        engine,
        theme,
        "assets/fonts/NotoSansSC-Regular.otf",
        load_font(),
    )
    .unwrap();
    pkg.root.id = "contract.title".to_string();
    let err = pack_bytes(&pkg).unwrap_err();
    assert!(err.to_string().contains(CODE_NODE_ID), "got {err}");
}

#[test]
fn engine_version_tamper_without_rebind_is_appearance_changed() {
    let mut pkg = unpack_bytes(&packed_contract()).unwrap();
    compile_pkg(&mut pkg);
    let mut lock: k2f_core::LockFile =
        serde_json::from_str(pkg.lock_json.as_ref().unwrap()).unwrap();
    lock.engine_version = "9.9.9".to_string();
    pkg.set_lock(&lock).unwrap();
    assert_eq!(
        verify_package(&pkg).unwrap(),
        VerifyStatus::AppearanceChanged
    );
}

#[test]
fn verify_without_fonts_is_font_missing() {
    let mut pkg = unpack_bytes(&packed_contract()).unwrap();
    compile_pkg(&mut pkg);
    pkg.fonts.clear();
    assert_eq!(verify_package(&pkg).unwrap(), VerifyStatus::FontMissing);
}

#[test]
fn invoice_template_compiles_and_verifies() {
    let bytes = fs::read(repo_root().join("examples/published/invoice.K2F")).unwrap();
    let mut pkg = unpack_bytes(&bytes).unwrap();
    compile_pkg(&mut pkg);
    let packed = pack_bytes(&pkg).unwrap();
    assert_eq!(packed, pack_bytes(&unpack_bytes(&packed).unwrap()).unwrap());
    assert_eq!(verify_package(&pkg).unwrap(), VerifyStatus::Valid);
}

#[test]
fn licenses_only_fonts_are_font_missing() {
    let (engine, theme) = load_contract_engine();
    let mut fonts = std::collections::BTreeMap::new();
    fonts.insert(
        "assets/fonts/licenses/Roboto-Apache.txt".into(),
        b"Apache-2.0".to_vec(),
    );
    let err = k2f_package::Package::from_engine_parts(
        engine.title.clone(),
        None,
        Some(0),
        engine,
        theme,
        fonts,
        Default::default(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("FONT_MISSING"), "got {err}");
}

#[test]
fn starter_keeps_license_files_and_has_a_face() {
    let pkg = k2f_package::load_dir(&repo_root().join("skills/k2f/starter")).unwrap();
    assert!(pkg
        .fonts
        .keys()
        .any(|p| p.contains("licenses/") && p.ends_with(".txt")));
    assert!(k2f_package::paths::has_font_face(&pkg.fonts));
}

#[test]
fn pack_rejects_svg_text_element() {
    let (engine, theme) = load_contract_engine();
    let mut pkg = package_from_engine(
        engine,
        theme,
        "assets/fonts/NotoSansSC-Regular.otf",
        load_font(),
    )
    .unwrap();
    pkg.assets.insert(
        "assets/images/mark.svg".into(),
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><text x="1" y="2">A</text></svg>"#.to_vec(),
    );
    let err = pack_bytes(&pkg).unwrap_err();
    assert!(
        err.to_string().contains(k2f_package::CODE_IMAGE_SIZE),
        "got {err}"
    );
    assert!(err.to_string().contains("<text>"), "got {err}");
}

#[test]
fn pack_allows_svg_comment_mentioning_text() {
    let (engine, theme) = load_contract_engine();
    let mut pkg = package_from_engine(
        engine,
        theme,
        "assets/fonts/NotoSansSC-Regular.otf",
        load_font(),
    )
    .unwrap();
    pkg.assets.insert(
        "assets/images/mark.svg".into(),
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><!-- <text> converted to path --><rect width="4" height="4" fill="#00f"/></svg>"##.to_vec(),
    );
    pack_bytes(&pkg).unwrap();
}

#[test]
fn pack_rejects_svg_textpath() {
    let (engine, theme) = load_contract_engine();
    let mut pkg = package_from_engine(
        engine,
        theme,
        "assets/fonts/NotoSansSC-Regular.otf",
        load_font(),
    )
    .unwrap();
    pkg.assets.insert(
        "assets/images/mark.svg".into(),
        br##"<svg xmlns="http://www.w3.org/2000/svg"><textPath href="#p">A</textPath></svg>"##
            .to_vec(),
    );
    let err = pack_bytes(&pkg).unwrap_err();
    assert!(
        err.to_string().contains(k2f_package::CODE_IMAGE_SIZE),
        "got {err}"
    );
}

#[test]
fn pack_rejects_svg_foreign_object() {
    let (engine, theme) = load_contract_engine();
    let mut pkg = package_from_engine(
        engine,
        theme,
        "assets/fonts/NotoSansSC-Regular.otf",
        load_font(),
    )
    .unwrap();
    pkg.assets.insert(
        "assets/images/mark.svg".into(),
        br##"<svg xmlns="http://www.w3.org/2000/svg"><foreignObject width="4" height="4">A</foreignObject></svg>"##.to_vec(),
    );
    let err = pack_bytes(&pkg).unwrap_err();
    assert!(
        err.to_string().contains(k2f_package::CODE_IMAGE_SIZE),
        "got {err}"
    );
}
