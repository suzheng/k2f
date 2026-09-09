mod common;

use k2f_core::{BoxDecoration, Fill, FillRef, PaintOp, Pt, Rect};
use k2f_docx::{export_opened, filter_chrome_ops, ChromeKeep};
use std::collections::HashSet;

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

fn glass_keep() -> ChromeKeep {
    ChromeKeep {
        effect_ids: HashSet::from(["card.glass".into()]),
        math_text_ids: HashSet::new(),
        keep_leading_page_background: true,
        ..Default::default()
    }
}

fn raster_count(xml: &str) -> usize {
    xml.matches("k2f-raster:").count()
}

fn w_t_blob(xml: &str) -> String {
    let doc = roxmltree::Document::parse(xml).unwrap();
    doc.descendants()
        .filter(|n| n.has_tag_name("t"))
        .filter_map(|n| n.text())
        .collect::<Vec<_>>()
        .join("")
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
    let filtered = filter_chrome_ops(&ops, &glass_keep());
    assert!(
        filtered
            .iter()
            .all(|op| !matches!(op, PaintOp::DrawText { .. })),
        "chrome ops must not keep body DrawText, got {filtered:?}"
    );
    assert!(filtered
        .iter()
        .all(|op| !matches!(op, PaintOp::DrawImage { .. })));
    assert!(
        !filtered
            .iter()
            .any(|op| matches!(op, PaintOp::DrawBox { node_id, .. } if node_id == "card.solid")),
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
fn filter_ops_drops_table_cell_paint_ops() {
    let ops = vec![
        draw_box("root::page_0::background"),
        draw_box("invoice.table::cell_0_0"),
        draw_text("invoice.table::cell_0_0"),
        backdrop("card.glass"),
        draw_box("card.glass"),
    ];
    let filtered = filter_chrome_ops(&ops, &glass_keep());
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
    let filtered = filter_chrome_ops(ops, &glass_keep());
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
    assert!(filtered.iter().any(|op| matches!(
        op,
        PaintOp::DrawText { node_id, .. } if node_id == "eq.frac"
    )));
    assert!(filtered.iter().any(|op| matches!(
        op,
        PaintOp::DrawBox { node_id, .. } if node_id == "eq.frac::rule_0"
    )));
    assert!(!filtered.iter().any(|op| matches!(
        op,
        PaintOp::DrawText { node_id, .. } if node_id == "title.math"
    )));
}

#[test]
fn glass_fixture_raster_plus_editable_title() {
    let Some(doc) = common::open_fixture("glass_title.K2F") else {
        return;
    };
    let docx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    assert!(
        w_t_blob(&xml).contains("UNIQUE_TOKEN_XYZ"),
        "title must remain editable w:t"
    );
    assert!(
        xml.contains("k2f-raster:card.glass"),
        "glass card must be a k2f-raster pic"
    );
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let raster_docpr_count = parsed
        .descendants()
        .filter(|n| n.has_tag_name("docPr"))
        .filter(|n| {
            common::local_attr(n, "name")
                .unwrap_or("")
                .starts_with("k2f-raster:")
        })
        .count();
    assert_eq!(
        raster_docpr_count, 1,
        "glass must emit one raster anchor (not duplicate blur+alpha slices)"
    );
    assert!(
        xml.contains("w:txbxContent") || xml.contains("wps:txbx"),
        "title text box must still be present"
    );
    let names = common::unzip_names(&docx);
    assert!(
        names.iter().any(|n| n == "word/media/raster1.png"),
        "dedicated raster media, got {names:?}"
    );
    let png = common::bytes_in(&docx, "word/media/raster1.png");
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    assert!(png.len() > 64);
    let glass = parsed
        .descendants()
        .find(|n| {
            n.has_tag_name("docPr")
                && common::local_attr(n, "name") == Some("k2f-raster:card.glass")
        })
        .and_then(|pr| pr.ancestors().find(|n| n.has_tag_name("anchor")))
        .expect("glass raster anchor");
    assert!(
        glass.descendants().any(|n| n.has_tag_name("blipFill")),
        "glass chrome must be a shape blipFill so Writer z-orders it with text"
    );
    assert!(
        glass.descendants().any(|n| n.has_tag_name("wsp")),
        "glass chrome must be wps:wsp, not pic:pic"
    );
    assert!(
        !glass.descendants().any(|n| n.has_tag_name("pic")),
        "pic:pic would paint above later text boxes in LibreOffice Writer"
    );
}

#[test]
fn math_is_blip_not_txbx() {
    let Some(doc) = common::open_fixture("math_frac.K2F") else {
        return;
    };
    let docx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    assert!(w_t_blob(&xml).contains("MATH_FIXTURE"));
    assert!(xml.contains("k2f-raster:eq.frac"));
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let math_as_txbx = parsed.descendants().any(|n| {
        n.has_tag_name("docPr")
            && common::local_attr(&n, "name") == Some("eq.frac")
            && n.ancestors()
                .any(|a| a.has_tag_name("txbxContent") || a.has_tag_name("txbx"))
    });
    assert!(!math_as_txbx, "math must not be an editable txbx");
    assert!(
        !xml.contains("name=\"eq.frac::rule_0\""),
        "fraction rule must merge into the math raster"
    );
}

#[test]
fn invoice_zero_raster_if_no_effects() {
    let docx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    assert_eq!(
        raster_count(&xml),
        0,
        "invoice has no blur/shadow/gradient; must not emit k2f-raster pics"
    );
}
