mod common;

use k2f_docx::{
    export_bytes, export_opened, millipt_to_emu, millipt_to_twips, pt_to_twips, DocxError,
};

#[test]
fn millipt_to_emu_a4_and_widescreen() {
    assert_eq!(millipt_to_emu(595_000), 7_556_500);
    assert_eq!(millipt_to_emu(842_000), 10_693_400);
    assert_eq!(millipt_to_emu(960_000), 12_192_000);
    assert_eq!(millipt_to_emu(540_000), 6_858_000);
}

#[test]
fn millipt_to_twips_a4_and_widescreen() {
    assert_eq!(millipt_to_twips(595_000), 11_900);
    assert_eq!(millipt_to_twips(842_000), 16_840);
    assert_eq!(millipt_to_twips(960_000), 19_200);
    assert_eq!(millipt_to_twips(540_000), 10_800);
}

#[test]
fn export_invoice_is_zip_with_document_xml() {
    let docx = export_opened(&common::invoice()).unwrap();
    assert!(docx.starts_with(b"PK"), "docx must be a zip");
    let names = common::unzip_names(&docx);
    assert!(
        names.iter().any(|n| n == "word/document.xml"),
        "missing word/document.xml in {names:?}"
    );
    for required in [
        "[Content_Types].xml",
        "_rels/.rels",
        "docProps/core.xml",
        "docProps/app.xml",
        "word/_rels/document.xml.rels",
        "word/styles.xml",
        "word/settings.xml",
        "word/fontTable.xml",
        "word/webSettings.xml",
        "word/theme/theme1.xml",
    ] {
        assert!(
            names.iter().any(|n| n == required),
            "missing {required} in {names:?}"
        );
    }
}

#[test]
fn theme_pins_dk1_lt1_to_srgb_not_system_window_colors() {
    let docx = export_opened(&common::invoice()).unwrap();
    let theme = common::xml_in(&docx, "word/theme/theme1.xml");
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
    let ct = common::xml_in(&docx, "[Content_Types].xml");
    assert!(
        ct.contains("/word/theme/theme1.xml"),
        "Content_Types must register the theme part, got {ct}"
    );
    let rels = common::xml_in(&docx, "word/_rels/document.xml.rels");
    assert!(
        rels.contains("/relationships/theme") && rels.contains("theme/theme1.xml"),
        "document.xml.rels must point at the theme, got {rels}"
    );
}

#[test]
fn paper_background_is_explicit_rgb_not_automatic() {
    let docx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    assert!(
        xml.contains("<w:background ") && !xml.contains(r#"w:color="FFFFFF""#),
        "Word Dark Mode remaps automatic/white paper; got {xml}"
    );
    assert!(
        xml.contains("invoice_root::page_0::background") && xml.contains(r#"behindDoc="1""#),
        "page paper must be a behindDoc drawing so Dark Mode cannot drop the wash, got {xml}"
    );
    let settings = common::xml_in(&docx, "word/settings.xml");
    assert!(
        settings.contains("<w:displayBackgroundShape/>"),
        "settings must show the paper color, got {settings}"
    );
}

#[test]
fn page_breaks_match_lock_page_count() {
    let doc = common::invoice();
    let docx = export_opened(&doc).unwrap();
    let lock = doc.lock().expect("invoice is locked");
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let breaks = parsed
        .descendants()
        .filter(|n| n.has_tag_name("br") && common::local_attr(n, "type") == Some("page"))
        .count();
    assert_eq!(breaks, lock.geometry.pages.len().saturating_sub(1));
}

#[test]
#[allow(non_snake_case)]
fn pgSz_matches_lock() {
    let doc = common::invoice();
    let docx = export_opened(&doc).unwrap();
    let lock = doc.lock().expect("invoice is locked");
    let page = &lock.geometry.pages[0];
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let sz = parsed
        .descendants()
        .find(|n| n.has_tag_name("pgSz"))
        .expect("pgSz");
    let w: i64 = common::local_attr(&sz, "w").unwrap().parse().unwrap();
    let h: i64 = common::local_attr(&sz, "h").unwrap().parse().unwrap();
    assert_eq!(w, pt_to_twips(page.width));
    assert_eq!(h, pt_to_twips(page.height));
}

#[test]
#[allow(non_snake_case)]
fn pgMar_all_zero() {
    let docx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let mar = parsed
        .descendants()
        .find(|n| n.has_tag_name("pgMar"))
        .expect("pgMar");
    for name in [
        "top", "right", "bottom", "left", "header", "footer", "gutter",
    ] {
        let v = common::local_attr(&mar, name).unwrap_or("missing");
        assert_eq!(v, "0", "pgMar {name}={v}");
    }
}

#[test]
fn export_twice_byte_identical() {
    let doc = common::invoice();
    let a = export_opened(&doc).unwrap();
    let b = export_opened(&doc).unwrap();
    assert_eq!(a, b);
}

#[test]
fn docx_input_rejected() {
    let docx = export_opened(&common::invoice()).unwrap();
    let err = export_bytes(&docx).unwrap_err();
    assert!(matches!(err, DocxError::NotASource), "got {err}");
}

#[test]
fn unlocked_fails() {
    let err = export_bytes(&[]).unwrap_err();
    assert!(!matches!(err, DocxError::NotASource));
}

#[test]
fn unknown_paint_op_fails() {
    let doc = common::invoice();
    let lock = doc.lock().expect("invoice is locked");
    assert!(!lock.has_unknown_paint_ops());
}
