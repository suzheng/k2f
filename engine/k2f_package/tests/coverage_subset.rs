mod common;

use common::{compile_pkg, load_font, packed_contract, repo_root};
use k2f_core::{for_each_node_mut, NodeContent};
use k2f_package::{
    apply_coverage_subset, pack_bytes, unpack_bytes, verify_package, VerifyStatus,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[test]
fn roboto_bytes_unchanged() {
    let bytes = fs::read(repo_root().join("assets/fonts/Roboto-Regular.ttf")).unwrap();
    let mut fonts = BTreeMap::new();
    fonts.insert("assets/fonts/Roboto-Regular.ttf".into(), bytes.clone());
    apply_coverage_subset(&mut fonts).unwrap();
    assert_eq!(
        fonts.get("assets/fonts/Roboto-Regular.ttf").unwrap(),
        &bytes
    );
}

#[test]
fn official_sc_demo_bytes_unchanged() {
    let bytes = load_font();
    let mut fonts = BTreeMap::new();
    fonts.insert("assets/fonts/NotoSansSC-Regular.otf".into(), bytes.clone());
    apply_coverage_subset(&mut fonts).unwrap();
    assert_eq!(
        fonts.get("assets/fonts/NotoSansSC-Regular.otf").unwrap(),
        &bytes
    );
}

fn full_sc_face() -> Option<(String, Vec<u8>)> {
    let deck = Path::new("/Users/suzheng/Downloads/soft-morning-light-deck.K2F");
    if deck.exists() {
        let pkg = unpack_bytes(&fs::read(deck).ok()?).ok()?;
        return pkg.fonts.into_iter().find(|(path, bytes)| {
            k2f_core::is_font_face_path(path) && bytes.len() > 1_500_000
        });
    }
    let cache = repo_root().join("target/font-src/NotoSansSC-Regular-full.otf");
    let bytes = fs::read(cache).ok()?;
    if bytes.len() > 1_500_000 {
        Some(("assets/fonts/NotoSansSC-Regular.otf".into(), bytes))
    } else {
        None
    }
}

fn compile_with_text(fonts: BTreeMap<String, Vec<u8>>, text: &str) -> Result<(), String> {
    let mut pkg = unpack_bytes(&packed_contract()).unwrap();
    pkg.fonts = fonts;
    pkg.lock_json = None;
    for_each_node_mut(&mut pkg.root, &mut |node| {
        if let NodeContent::Text(value) = &mut node.content {
            if !value.is_empty() {
                *value = text.into();
            }
        }
    });
    let assets: std::collections::HashMap<String, Vec<u8>> =
        pkg.assets.clone().into_iter().collect();
    k2f_layout::compile_manifest(
        pkg.engine_manifest(),
        &pkg.theme_json,
        &pkg.fonts,
        if assets.is_empty() {
            None
        } else {
            Some(&assets)
        },
    )
    .map(|_| ())
}

#[test]
fn full_sc_coverage_subset_shrinks_and_keeps_union() {
    let Some((path, full)) = full_sc_face() else {
        eprintln!("skip full_sc_coverage_subset_shrinks_and_keeps_union: no full Noto Sans SC fixture");
        return;
    };
    assert!(full.len() > 10_000_000, "expected full CJK face, got {}", full.len());
    let mut fonts = BTreeMap::new();
    fonts.insert(path.clone(), full);
    apply_coverage_subset(&mut fonts).unwrap();
    let cut = fonts.get(&path).unwrap();
    assert!(
        cut.len() < 9_000_000,
        "coverage subset still large: {}",
        cut.len()
    );
    assert!(cut.len() > 1_000_000, "coverage subset unexpectedly tiny: {}", cut.len());

    compile_with_text(fonts.clone(), "你體灣").expect("GB2312/Big5 daily Han must compile");

    let mut pkg = unpack_bytes(&packed_contract()).unwrap();
    pkg.fonts = fonts;
    pkg.lock_json = None;
    compile_pkg(&mut pkg);
    assert_eq!(verify_package(&pkg).unwrap(), VerifyStatus::Valid);
    let packed = pack_bytes(&pkg).unwrap();
    assert!(packed.len() < 10_000_000, "packed {}", packed.len());
    let again = unpack_bytes(&packed).unwrap();
    assert_eq!(pack_bytes(&again).unwrap(), packed);
}

#[test]
fn coverage_subset_drops_han_outside_union() {
    let Some((path, full)) = full_sc_face() else {
        eprintln!("skip coverage_subset_drops_han_outside_union: no full Noto Sans SC fixture");
        return;
    };
    let mut fonts = BTreeMap::new();
    fonts.insert(path, full);
    apply_coverage_subset(&mut fonts).unwrap();
    let err = compile_with_text(fonts, "龦").unwrap_err();
    assert!(
        err.contains("FONT_MISSING_GLYPH"),
        "expected missing glyph, got {err}"
    );
}
