mod common;

use k2f_core::PaintOp;
use k2f_docx::export_opened;

fn relative_height(anchor: roxmltree::Node<'_, '_>) -> u32 {
    common::local_attr(&anchor, "relativeHeight")
        .unwrap()
        .parse()
        .unwrap()
}

fn first_anchor_with<'a, 'input>(
    doc: &'a roxmltree::Document<'input>,
    name: &str,
    want_txbx: bool,
) -> roxmltree::Node<'a, 'input> {
    doc.descendants()
        .filter(|n| n.has_tag_name("docPr") && common::local_attr(n, "name") == Some(name))
        .filter_map(|pr| pr.ancestors().find(|n| n.has_tag_name("anchor")))
        .find(|anchor| anchor.descendants().any(|n| n.has_tag_name("txbxContent")) == want_txbx)
        .unwrap_or_else(|| panic!("no anchor name={name} txbx={want_txbx}"))
}

#[test]
fn page_background_behind_doc() {
    let docx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let anchor = common::anchor_named(&parsed, "invoice_root::page_0::background")
        .expect("page paper drawing");
    assert_eq!(
        common::local_attr(&anchor, "behindDoc"),
        Some("1"),
        "page paper is the only behindDoc fill"
    );
    let cell = common::anchor_named(&parsed, "invoice.header").expect("header text");
    assert_eq!(common::local_attr(&cell, "behindDoc"), Some("0"));
}

#[test]
fn landscape_pg_sz_sets_orient() {
    let xml = common::xml_in(
        &export_opened(&common::invoice()).unwrap(),
        "word/document.xml",
    );
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let sz = parsed
        .descendants()
        .find(|n| n.has_tag_name("pgSz"))
        .expect("pgSz");
    let w: i64 = common::local_attr(&sz, "w").unwrap().parse().unwrap();
    let h: i64 = common::local_attr(&sz, "h").unwrap().parse().unwrap();
    if w > h {
        assert_eq!(common::local_attr(&sz, "orient"), Some("landscape"));
    } else {
        assert!(common::local_attr(&sz, "orient").is_none());
    }
}

#[test]
fn z_order_follows_paint_ops() {
    let doc = common::invoice();
    let lock = doc.lock().expect("locked");
    let page0 = lock
        .render_plan
        .pages
        .iter()
        .find(|p| p.index == 0)
        .expect("page 0");
    assert!(matches!(
        &page0.ops[1],
        PaintOp::DrawImage { node_id, .. } if node_id == "invoice.logo"
    ));
    assert!(matches!(
        &page0.ops[2],
        PaintOp::DrawText { node_id, .. } if node_id == "invoice.header"
    ));
    let docx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let pic_h = relative_height(common::anchor_named(&parsed, "invoice.logo").expect("logo pic"));
    let text_h = relative_height(first_anchor_with(&parsed, "invoice.header", true));
    assert!(
        text_h > pic_h,
        "later text relativeHeight {text_h} must stay above earlier picture {pic_h}"
    );
}

#[test]
fn content_types_and_rels_register_png() {
    let docx = export_opened(&common::invoice()).unwrap();
    let ct = common::xml_in(&docx, "[Content_Types].xml");
    assert!(
        ct.contains("Extension=\"png\"") && ct.contains("image/png"),
        "Content_Types must register png, got {ct}"
    );
    let rels = common::xml_in(&docx, "word/_rels/document.xml.rels");
    assert!(
        rels.contains("/relationships/image") && rels.contains("Target=\"media/"),
        "document.xml.rels must point at word/media, got {rels}"
    );
}
