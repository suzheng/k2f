mod common;

use k2f_core::PaintOp;
use k2f_paint::{decode_raster, letterbox_rect, lookup_image};
use k2f_pptx::{export_opened, pt_to_emu};

fn slide_xml_names(pptx: &[u8]) -> Vec<String> {
    common::unzip_names(pptx)
        .into_iter()
        .filter(|n| {
            n.starts_with("ppt/slides/slide") && n.ends_with(".xml") && !n.contains("_rels")
        })
        .collect()
}

fn is_txbox(sp: roxmltree::Node<'_, '_>) -> bool {
    sp.descendants()
        .any(|n| n.has_tag_name("cNvSpPr") && n.attribute("txBox") == Some("1"))
}

fn sp_named<'a, 'b>(
    doc: &'a roxmltree::Document<'b>,
    name: &str,
) -> Option<roxmltree::Node<'a, 'b>> {
    doc.descendants().find(|n| {
        n.has_tag_name("sp")
            && n.descendants()
                .any(|c| c.has_tag_name("cNvPr") && c.attribute("name") == Some(name))
    })
}

fn off_xy(node: roxmltree::Node<'_, '_>) -> (i64, i64) {
    let off = node
        .descendants()
        .find(|n| n.has_tag_name("off"))
        .expect("a:off");
    (
        off.attribute("x").unwrap().parse().unwrap(),
        off.attribute("y").unwrap().parse().unwrap(),
    )
}

#[test]
fn invoice_has_solid_shapes() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let mut found_shape_fill = false;
    for name in slide_xml_names(&pptx) {
        let xml = common::xml_in(&pptx, &name);
        let parsed = roxmltree::Document::parse(&xml).unwrap();
        for sp in parsed.descendants().filter(|n| n.has_tag_name("sp")) {
            if is_txbox(sp) {
                continue;
            }
            let sp_pr = sp.descendants().find(|n| n.has_tag_name("spPr"));
            if let Some(pr) = sp_pr {
                if pr.descendants().any(|n| n.has_tag_name("solidFill")) {
                    found_shape_fill = true;
                }
            }
        }
    }
    assert!(
        found_shape_fill,
        "expected a non-textbox p:sp with a:solidFill (card/background)"
    );
}

#[test]
fn shape_rect_near_lock_drawbox() {
    let doc = common::invoice();
    let lock = doc.lock().expect("locked");
    let plan = lock
        .render_plan
        .pages
        .iter()
        .find(|p| p.index == 0)
        .expect("page 0 plan");
    let absorbed = common::native_table_member_ids(&doc);
    let (node_id, rect) = plan
        .ops
        .iter()
        .find_map(|op| match op {
            PaintOp::DrawBox {
                node_id,
                rect,
                decoration,
            } if decoration.shadow.is_none()
                && decoration.blur.is_none()
                && !node_id.contains("::rule_")
                && !absorbed.contains(node_id) =>
            {
                Some((node_id.clone(), rect.clone()))
            }
            _ => None,
        })
        .expect("page 0 has a simple DrawBox");

    let pptx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&pptx, "ppt/slides/slide1.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let sp = sp_named(&parsed, &node_id).unwrap_or_else(|| panic!("no shape named {node_id}"));
    assert!(
        !is_txbox(sp),
        "{node_id} should be a filled shape, not txBox"
    );
    let (x, y) = off_xy(sp);
    assert!(
        (x - pt_to_emu(rect.x)).abs() <= 1,
        "x {x} vs {} for {node_id}",
        pt_to_emu(rect.x)
    );
    assert!(
        (y - pt_to_emu(rect.y)).abs() <= 1,
        "y {y} vs {} for {node_id}",
        pt_to_emu(rect.y)
    );
}

#[test]
fn images_are_separate_media_if_present() {
    let doc = common::invoice();
    let lock = doc.lock().expect("locked");
    let (node_id, rect, src) = lock.render_plan.pages[0]
        .ops
        .iter()
        .find_map(|op| match op {
            PaintOp::DrawImage { node_id, rect, src } => {
                Some((node_id.clone(), rect.clone(), src.clone()))
            }
            _ => None,
        })
        .expect("invoice page 0 has DrawImage");
    let asset = lookup_image(doc.assets(), &src).expect("logo bytes");
    let img = decode_raster(asset).expect("logo decode");
    let dest = letterbox_rect(img.width(), img.height(), &rect).unwrap_or(rect.clone());

    let pptx = export_opened(&doc).unwrap();
    let names = common::unzip_names(&pptx);
    let media: Vec<_> = names
        .iter()
        .filter(|n| n.starts_with("ppt/media/"))
        .cloned()
        .collect();
    assert!(
        !media.is_empty(),
        "invoice logo must be a separate media part, got {names:?}"
    );
    let media_bytes = common::bytes_in(&pptx, &media[0]);
    assert_eq!(
        media_bytes, *asset,
        "media must be the original image bytes"
    );

    let xml = common::xml_in(&pptx, "ppt/slides/slide1.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let pic = parsed
        .descendants()
        .find(|n| {
            n.has_tag_name("pic")
                && n.descendants().any(|c| {
                    c.has_tag_name("cNvPr") && c.attribute("name") == Some(node_id.as_str())
                })
        })
        .unwrap_or_else(|| panic!("missing p:pic named {node_id}"));
    let (x, y) = off_xy(pic);
    assert!(
        (x - pt_to_emu(dest.x)).abs() <= 1,
        "pic x {x} vs {}",
        pt_to_emu(dest.x)
    );
    assert!(
        (y - pt_to_emu(dest.y)).abs() <= 1,
        "pic y {y} vs {}",
        pt_to_emu(dest.y)
    );
    let ext = pic
        .descendants()
        .find(|n| n.has_tag_name("ext"))
        .expect("a:ext");
    let cx: i64 = ext.attribute("cx").unwrap().parse().unwrap();
    let cy: i64 = ext.attribute("cy").unwrap().parse().unwrap();
    assert!(
        (cx - pt_to_emu(dest.width)).abs() <= 1,
        "pic cx {cx} vs {}",
        pt_to_emu(dest.width)
    );
    assert!(
        (cy - pt_to_emu(dest.height)).abs() <= 1,
        "pic cy {cy} vs {}",
        pt_to_emu(dest.height)
    );
}

#[test]
fn text_still_txbody() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&pptx, "ppt/slides/slide1.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    assert!(parsed.descendants().any(|n| n.has_tag_name("txBody")));
    assert!(parsed
        .descendants()
        .any(|n| n.has_tag_name("t") && n.text().is_some()));
}

