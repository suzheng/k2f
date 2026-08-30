mod common;

use k2f_core::PaintOp;
use k2f_pdf::{export_opened, parse_notes, PdfScale};

#[test]
fn invoice_pdf_boxes_and_images_match_lock() {
    let doc = common::invoice();
    let lock = doc.lock().unwrap();
    let pdf = export_opened(&doc, PdfScale::DEFAULT).unwrap();
    let parsed = lopdf::Document::load_mem(&pdf).unwrap();
    for (i, id) in parsed
        .get_pages()
        .values()
        .enumerate()
        .take(lock.geometry.pages.len())
    {
        let content = parsed.get_page_content(*id).unwrap();
        let (boxes, _, images) = parse_notes(&content);
        let plan = lock
            .render_plan
            .pages
            .iter()
            .find(|p| p.index == lock.geometry.pages[i].index)
            .unwrap();
        let mut expected_boxes = Vec::new();
        let mut expected_images = Vec::new();
        for op in &plan.ops {
            match op {
                PaintOp::DrawBox { rect, .. } | PaintOp::DrawTableReference { rect, .. } => {
                    expected_boxes.push([
                        rect.x.as_f64_pt(),
                        rect.y.as_f64_pt(),
                        rect.width.as_f64_pt(),
                        rect.height.as_f64_pt(),
                    ]);
                }
                PaintOp::DrawImage { rect, .. } => {
                    expected_images.push([
                        rect.x.as_f64_pt(),
                        rect.y.as_f64_pt(),
                        rect.width.as_f64_pt(),
                        rect.height.as_f64_pt(),
                    ]);
                }
                _ => {}
            }
        }
        assert_eq!(boxes.len(), expected_boxes.len(), "page {i} boxes");
        assert_eq!(images.len(), expected_images.len(), "page {i} images");
        for (a, b) in boxes.iter().zip(expected_boxes.iter()) {
            assert!(
                (0..4).all(|k| common::near(a[k], b[k])),
                "page {i} box {a:?} vs {b:?}"
            );
        }
        for (a, b) in images.iter().zip(expected_images.iter()) {
            assert!(
                (0..4).all(|k| common::near(a[k], b[k])),
                "page {i} image {a:?} vs {b:?}"
            );
        }
    }
}
