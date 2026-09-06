mod common;

use k2f_core::PaintOp;
use k2f_docx::{export_opened, pt_to_emu};
use k2f_paint::{decode_raster, letterbox_rect, lookup_image};

fn is_textbox_wsp(wsp: roxmltree::Node<'_, '_>) -> bool {
    wsp.descendants()
        .any(|n| n.has_tag_name("cNvSpPr") && common::local_attr(&n, "txBox") == Some("1"))
}

fn simple_drawbox(doc: &k2f_paint::OpenedDocument) -> (String, k2f_core::Rect) {
    let lock = doc.lock().expect("locked");
    lock.render_plan
        .pages
        .iter()
        .flat_map(|p| p.ops.iter())
        .find_map(|op| match op {
            PaintOp::DrawBox {
                node_id,
                rect,
                decoration,
            } if decoration.shadow.is_none()
                && decoration.blur.is_none()
                && !node_id.contains("::rule_") =>
            {
                Some((node_id.clone(), rect.clone()))
            }
            _ => None,
        })
        .expect("lock has a simple DrawBox")
}

#[test]
fn invoice_has_solid_wsp() {
    let doc = common::invoice();
    let (node_id, _) = simple_drawbox(&doc);
    let docx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let mut found = false;
    for wsp in parsed.descendants().filter(|n| n.has_tag_name("wsp")) {
        if is_textbox_wsp(wsp) {
            continue;
        }
        let Some(anchor) = wsp.ancestors().find(|n| n.has_tag_name("anchor")) else {
            continue;
        };
        let name = anchor
            .descendants()
            .find(|n| n.has_tag_name("docPr"))
            .and_then(|n| common::local_attr(&n, "name"));
        let has_fill = wsp.descendants().any(|n| n.has_tag_name("solidFill"));
        if has_fill && name == Some(node_id.as_str()) {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "expected non-textbox wps:wsp with a:solidFill named {node_id}"
    );
}

#[test]
fn shape_extent_near_lock_drawbox() {
    let doc = common::invoice();
    let (node_id, rect) = simple_drawbox(&doc);
    let docx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let anchor = common::anchor_named(&parsed, &node_id)
        .unwrap_or_else(|| panic!("no wp:anchor docPr name={node_id}"));
    let (x, y) = common::pos_xy(anchor);
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
    let ext = anchor
        .descendants()
        .find(|n| n.has_tag_name("extent"))
        .expect("wp:extent");
    let cx: i64 = common::local_attr(&ext, "cx").unwrap().parse().unwrap();
    let cy: i64 = common::local_attr(&ext, "cy").unwrap().parse().unwrap();
    assert!(
        (cx - pt_to_emu(rect.width)).abs() <= 1,
        "cx {cx} vs {}",
        pt_to_emu(rect.width)
    );
    assert!(
        (cy - pt_to_emu(rect.height)).abs() <= 1,
        "cy {cy} vs {}",
        pt_to_emu(rect.height)
    );
}

#[test]
fn pictures_get_media_if_present() {
    let doc = common::invoice();
    let lock = doc.lock().expect("locked");
    let (node_id, rect, src) = lock
        .render_plan
        .pages
        .iter()
        .flat_map(|p| p.ops.iter())
        .find_map(|op| match op {
            PaintOp::DrawImage { node_id, rect, src } => {
                Some((node_id.clone(), rect.clone(), src.clone()))
            }
            _ => None,
        })
        .expect("invoice has DrawImage");
    let asset = lookup_image(doc.assets(), &src).expect("image bytes");
    let img = decode_raster(asset).expect("image decode");
    let dest = letterbox_rect(img.width(), img.height(), &rect).unwrap_or(rect.clone());

    let docx = export_opened(&doc).unwrap();
    let names = common::unzip_names(&docx);
    let media: Vec<_> = names
        .iter()
        .filter(|n| n.starts_with("word/media/"))
        .cloned()
        .collect();
    assert!(
        !media.is_empty(),
        "word/media/ must be non-empty when lock has DrawImage, got {names:?}"
    );
    let media_bytes = common::bytes_in(&docx, &media[0]);
    assert_eq!(media_bytes, *asset);

    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    assert!(
        parsed.descendants().any(|n| n.has_tag_name("pic")),
        "missing pic:pic"
    );
    assert!(
        parsed.descendants().any(|n| n.has_tag_name("blip")),
        "missing a:blip"
    );
    let anchor = common::anchor_named(&parsed, &node_id)
        .unwrap_or_else(|| panic!("no picture docPr name={node_id}"));
    let (x, y) = common::pos_xy(anchor);
    assert!((x - pt_to_emu(dest.x)).abs() <= 1);
    assert!((y - pt_to_emu(dest.y)).abs() <= 1);
    let ext = anchor
        .descendants()
        .find(|n| n.has_tag_name("extent"))
        .expect("wp:extent");
    let cx: i64 = common::local_attr(&ext, "cx").unwrap().parse().unwrap();
    let cy: i64 = common::local_attr(&ext, "cy").unwrap().parse().unwrap();
    assert!((cx - pt_to_emu(dest.width)).abs() <= 1);
    assert!((cy - pt_to_emu(dest.height)).abs() <= 1);
}

#[test]
fn text_still_txbx() {
    let docx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    assert!(
        parsed.descendants().any(|n| n.has_tag_name("txbxContent")),
        "step 2 text boxes must remain w:txbxContent"
    );
}

#[test]
fn no_effect_raster_prefix_yet() {
    let docx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    for pr in parsed.descendants().filter(|n| n.has_tag_name("docPr")) {
        let name = common::local_attr(&pr, "name").unwrap_or("");
        assert!(
            !name.starts_with("k2f-raster:"),
            "invoice docPr name {name} must not use effect-raster prefix"
        );
    }
}
