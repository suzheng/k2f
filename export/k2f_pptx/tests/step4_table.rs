mod common;

use k2f_core::{
    find_in_trees, for_each_node, node_text, GeometryNode, NodeContent, PaintOp, TableDataSource,
};
use k2f_pptx::{export_opened, pt_to_emu};

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

fn invoice_table_id_and_col_count(doc: &k2f_paint::OpenedDocument) -> (String, usize) {
    let mut found = None;
    for_each_node(doc.semantic_root(), &mut |n| {
        if let NodeContent::Table(spec) = &n.content {
            found = Some((n.id.clone(), spec.column_widths.len()));
        }
    });
    found.expect("invoice semantic tree has a Table node after asset expand")
}

fn invoice_sample_cell(doc: &k2f_paint::OpenedDocument) -> (String, String) {
    let mut sample = None;
    for_each_node(doc.semantic_root(), &mut |n| {
        if let NodeContent::Table(spec) = &n.content {
            if let TableDataSource::Inline { rows } = &spec.data {
                if let Some(cell) = rows.get(1).and_then(|r| r.first()) {
                    if let Some(text) = node_text(cell) {
                        sample = Some((cell.id.clone(), text.to_string()));
                    }
                }
            } else {
                panic!("invoice table still Asset; OpenedDocument should expand");
            }
        }
    });
    sample.expect("invoice table has a text cell in row 1")
}

fn invoice_cell_ids(doc: &k2f_paint::OpenedDocument) -> Vec<String> {
    let mut ids = Vec::new();
    for_each_node(doc.semantic_root(), &mut |n| {
        if let NodeContent::Table(spec) = &n.content {
            if let TableDataSource::Inline { rows } = &spec.data {
                ids.extend(rows.iter().flatten().map(|c| c.id.clone()));
            }
        }
    });
    ids
}

#[test]
fn invoice_or_fixture_has_a_tbl() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let xml = all_slide_xml(&pptx);
    assert!(
        xml.contains("<a:tbl>"),
        "invoice line items must export as a:tbl"
    );
    assert!(
        xml.contains("<a:tblGrid>"),
        "native table must include a:tblGrid"
    );
}

#[test]
fn tbl_grid_col_count_matches_spec() {
    let doc = common::invoice();
    let (_, want) = invoice_table_id_and_col_count(&doc);
    let pptx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&pptx, "ppt/slides/slide2.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let cols = parsed
        .descendants()
        .filter(|n| n.has_tag_name("gridCol"))
        .count();
    assert_eq!(cols, want, "a:gridCol count vs spec.column_widths");
}

#[test]
fn cell_text_matches_semantic() {
    let doc = common::invoice();
    let (cell_id, text) = invoice_sample_cell(&doc);
    assert!(!text.is_empty(), "sample cell text");
    let node = find_in_trees(doc.semantic_root(), doc.running_blocks(), &cell_id)
        .unwrap_or_else(|| panic!("missing {cell_id}"));
    assert_eq!(node_text(node), Some(text.as_str()));

    let pptx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&pptx, "ppt/slides/slide2.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let tbl = parsed
        .descendants()
        .find(|n| n.has_tag_name("tbl"))
        .expect("slide2 a:tbl");
    let blob: String = tbl
        .descendants()
        .filter(|n| n.has_tag_name("t"))
        .filter_map(|n| n.text())
        .collect();
    assert!(
        blob.contains(&text),
        "cell {text:?} ({cell_id}) missing from a:tc"
    );
}

