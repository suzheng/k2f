mod common;

use base64::Engine;
use k2f_core::{BoxDecoration, Fill, FillRef, PaintOp, Pt, Rect};
use k2f_idml::{export_bytes, export_opened, filter_chrome_ops, ChromeKeep, IdmlError};
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

fn all_xml(idml: &[u8]) -> String {
    let mut blob = String::new();
    for name in common::unzip_names(idml) {
        if name.ends_with(".xml") {
            blob.push_str(&common::xml_in(idml, &name));
        }
    }
    blob
}

fn story_contents(idml: &[u8]) -> String {
    let mut blob = String::new();
    for name in common::unzip_names(idml) {
        if name.starts_with("Stories/") && name.ends_with(".xml") {
            blob.push_str(&common::xml_in(idml, &name));
        }
    }
    blob
}

fn raster_name_count(xml: &str) -> usize {
    xml.matches("k2f-raster:").count()
}

fn raster_rectangle_names(idml: &[u8]) -> Vec<String> {
    let spread0 = common::xml_in(idml, "Spreads/Spread_k0.xml");
    let parsed = roxmltree::Document::parse(&spread0).unwrap();
    parsed
        .descendants()
        .filter(|n| n.has_tag_name("Rectangle"))
        .filter_map(|n| n.attribute("Name"))
        .filter(|name| name.starts_with("k2f-raster:"))
        .map(str::to_string)
        .collect()
}

fn first_raster_png(idml: &[u8]) -> Vec<u8> {
    let spread0 = common::xml_in(idml, "Spreads/Spread_k0.xml");
    let parsed = roxmltree::Document::parse(&spread0).unwrap();
    let rect = parsed
        .descendants()
        .find(|n| {
            n.has_tag_name("Rectangle")
                && n.attribute("Name")
                    .map(|name| name.starts_with("k2f-raster:"))
                    .unwrap_or(false)
        })
        .expect("k2f-raster Rectangle");
    let b64 = rect
        .descendants()
        .find(|n| n.has_tag_name("Contents"))
        .and_then(|n| n.text())
        .expect("raster Image Contents");
    let compact: String = b64.chars().filter(|c| !c.is_whitespace()).collect();
    base64::engine::general_purpose::STANDARD
        .decode(compact)
        .expect("valid base64 PNG payload")
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
    assert!(
        filtered
            .iter()
            .all(|op| !matches!(op, PaintOp::DrawImage { .. })),
        "chrome ops must not keep DrawImage"
    );
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
fn glass_fixture_has_raster_and_editable_title() {
    let Some(doc) = common::open_fixture("glass_title.K2F") else {
        return;
    };
    let idml = export_opened(&doc).unwrap();
    let stories = story_contents(&idml);
    assert!(
        stories.contains("UNIQUE_TOKEN_XYZ"),
        "title must remain editable Story Content"
    );
    let xml = all_xml(&idml);
    assert!(
        xml.contains("k2f-raster:card.glass"),
        "glass card must be a k2f-raster Image, snippet: {}",
        &xml[..xml.len().min(800)]
    );
    assert!(
        xml.contains("<TextFrame"),
        "title text frame must still be present"
    );
    assert_eq!(
        raster_rectangle_names(&idml),
        vec!["k2f-raster:card.glass".to_string()],
        "blur+alpha glass must merge into one raster slice"
    );
    let png = first_raster_png(&idml);
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "raster must embed PNG bytes");
    assert!(png.len() > 64, "raster PNG should be non-trivial");
}

#[test]
fn export_glass_still_byte_identical() {
    let Some(doc) = common::open_fixture("glass_title.K2F") else {
        return;
    };
    let a = export_opened(&doc).unwrap();
    let b = export_opened(&doc).unwrap();
    assert_eq!(a, b, "effect raster export must stay deterministic");
}

#[test]
fn math_is_picture_not_story_text() {
    let Some(doc) = common::open_fixture("math_frac.K2F") else {
        return;
    };
    let idml = export_opened(&doc).unwrap();
    let stories = story_contents(&idml);
    assert!(
        stories.contains("MATH_FIXTURE"),
        "non-math heading stays as editable text"
    );
    assert!(
        !stories.contains(r"\frac{a}{b}") && !stories.contains("frac{a}{b}"),
        "math source must not appear in Story Content"
    );
    let xml = all_xml(&idml);
    assert!(
        xml.contains("k2f-raster:eq.frac"),
        "math node must be a k2f-raster Image"
    );
    let spread0 = common::xml_in(&idml, "Spreads/Spread_k0.xml");
    let parsed = roxmltree::Document::parse(&spread0).unwrap();
    let math_as_textframe = parsed
        .descendants()
        .any(|n| n.has_tag_name("TextFrame") && n.attribute("Name") == Some("eq.frac"));
    assert!(!math_as_textframe, "math must not be an editable TextFrame");
    assert!(
        !xml.contains("Name=\"eq.frac::rule_0\""),
        "fraction rule must merge into the math raster, not a native shape"
    );
}

#[test]
fn simple_invoice_has_zero_k2f_raster_if_no_blur() {
    let idml = export_opened(&common::invoice()).unwrap();
    let xml = all_xml(&idml);
    assert_eq!(
        raster_name_count(&xml),
        0,
        "invoice has no blur/shadow/gradient/alpha; must not emit k2f-raster pics"
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
    assert!(matches!(err, IdmlError::UnknownOp), "got {err}");
}
