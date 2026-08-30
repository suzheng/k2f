mod common;

use k2f_paint::{OpenedDocument, OFFICIAL_PNG_SCALE};

#[test]
fn card_glass_variant_paints_via_raster_not_pdf() {
    let png = |variant: &str| {
        let mut ed = common::open("invoice");
        common::insert_text(&mut ed, "root", "card1", "body", "Named glass card");
        ed.set_role("card1", "card", Some(variant)).unwrap();
        let bytes = ed.save_bytes().unwrap();
        OpenedDocument::open(&bytes)
            .unwrap()
            .render_page(0, OFFICIAL_PNG_SCALE)
            .unwrap()
    };
    let glass = png("glass");
    let flat = png("flat");
    assert!(glass.len() > 100);
    assert_ne!(
        glass, flat,
        "role card variant glass must raster differently from flat"
    );
}
