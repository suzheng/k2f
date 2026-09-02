mod common;

use k2f_core::{BoxDecoration, Fill, FillRef, PaintOp, Pt, Rect};
use k2f_pptx::{export_bytes, export_opened, filter_chrome_ops, ChromeKeep, PptxError};
use std::collections::HashSet;

fn slide_xml_names(pptx: &[u8]) -> Vec<String> {
    common::unzip_names(pptx)
        .into_iter()
        .filter(|n| {
            n.starts_with("ppt/slides/slide") && n.ends_with(".xml") && !n.contains("_rels")
        })
        .collect()
}

fn all_slide_xml(pptx: &[u8]) -> String {
    slide_xml_names(pptx)
        .iter()
        .map(|n| common::xml_in(pptx, n))
        .collect::<Vec<_>>()
        .join("\n")
}

fn dummy_rect() -> Rect {
    Rect {
        x: Pt(0),
        y: Pt(0),
        width: Pt(100_000),
        height: Pt(40_000),
    }
}

fn draw_text(id: &str) -> PaintOp {
    PaintOp::DrawText {
        node_id: id.to_string(),
        rect: dummy_rect(),
        runs: vec![],
    }
}

fn draw_box(id: &str) -> PaintOp {
    PaintOp::DrawBox {
        node_id: id.to_string(),
        rect: dummy_rect(),
        decoration: BoxDecoration {
            background: Some(FillRef::Inline(Fill::Solid {
                color: "#FFFFFF".into(),
            })),
            ..Default::default()
        },
    }
}

fn backdrop(id: &str) -> PaintOp {
    PaintOp::BackdropBlur {
        node_id: id.to_string(),
        rect: dummy_rect(),
        radius_pt: 8_000,
        corner_radius_pt: Some(4_000),
    }
}

fn raster_pic_count(xml: &str) -> usize {
    xml.matches("k2f-raster:").count()
}

#[test]
fn filter_ops_drops_draw_text() {
    let ops = vec![
        draw_box("root::page_0::background"),
        draw_text("title.unique"),
        draw_box("card.solid"),
        backdrop("card.glass"),
        draw_box("card.glass"),
        draw_text("card.glass"),
        PaintOp::DrawImage {
            node_id: "logo".into(),
            rect: dummy_rect(),
            src: "assets/images/logo.png".into(),
        },
    ];
    let keep = ChromeKeep {
        effect_ids: HashSet::from(["card.glass".into()]),
        math_text_ids: HashSet::new(),
        keep_leading_page_background: true,
        ..Default::default()
    };
    let filtered = filter_chrome_ops(&ops, &keep);
    assert!(
        filtered
            .iter()
            .all(|op| !matches!(op, PaintOp::DrawText { .. })),
        "chrome ops must not keep body DrawText, got {filtered:?}"
    );
    assert!(
        filtered
            .iter()
            .all(|op| !matches!(op, PaintOp::DrawImage { .. })),
        "chrome ops must not keep DrawImage"
    );
    assert!(
        !filtered.iter().any(
            |op| matches!(op, PaintOp::DrawBox { node_id, .. } if node_id == "card.solid")
        ),
        "native-simple boxes must not bake into glass chrome"
    );
    assert!(filtered
        .iter()
        .any(|op| matches!(op, PaintOp::BackdropBlur { node_id, .. } if node_id == "card.glass")));
    assert!(filtered
        .iter()
        .any(|op| matches!(op, PaintOp::DrawBox { node_id, .. } if node_id == "card.glass")));
    assert!(filtered.iter().any(
        |op| matches!(op, PaintOp::DrawBox { node_id, .. } if node_id == "root::page_0::background")
    ));
}

#[test]
fn filter_ops_keeps_math_chrome_from_fixture() {
    let Some(doc) = common::open_fixture("math_frac.K2F") else {
        return;
    };
    let lock = doc.lock().expect("locked");
    let ops = &lock.render_plan.pages[0].ops;
    let keep = ChromeKeep {
        effect_ids: HashSet::from(["eq.frac".into()]),
        math_text_ids: HashSet::from(["eq.frac".into()]),
        keep_leading_page_background: true,
        ..Default::default()
    };
    let filtered = filter_chrome_ops(ops, &keep);
    assert!(
        filtered.iter().any(|op| matches!(
            op,
            PaintOp::DrawText { node_id, .. } if node_id == "eq.frac"
        )),
        "math chrome must keep the math DrawText"
    );
    assert!(
        filtered.iter().any(|op| matches!(
            op,
            PaintOp::DrawBox { node_id, .. } if node_id == "eq.frac::rule_0"
        )),
        "math chrome must keep fraction rule boxes"
    );
    assert!(
        !filtered.iter().any(|op| matches!(
            op,
            PaintOp::DrawText { node_id, .. } if node_id == "title.math"
        )),
        "math chrome must drop unrelated body text"
    );
}

