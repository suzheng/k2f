mod common;

use k2f_core::{
    Border, BorderEdge, BorderStyle, BoxDecoration, FillRef, PaintOp, Pt, Rect, Shadow, ShadowRef,
};
use k2f_idml::{export_opened, picture_from_draw, shapes_from_box, IdmlError, SpreadSpace};
use std::collections::BTreeMap;

fn all_xml(idml: &[u8]) -> String {
    let mut blob = String::new();
    for name in common::unzip_names(idml) {
        if !name.ends_with(".xml") {
            continue;
        }
        blob.push_str(&common::xml_in(idml, &name));
    }
    blob
}

fn item_xy(el: roxmltree::Node<'_, '_>) -> (f64, f64) {
    let tf = el.attribute("ItemTransform").expect("ItemTransform");
    let p: Vec<&str> = tf.split_whitespace().collect();
    assert!(p.len() >= 6, "ItemTransform {tf}");
    (p[4].parse().unwrap(), p[5].parse().unwrap())
}

fn sample_rect() -> Rect {
    Rect {
        x: Pt(0),
        y: Pt(0),
        width: Pt(100_000),
        height: Pt(50_000),
    }
}

#[test]
fn invoice_has_fillcolor_rectangle() {
    let idml = export_opened(&common::invoice()).unwrap();
    let mut found = false;
    for name in common::unzip_names(&idml) {
        if !name.starts_with("Spreads/") {
            continue;
        }
        let xml = common::xml_in(&idml, &name);
        let doc = roxmltree::Document::parse(&xml).unwrap();
        found |= doc.descendants().any(|n| {
            n.has_tag_name("Rectangle")
                && n.attribute("FillColor")
                    .is_some_and(|c| c.starts_with("Color/k2f_"))
        });
    }
    assert!(found, "expected a Spread Rectangle with Color/k2f_ fill");
}

#[test]
fn shape_center_near_lock_drawbox() {
    let doc = common::invoice();
    let lock = doc.lock().expect("locked");
    let plan = &lock.render_plan.pages[0];
    let (node_id, rect) = plan
        .ops
        .iter()
        .find_map(|op| match op {
            PaintOp::DrawBox {
                node_id,
                rect,
                decoration,
            } if decoration.blur.is_none()
                && !node_id.contains("::rule_")
                && match &decoration.shadow {
                    Some(ShadowRef::Inline(Shadow { layers })) => layers.is_empty(),
                    Some(ShadowRef::Ref(_)) => false,
                    None => true,
                } =>
            {
                Some((node_id.clone(), rect.clone()))
            }
            _ => None,
        })
        .expect("page 0 simple DrawBox");
    let space = SpreadSpace::new(lock.geometry.pages[0].width, lock.geometry.pages[0].height);
    let (want_x, want_y) = space.box_center(&rect);
    let idml = export_opened(&doc).unwrap();
    let xml = common::xml_in(&idml, "Spreads/Spread_k0.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let el = parsed
        .descendants()
        .find(|n| n.has_tag_name("Rectangle") && n.attribute("Name") == Some(node_id.as_str()))
        .unwrap_or_else(|| panic!("no Rectangle Name={node_id}"));
    let (tx, ty) = item_xy(el);
    assert!(
        (tx - want_x).abs() <= 0.02,
        "tx {tx} vs {want_x} for {node_id}"
    );
    assert!(
        (ty - want_y).abs() <= 0.02,
        "ty {ty} vs {want_y} for {node_id}"
    );
}

#[test]
fn invoice_logo_is_embedded_image() {
    let doc = common::invoice();
    let has_image = doc.lock().unwrap().render_plan.pages.iter().any(|p| {
        p.ops
            .iter()
            .any(|op| matches!(op, PaintOp::DrawImage { .. }))
    });
    assert!(has_image, "invoice fixture must contain DrawImage");
    let idml = export_opened(&doc).unwrap();
    let blob = all_xml(&idml);
    assert!(
        !blob.contains("file:"),
        "embedded images must not use file: links"
    );
    let mut found = false;
    for name in common::unzip_names(&idml) {
        if !name.starts_with("Spreads/") {
            continue;
        }
        let xml = common::xml_in(&idml, &name);
        let parsed = roxmltree::Document::parse(&xml).unwrap();
        for img in parsed.descendants().filter(|n| n.has_tag_name("Image")) {
            let contents = img
                .descendants()
                .find(|n| n.has_tag_name("Contents"))
                .and_then(|n| n.text())
                .unwrap_or("")
                .trim();
            if !contents.is_empty() {
                found = true;
                assert!(
                    xml.contains("<![CDATA["),
                    "{name} Image Contents must be CDATA"
                );
            }
        }
    }
    assert!(found, "expected Image with non-empty Contents CDATA");
}

#[test]
fn text_frames_still_present() {
    let idml = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&idml, "Spreads/Spread_k0.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    assert!(parsed.descendants().any(|n| n.has_tag_name("TextFrame")));
}

#[test]
fn no_k2f_raster_yet() {
    let blob = all_xml(&export_opened(&common::invoice()).unwrap());
    assert!(
        !blob.contains("k2f-raster:"),
        "step 3 must not emit effect rasters"
    );
}

#[test]
fn missing_image_is_write_error() {
    let err =
        picture_from_draw("pic", &sample_rect(), "nope.png", &BTreeMap::new(), 1).unwrap_err();
    match err {
        IdmlError::Write(msg) => assert!(msg.contains("missing image"), "{msg}"),
        other => panic!("expected Write, got {other:?}"),
    }
}

#[test]
fn svg_encodes_as_png() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><rect width="4" height="4" fill="#00f"/></svg>"##;
    let mut assets = BTreeMap::new();
    assets.insert("mark.svg".into(), svg.to_vec());
    let pic = picture_from_draw("pic", &sample_rect(), "mark.svg", &assets, 1).unwrap();
    assert_eq!(pic.ext, "png");
    assert!(
        pic.bytes.starts_with(&[0x89, b'P', b'N', b'G']),
        "svg must become PNG bytes"
    );
}

#[test]
fn unresolved_fill_ref_fails() {
    let dec = BoxDecoration {
        background: Some(FillRef::Ref("missing".into())),
        ..Default::default()
    };
    let err = shapes_from_box("card", &sample_rect(), &dec).unwrap_err();
    match err {
        IdmlError::Write(msg) => assert!(msg.contains("unresolved fill ref"), "{msg}"),
        other => panic!("expected Write, got {other:?}"),
    }
}

#[test]
fn partial_border_emits_edge_bars() {
    let dec = BoxDecoration {
        border: Some(Border {
            width_pt: 2_500,
            color: "#0C0C0E".into(),
            edges: vec![BorderEdge::Left],
            style: BorderStyle::Solid,
        }),
        ..Default::default()
    };
    let boxes = shapes_from_box("quote", &sample_rect(), &dec).unwrap();
    assert!(
        boxes.iter().any(|s| s.node_id.ends_with("::edge_left")),
        "expected ::edge_left, got {:?}",
        boxes.iter().map(|s| &s.node_id).collect::<Vec<_>>()
    );
    assert!(
        boxes.iter().all(|s| s.line_hex.is_none()),
        "partial border must not set four-side Stroke"
    );
}
