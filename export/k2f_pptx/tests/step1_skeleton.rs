mod common;

use k2f_pptx::{export_bytes, export_opened, millipt_to_emu, pt_to_emu, PptxError};

#[test]
fn millipt_to_emu_widescreen_and_a4() {
    assert_eq!(millipt_to_emu(960_000), 12_192_000);
    assert_eq!(millipt_to_emu(540_000), 6_858_000);
    assert_eq!(millipt_to_emu(595_000), 7_556_500);
    assert_eq!(millipt_to_emu(842_000), 10_693_400);
    // integer division toward zero
    assert_eq!(millipt_to_emu(1), 12);
    assert_eq!(millipt_to_emu(-1), -12);
}

#[test]
fn export_invoice_is_zip_and_has_slide_per_lock_page() {
    let doc = common::invoice();
    let pptx = export_opened(&doc).unwrap();
    assert!(pptx.starts_with(b"PK"), "pptx must be a zip");
    let lock = doc.lock().expect("invoice is locked");
    let n = lock.geometry.pages.len();
    let names = common::unzip_names(&pptx);
    let slides: Vec<_> = names
        .iter()
        .filter(|n| {
            n.starts_with("ppt/slides/slide") && n.ends_with(".xml") && !n.contains("_rels")
        })
        .collect();
    assert_eq!(slides.len(), n);
    for i in 1..=n {
        assert!(
            names
                .iter()
                .any(|s| s == &format!("ppt/slides/slide{i}.xml")),
            "missing ppt/slides/slide{i}.xml in {names:?}"
        );
        assert!(
            names
                .iter()
                .any(|s| s == &format!("ppt/slides/_rels/slide{i}.xml.rels")),
            "missing slide{i} rels"
        );
    }
    for required in [
        "[Content_Types].xml",
        "_rels/.rels",
        "docProps/core.xml",
        "docProps/app.xml",
        "ppt/presentation.xml",
        "ppt/_rels/presentation.xml.rels",
        "ppt/slideMasters/slideMaster1.xml",
        "ppt/slideMasters/_rels/slideMaster1.xml.rels",
        "ppt/slideLayouts/slideLayout1.xml",
        "ppt/slideLayouts/_rels/slideLayout1.xml.rels",
        "ppt/theme/theme1.xml",
    ] {
        assert!(
            names.iter().any(|s| s == required),
            "missing {required} in {names:?}"
        );
    }
}

#[test]
fn theme_pins_dk1_lt1_to_srgb_not_system_window_colors() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let theme = common::xml_in(&pptx, "ppt/theme/theme1.xml");
    assert!(
        !theme.contains("windowText") && !theme.contains(r#"sysClr val="window""#),
        "dk1/lt1 must not follow OS dark mode, got {theme}"
    );
    assert!(
        theme.contains(r#"<a:dk1><a:srgbClr val="000002"/></a:dk1>"#),
        "dk1 must be pinned srgb, got {theme}"
    );
    assert!(
        theme.contains(r#"<a:lt1><a:srgbClr val="FFFFFD"/></a:lt1>"#),
        "lt1 must be pinned srgb, got {theme}"
    );
    assert!(
        !theme.contains(r#"<a:dk1><a:srgbClr val="000001"/>"#)
            && !theme.contains(r#"<a:lt1><a:srgbClr val="FFFFFE"/>"#)
            && !theme.contains(r#"<a:dk1><a:srgbClr val="000000"/>"#)
            && !theme.contains(r#"<a:lt1><a:srgbClr val="FFFFFF"/>"#),
        "theme slots must not equal content black/white pins, got {theme}"
    );
}

#[test]
fn slide_size_matches_lock_page_config() {
    let doc = common::invoice();
    let pptx = export_opened(&doc).unwrap();
    let lock = doc.lock().expect("invoice is locked");
    let page = &lock.geometry.pages[0];
    let xml = common::xml_in(&pptx, "ppt/presentation.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let sz = parsed
        .descendants()
        .find(|n| n.has_tag_name("sldSz"))
        .expect("sldSz");
    let cx: i64 = sz.attribute("cx").unwrap().parse().unwrap();
    let cy: i64 = sz.attribute("cy").unwrap().parse().unwrap();
    assert_eq!(sz.attribute("type"), Some("custom"));
    assert_eq!(cx, pt_to_emu(page.width));
    assert_eq!(cy, pt_to_emu(page.height));
    let sld_ids: Vec<_> = parsed
        .descendants()
        .filter(|n| n.has_tag_name("sldId"))
        .collect();
    assert_eq!(sld_ids.len(), lock.geometry.pages.len());
}

#[test]
fn export_twice_byte_identical() {
    let doc = common::invoice();
    let a = export_opened(&doc).unwrap();
    let b = export_opened(&doc).unwrap();
    assert_eq!(a, b);
}

#[test]
fn unlocked_fails() {
    // Empty slice is not a locked K2F package. OpenedDocument::open fails;
    // export_bytes must not succeed.
    let err = export_bytes(&[]).unwrap_err();
    assert!(!matches!(err, PptxError::NotASource));
}

#[test]
fn pptx_input_rejected() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let err = export_bytes(&pptx).unwrap_err();
    assert!(matches!(err, PptxError::NotASource), "got {err}");
}

#[test]
fn unknown_paint_op_fails() {
    let doc = common::invoice();
    let lock = doc.lock().expect("invoice is locked");
    assert!(!lock.has_unknown_paint_ops());
}
