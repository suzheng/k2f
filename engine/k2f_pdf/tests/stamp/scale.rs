mod helpers;

use k2f_pdf::{export_opened, PdfScale, PdfError};

#[test]
fn scale_4_doubles_stamp_width_vs_2() {
    let doc = helpers::glass_card();
    let pdf2 = export_opened(&doc, PdfScale::X2).unwrap();
    let pdf4 = export_opened(&doc, PdfScale::X4).unwrap();
    let w2 = stamp_image_width(&pdf2).expect("2x stamp width");
    let w4 = stamp_image_width(&pdf4).expect("4x stamp width");
    let ratio = w4 as f64 / w2 as f64;
    assert!(
        (ratio - 2.0).abs() < 0.05,
        "4x width {w4} vs 2x {w2} ratio {ratio}"
    );
}

#[test]
fn invalid_scale_rejected() {
    let err = PdfScale::from_f32(1.5).unwrap_err();
    assert!(matches!(err, PdfError::InvalidScale(_)));
}

fn stamp_image_width(pdf: &[u8]) -> Option<i64> {
    let doc = lopdf::Document::load_mem(pdf).ok()?;
    let mut widths = Vec::new();
    for obj in doc.objects.values() {
        if let lopdf::Object::Stream(s) = obj {
            if let Ok(w) = s.dict.get(b"Width") {
                if let lopdf::Object::Integer(n) = w {
                    if *n > 500 {
                        widths.push(*n);
                    }
                }
            }
        }
    }
    widths.into_iter().max()
}
