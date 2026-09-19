mod common;

use k2f_idml::{export_bytes, export_opened, fmt_pt, millipt_to_pt, IdmlError, SpreadSpace, MIME};
use zip::CompressionMethod;

#[test]
fn coord_unit_tests_already_in_src() {
    assert_eq!(fmt_pt(millipt_to_pt(595_000)), "595.000");
    assert_eq!(fmt_pt(millipt_to_pt(842_000)), "842.000");
    assert_eq!(fmt_pt(millipt_to_pt(960_000)), "960.000");
    assert_eq!(fmt_pt(millipt_to_pt(540_000)), "540.000");
}

#[test]
fn export_invoice_is_zip_and_has_spread_per_lock_page() {
    let doc = common::invoice();
    let idml = export_opened(&doc).unwrap();
    assert!(idml.starts_with(b"PK"), "idml must be a zip");
    let lock = doc.lock().expect("invoice is locked");
    let n = lock.geometry.pages.len();
    let names = common::unzip_names(&idml);
    let spreads: Vec<_> = names
        .iter()
        .filter(|s| s.starts_with("Spreads/Spread_k") && s.ends_with(".xml"))
        .collect();
    assert_eq!(spreads.len(), n);
    for i in 0..n {
        let want = format!("Spreads/Spread_k{i}.xml");
        assert!(
            names.iter().any(|s| s == &want),
            "missing {want} in {names:?}"
        );
    }
    let map = common::xml_in(&idml, "designmap.xml");
    let parsed = roxmltree::Document::parse(&map).unwrap();
    let listed = parsed
        .descendants()
        .filter(|n| n.has_tag_name("Spread"))
        .count();
    assert_eq!(listed, n);
}

#[test]
fn mimetype_is_first_entry_stored() {
    let idml = export_opened(&common::invoice()).unwrap();
    let (name, method) = common::zip_index0_name_and_method(&idml);
    assert_eq!(name, "mimetype");
    assert_eq!(method, CompressionMethod::Stored);
    let body = String::from_utf8(common::bytes_in(&idml, "mimetype")).unwrap();
    assert_eq!(body.trim(), MIME);
}

#[test]
fn page_size_matches_lock_page_config() {
    let doc = common::invoice();
    let idml = export_opened(&doc).unwrap();
    let lock = doc.lock().expect("invoice is locked");
    let page = &lock.geometry.pages[0];
    let xml = common::xml_in(&idml, "Resources/Preferences.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let pref = parsed
        .descendants()
        .find(|n| n.has_tag_name("DocumentPreference"))
        .expect("DocumentPreference");
    let w = fmt_pt(millipt_to_pt(page.width.0));
    let h = fmt_pt(millipt_to_pt(page.height.0));
    assert_eq!(pref.attribute("PageWidth"), Some(w.as_str()));
    assert_eq!(pref.attribute("PageHeight"), Some(h.as_str()));
    assert_eq!(
        pref.attribute("Intent"),
        Some("WebIntent"),
        "K2F paint is RGB; PrintIntent converts fills to CMYK"
    );
    assert_eq!(w, "595.000");
    assert_eq!(h, "842.000");
}

#[test]
fn page_geometric_bounds_centered() {
    let doc = common::invoice();
    let idml = export_opened(&doc).unwrap();
    let lock = doc.lock().expect("invoice is locked");
    let page = &lock.geometry.pages[0];
    let space = SpreadSpace::new(page.width, page.height);
    let want = space.page_geometric_bounds();
    let xml = common::xml_in(&idml, "Spreads/Spread_k0.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let el = parsed
        .descendants()
        .find(|n| n.has_tag_name("Page"))
        .expect("Page");
    assert_eq!(el.attribute("GeometricBounds"), Some(want.as_str()));
}

#[test]
fn export_twice_byte_identical() {
    let doc = common::invoice();
    let a = export_opened(&doc).unwrap();
    let b = export_opened(&doc).unwrap();
    assert_eq!(a, b);
}

#[test]
fn idml_input_rejected() {
    let idml = export_opened(&common::invoice()).unwrap();
    let err = export_bytes(&idml).unwrap_err();
    assert!(matches!(err, IdmlError::NotASource), "got {err}");
    assert!(err.to_string().contains("IDML_IS_NOT_A_SOURCE"));
}

#[test]
fn unlocked_fails() {
    let err = export_bytes(&[]).unwrap_err();
    assert!(!matches!(err, IdmlError::NotASource));
}

#[test]
fn unknown_paint_op_gate_present() {
    let doc = common::invoice();
    let lock = doc.lock().expect("invoice is locked");
    assert!(!lock.has_unknown_paint_ops());
    export_opened(&doc).unwrap();
}

#[test]
fn designmap_has_aid_processing_instruction() {
    let idml = export_opened(&common::invoice()).unwrap();
    let map = common::xml_in(&idml, "designmap.xml");
    assert!(
        map.contains(r#"<?aid style="50" type="document" readerVersion="6.0" featureSet="257""#),
        "InDesign requires the aid PI on designmap.xml, got {map}"
    );
}

#[test]
fn required_parts_present() {
    let doc = common::invoice();
    let idml = export_opened(&doc).unwrap();
    let n = doc.lock().unwrap().geometry.pages.len();
    let names = common::unzip_names(&idml);
    let mut required = vec![
        "mimetype".into(),
        "designmap.xml".into(),
        "META-INF/container.xml".into(),
        "META-INF/metadata.xml".into(),
        "Resources/Fonts.xml".into(),
        "Resources/Graphic.xml".into(),
        "Resources/Preferences.xml".into(),
        "Resources/Styles.xml".into(),
        "MasterSpreads/MasterSpread_kMaster.xml".into(),
        "XML/BackingStory.xml".into(),
        "XML/Tags.xml".into(),
    ];
    for i in 0..n {
        required.push(format!("Spreads/Spread_k{i}.xml"));
    }
    for part in &required {
        assert!(
            names.iter().any(|s| s == part),
            "missing {part} in {names:?}"
        );
    }
}

#[test]
fn facing_pages_false() {
    let idml = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&idml, "Resources/Preferences.xml");
    assert!(
        xml.contains(r#"FacingPages="false""#),
        "FacingPages must be false, got {xml}"
    );
}