#[test]
fn no_ppt_media_from_effects_yet() {
    let pptx = export_opened(&common::invoice()).unwrap();
    for name in slide_xml_names(&pptx) {
        let xml = common::xml_in(&pptx, &name);
        assert!(
            !xml.contains("k2f-raster:"),
            "{name} must not emit effect rasters in step 3"
        );
        let parsed = roxmltree::Document::parse(&xml).unwrap();
        let bg = parsed
            .descendants()
            .find(|n| n.has_tag_name("bgPr"))
            .expect("p:bgPr");
        assert!(
            bg.descendants().any(|n| n.has_tag_name("solidFill")),
            "{name} slide background must stay solidFill, not a full-page blip"
        );
        assert!(
            !bg.descendants().any(|n| n.has_tag_name("blipFill")),
            "{name} slide background must not be a blip"
        );
    }
}

#[test]
fn slide_bg_and_shape_fills_pin_exact_black_white() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&pptx, "ppt/slides/slide1.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let bg = parsed
        .descendants()
        .find(|n| n.has_tag_name("bgPr"))
        .expect("p:bgPr");
    let bg_fill = bg
        .descendants()
        .find(|n| n.has_tag_name("srgbClr"))
        .and_then(|n| n.attribute("val"))
        .expect("slide bg srgbClr");
    assert_ne!(bg_fill, "FFFFFF");
    assert_ne!(bg_fill, "000000");
    let shape_fills: Vec<_> = parsed
        .descendants()
        .filter(|n| n.has_tag_name("sp") && !is_txbox(*n))
        .filter_map(|sp| {
            sp.children()
                .find(|c| c.has_tag_name("spPr"))
                .and_then(|pr| {
                    pr.descendants()
                        .find(|n| n.has_tag_name("solidFill"))
                        .and_then(|sf| {
                            sf.descendants()
                                .find(|n| n.has_tag_name("srgbClr"))
                                .and_then(|n| n.attribute("val"))
                        })
                })
        })
        .collect();
    assert!(
        !shape_fills.is_empty(),
        "expected at least one non-textbox shape fill"
    );
    assert!(
        !shape_fills
            .iter()
            .any(|c| *c == "000000" || *c == "FFFFFF"),
        "shape RGB 000000/FFFFFF remaps in Dark Mode, got {shape_fills:?}"
    );
}

#[test]
fn paint_plan_z_order_preserved_on_invoice_slide1() {
    let doc = common::invoice();
    let pptx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&pptx, "ppt/slides/slide1.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let sp_tree = parsed
        .descendants()
        .find(|n| n.has_tag_name("spTree"))
        .expect("spTree");
    let names: Vec<String> = sp_tree
        .children()
        .filter(|n| n.has_tag_name("sp") || n.has_tag_name("pic"))
        .filter_map(|n| {
            n.descendants()
                .find(|c| c.has_tag_name("cNvPr"))
                .and_then(|c| c.attribute("name"))
                .map(str::to_string)
        })
        .collect();
    assert_eq!(
        names,
        vec![
            "invoice_root::page_0::background".to_string(),
            "invoice.logo".to_string(),
            "invoice.header".to_string(),
            "invoice.details".to_string(),
            "running.footer.pages".to_string(),
        ]
    );
}

#[test]
fn content_types_includes_media_extension() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let ct = common::xml_in(&pptx, "[Content_Types].xml");
    assert!(
        ct.contains(r#"<Default Extension="png" ContentType="image/png"/>"#),
        "invoice logo must register png in [Content_Types].xml"
    );
}

#[test]
fn contract_rounded_warning_is_round_rect() {
    let pptx = export_opened(&common::contract()).unwrap();
    let xml = common::xml_in(&pptx, "ppt/slides/slide1.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let sp = sp_named(&parsed, "contract.warning").expect("warning shape");
    let geom = sp
        .descendants()
        .find(|n| n.has_tag_name("prstGeom"))
        .expect("prstGeom");
    assert_eq!(geom.attribute("prst"), Some("roundRect"));
}