#[test]
fn table_cells_not_duplicated_as_textboxes() {
    let doc = common::invoice();
    let cell_ids = invoice_cell_ids(&doc);
    assert!(cell_ids.iter().any(|id| id == "invoice.row_1.item"));

    let pptx = export_opened(&doc).unwrap();
    for name in slide_xml_names(&pptx) {
        let xml = common::xml_in(&pptx, &name);
        let parsed = roxmltree::Document::parse(&xml).unwrap();
        for sp in parsed.descendants().filter(|n| n.has_tag_name("sp")) {
            let is_txbox = sp
                .descendants()
                .any(|c| c.has_tag_name("cNvSpPr") && c.attribute("txBox") == Some("1"));
            if !is_txbox {
                continue;
            }
            let Some(cnv) = sp.descendants().find(|c| c.has_tag_name("cNvPr")) else {
                continue;
            };
            let Some(id) = cnv.attribute("name") else {
                continue;
            };
            assert!(
                !cell_ids.iter().any(|c| c == id),
                "{name}: cell {id} still exported as a text box"
            );
        }
    }
}

#[test]
fn pptx_still_byte_identical_with_tables() {
    let doc = common::invoice();
    let a = export_opened(&doc).unwrap();
    let b = export_opened(&doc).unwrap();
    assert_eq!(a, b);
}

fn table_geometry_on_page(doc: &k2f_paint::OpenedDocument, page_index: usize) -> GeometryNode {
    let lock = doc.lock().expect("locked");
    let page = lock
        .geometry
        .pages
        .iter()
        .find(|p| p.index == page_index)
        .unwrap_or_else(|| panic!("page {page_index}"));
    let (table_id, _) = invoice_table_id_and_col_count(doc);
    find_geo(&page.root, &table_id).unwrap_or_else(|| panic!("table geo on page {page_index}"))
}

fn find_geo(node: &GeometryNode, id: &str) -> Option<GeometryNode> {
    if node.id == id {
        return Some(node.clone());
    }
    node.children.iter().find_map(|c| find_geo(c, id))
}

fn is_txbox_sp(sp: roxmltree::Node<'_, '_>) -> bool {
    sp.descendants()
        .any(|c| c.has_tag_name("cNvSpPr") && c.attribute("txBox") == Some("1"))
}

#[test]
fn table_cells_not_duplicated_as_shapes() {
    let doc = common::invoice();
    let cell_ids = invoice_cell_ids(&doc);
    let pptx = export_opened(&doc).unwrap();
    for name in slide_xml_names(&pptx) {
        let xml = common::xml_in(&pptx, &name);
        let parsed = roxmltree::Document::parse(&xml).unwrap();
        for sp in parsed.descendants().filter(|n| n.has_tag_name("sp")) {
            if is_txbox_sp(sp) {
                continue;
            }
            let Some(cnv) = sp.descendants().find(|c| c.has_tag_name("cNvPr")) else {
                continue;
            };
            let Some(id) = cnv.attribute("name") else {
                continue;
            };
            assert!(
                !cell_ids.iter().any(|c| c == id),
                "{name}: cell {id} still exported as a shape"
            );
        }
    }
}

fn tc_fill<'a>(tc: roxmltree::Node<'a, 'a>) -> Option<&'a str> {
    tc.descendants()
        .find(|n| n.has_tag_name("tcPr"))
        .and_then(|tcpr| {
            tcpr.children()
                .find(|n| n.has_tag_name("solidFill"))
                .and_then(|sf| sf.descendants().find(|n| n.has_tag_name("srgbClr")))
                .and_then(|c| c.attribute("val"))
        })
}

#[test]
fn header_row_fill_from_drawbox() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&pptx, "ppt/slides/slide2.xml");
    assert!(
        xml.contains("1A73E8"),
        "invoice header cell fill should come from lock DrawBox"
    );
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let header_tc = parsed.descendants().find(|n| {
        n.has_tag_name("tc")
            && n.descendants()
                .any(|t| t.has_tag_name("t") && t.text() == Some("Item"))
            && tc_fill(*n) == Some("1A73E8")
    });
    assert_eq!(
        tc_fill(header_tc.expect("header Item cell")),
        Some("1A73E8")
    );
}

#[test]
fn paginated_table_on_slide3() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&pptx, "ppt/slides/slide3.xml");
    assert!(
        xml.contains("<a:tbl>"),
        "paginated table fragment on slide 3"
    );
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    assert_eq!(
        parsed
            .descendants()
            .filter(|n| n.has_tag_name("gridCol"))
            .count(),
        4
    );
}