#[test]
fn filter_ops_drops_table_cell_paint_ops() {
    let ops = vec![
        draw_box("root::page_0::background"),
        draw_box("invoice.table::cell_0_0"),
        draw_text("invoice.table::cell_0_0"),
        backdrop("card.glass"),
        draw_box("card.glass"),
    ];
    let keep = ChromeKeep {
        effect_ids: HashSet::from(["card.glass".into()]),
        math_text_ids: HashSet::new(),
        keep_leading_page_background: true,
        ..Default::default()
    };
    let filtered = filter_chrome_ops(&ops, &keep);
    assert!(
        !filtered.iter().any(|op| matches!(
            op,
            PaintOp::DrawText { node_id, .. } if node_id.starts_with("invoice.table::")
        )),
        "harvested table cell text must not repaint into effect chrome"
    );
    assert!(
        !filtered.iter().any(|op| matches!(
            op,
            PaintOp::DrawBox { node_id, .. } if node_id.starts_with("invoice.table::")
        )),
        "harvested table cell boxes must not bake into effect chrome"
    );
}

#[test]
fn filter_ops_excludes_body_text_on_glass_fixture() {
    let Some(doc) = common::open_fixture("glass_title.K2F") else {
        return;
    };
    let lock = doc.lock().expect("locked");
    let ops = &lock.render_plan.pages[0].ops;
    let keep = ChromeKeep {
        effect_ids: HashSet::from(["card.glass".into()]),
        math_text_ids: HashSet::new(),
        keep_leading_page_background: true,
        ..Default::default()
    };
    let filtered = filter_chrome_ops(ops, &keep);
    assert!(
        !filtered.iter().any(|op| matches!(
            op,
            PaintOp::DrawText { node_id, .. } if node_id == "title.unique"
        )),
        "chrome filter must drop UNIQUE_TOKEN title DrawText"
    );
    assert!(
        !filtered
            .iter()
            .any(|op| matches!(op, PaintOp::DrawText { .. })),
        "glass chrome must drop all DrawText"
    );
}

#[test]
fn glass_fixture_emits_raster_png_media() {
    let Some(doc) = common::open_fixture("glass_title.K2F") else {
        return;
    };
    let pptx = export_opened(&doc).unwrap();
    let names = common::unzip_names(&pptx);
    assert!(
        names.iter().any(|n| n == "ppt/media/raster1.png"),
        "glass export must write a dedicated raster media file, got {names:?}"
    );
    let png = common::bytes_in(&pptx, "ppt/media/raster1.png");
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "raster media must be PNG");
    assert!(png.len() > 64, "raster PNG should be non-trivial");
}

#[test]
fn glass_fixture_has_both_raster_pic_and_editable_title() {
    let Some(doc) = common::open_fixture("glass_title.K2F") else {
        return;
    };
    let pptx = export_opened(&doc).unwrap();
    let xml = all_slide_xml(&pptx);
    assert!(
        xml.contains("UNIQUE_TOKEN_XYZ"),
        "title must remain editable text, not only pixels"
    );
    assert!(
        xml.contains("<a:t>UNIQUE_TOKEN_XYZ</a:t>") || xml.contains("UNIQUE_TOKEN_XYZ"),
        "title should appear in a:t"
    );
    assert!(
        xml.contains("k2f-raster:card.glass"),
        "glass card must be a k2f-raster pic, xml snippet: {}",
        &xml[..xml.len().min(800)]
    );
    assert!(
        xml.contains("txBox=\"1\""),
        "title text box must still be present"
    );
}

#[test]
fn math_is_picture_not_txbody() {
    let Some(doc) = common::open_fixture("math_frac.K2F") else {
        return;
    };
    let pptx = export_opened(&doc).unwrap();
    let xml = all_slide_xml(&pptx);
    assert!(
        xml.contains("MATH_FIXTURE"),
        "non-math heading stays as text"
    );
    assert!(
        xml.contains("k2f-raster:eq.frac"),
        "math node must be a raster pic"
    );
    let slide1 = common::xml_in(&pptx, "ppt/slides/slide1.xml");
    let parsed = roxmltree::Document::parse(&slide1).unwrap();
    let math_as_textbox = parsed.descendants().any(|n| {
        n.has_tag_name("sp")
            && n.descendants()
                .any(|c| c.has_tag_name("cNvPr") && c.attribute("name") == Some("eq.frac"))
            && n.descendants()
                .any(|c| c.has_tag_name("cNvSpPr") && c.attribute("txBox") == Some("1"))
    });
    assert!(!math_as_textbox, "math must not be an editable txBody");
    assert!(
        !xml.contains("name=\"eq.frac::rule_0\""),
        "fraction rule must merge into the math raster, not a native shape"
    );
}

#[test]
fn simple_invoice_has_zero_k2f_raster_if_no_blur() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let xml = all_slide_xml(&pptx);
    assert_eq!(
        raster_pic_count(&xml),
        0,
        "invoice has no blur/shadow/gradient; must not emit k2f-raster pics"
    );
}

#[test]
fn unknown_op_still_fails() {
    let src = common::invoice_bytes();
    let lock = common::xml_in(&src, "document.K2F.lock");
    let rp = lock.find("\"render_plan\"").expect("render_plan");
    let needle = "\"ops\":[";
    let rel = lock[rp..].find(needle).expect("ops");
    let at = rp + rel + needle.len();
    let mut mutated = String::with_capacity(lock.len() + 32);
    mutated.push_str(&lock[..at]);
    mutated.push_str("{\"type\":\"draw_unicorn\"},");
    mutated.push_str(&lock[at..]);
    let k2f = common::rewrite_zip(&src, |name, data| {
        if name == "document.K2F.lock" {
            Some(mutated.as_bytes().to_vec())
        } else {
            Some(data)
        }
    });
    let err = export_bytes(&k2f).unwrap_err();
    assert!(matches!(err, PptxError::UnknownOp), "got {err}");
}
