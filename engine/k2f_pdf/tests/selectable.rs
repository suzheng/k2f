mod common;

use k2f_pdf::{export_opened, PdfScale};

#[test]
fn invoice_pdf_keeps_outlines_and_exposes_selectable_text() {
    let doc = common::invoice();
    let sample = doc
        .text_layer(0)
        .into_iter()
        .map(|s| s.text)
        .find(|t| t.chars().count() >= 4)
        .expect("invoice text layer has a real phrase");
    let pdf = export_opened(&doc, PdfScale::DEFAULT).unwrap();
    assert!(
        common::extract::page_content_has(&pdf, 0, "f"),
        "visual layer must still fill glyph outlines"
    );
    assert!(
        common::extract::page_content_has(&pdf, 0, "3 Tr"),
        "invisible text layer must set render mode 3"
    );
    assert!(
        common::extract::page_has_font(&pdf, 0),
        "page resources must list a Font"
    );
    let extracted = common::extract::extract_pdf_text(&pdf);
    assert!(
        extracted.contains(&sample),
        "extracted {extracted:?} must contain text-layer sample {sample:?}"
    );
}