#[test]
fn table_graphic_frame_near_lock_rect() {
    let doc = common::invoice();
    let geo = table_geometry_on_page(&doc, 1);
    let pptx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&pptx, "ppt/slides/slide2.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let frame = parsed
        .descendants()
        .find(|n| n.has_tag_name("graphicFrame"))
        .expect("slide2 graphicFrame");
    let off = frame
        .descendants()
        .find(|n| n.has_tag_name("off"))
        .expect("a:off");
    let ext = frame
        .descendants()
        .find(|n| n.has_tag_name("ext"))
        .expect("a:ext");
    let x: i64 = off.attribute("x").unwrap().parse().unwrap();
    let y: i64 = off.attribute("y").unwrap().parse().unwrap();
    let cx: i64 = ext.attribute("cx").unwrap().parse().unwrap();
    let cy: i64 = ext.attribute("cy").unwrap().parse().unwrap();
    assert!((x - pt_to_emu(geo.x)).abs() <= 1);
    assert!((y - pt_to_emu(geo.y)).abs() <= 1);
    assert!((cx - pt_to_emu(geo.width)).abs() <= 1);
    assert!((cy - pt_to_emu(geo.height)).abs() <= 1);
}

#[test]
fn unfilled_cells_get_opaque_underlay() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&pptx, "ppt/slides/slide2.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let tcs: Vec<_> = parsed
        .descendants()
        .filter(|n| n.has_tag_name("tc"))
        .collect();
    assert!(!tcs.is_empty(), "slide2 must have native table cells");
    for tc in &tcs {
        assert!(
            tc_fill(*tc).is_some(),
            "every native table cell needs a solid fill so Dark Mode does not invert"
        );
    }
}

#[test]
fn native_table_tcpr_emits_anchor() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let mut seen = 0usize;
    for name in slide_xml_names(&pptx) {
        let xml = common::xml_in(&pptx, &name);
        let parsed = roxmltree::Document::parse(&xml).unwrap();
        for tc in parsed.descendants().filter(|n| n.has_tag_name("tc")) {
            seen += 1;
            let tcpr = tc
                .children()
                .find(|n| n.has_tag_name("tcPr"))
                .expect("a:tcPr");
            let anchor = tcpr.attribute("anchor");
            assert!(
                anchor == Some("ctr") || anchor == Some("t"),
                "{name}: PowerPoint reads tcPr/@anchor (default t); got {anchor:?}"
            );
            assert_eq!(
                tcpr.attribute("marT"),
                Some("0"),
                "{name}: top padding is bodyPr tIns, not tcPr marT"
            );
        }
    }
    assert!(seen > 0, "invoice must have native table cells");
}

#[test]
fn native_table_cells_do_not_fake_gray_grid() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let xml = all_slide_xml(&pptx);
    assert!(
        !xml.contains(r#"<a:lnL w="6350"><a:solidFill><a:srgbClr val="D0D0D0"/>"#),
        "native table cells must follow lock edges, not a fake #D0D0D0 grid"
    );
    assert!(
        xml.contains("<a:lnL><a:noFill/></a:lnL>") || xml.contains("<a:noFill/>"),
        "cells without lock borders need explicit noFill so hosts do not invent a grid"
    );
}

#[test]
fn draw_table_reference_is_shape_not_tbl() {
    let doc = common::invoice();
    let lock = doc.lock().expect("locked");
    let has_ref = lock.render_plan.pages.iter().any(|p| {
        p.ops
            .iter()
            .any(|op| matches!(op, PaintOp::DrawTableReference { .. }))
    });
    if !has_ref {
        return;
    }
    let pptx = export_opened(&doc).unwrap();
    let xml = all_slide_xml(&pptx);
    assert!(
        !xml.contains("DrawTableReference"),
        "placeholder uses shape, not raw op name"
    );
}
