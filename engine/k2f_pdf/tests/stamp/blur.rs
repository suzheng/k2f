mod helpers;

use k2f_pdf::{export_opened, PdfExportOptions, PdfScale};

#[test]
fn glass_card_exports_with_stamp() {
    let doc = helpers::glass_card();
    let pdf = export_opened(
        &doc,
        PdfExportOptions::new(PdfScale::DEFAULT).with_trust_pack(),
    )
    .unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    assert!(
        pdf.windows(16).any(|w| w == b"appearance_hash="),
        "trust-pack PDF must identify K2F source"
    );
    assert_eq!(
        lopdf::Document::load_mem(&pdf).unwrap().get_pages().len(),
        2,
        "content + verify pages"
    );
}

#[path = "../common/mod.rs"]
mod common;

#[test]
fn stamp_page_keeps_invisible_selectable_text() {
    let doc = helpers::glass_card();
    let pdf = export_opened(&doc, PdfScale::DEFAULT).unwrap();
    assert!(
        common::extract::page_content_has(&pdf, 0, "3 Tr"),
        "stamp pages must keep invisible selectable text"
    );
    let sample = doc
        .text_layer(0)
        .into_iter()
        .map(|s| s.text)
        .find(|t| t.chars().count() >= 4)
        .expect("glass card text layer");
    let extracted = common::extract::extract_pdf_text(&pdf);
    assert!(
        extracted.contains(&sample),
        "extracted {extracted:?} must contain {sample:?}"
    );
}
