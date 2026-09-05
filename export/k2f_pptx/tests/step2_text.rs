mod common;

use k2f_core::{find_in_trees, for_each_node, node_text, NodeContent, PaintOp};
use k2f_pptx::{escape_xml, export_opened, pt_to_emu};

fn slide_xml_names(pptx: &[u8]) -> Vec<String> {
    common::unzip_names(pptx)
        .into_iter()
        .filter(|n| {
            n.starts_with("ppt/slides/slide") && n.ends_with(".xml") && !n.contains("_rels")
        })
        .collect()
}

fn all_a_t_text(pptx: &[u8]) -> String {
    slide_xml_names(pptx)
        .iter()
        .map(|n| concatenated_a_t(&common::xml_in(pptx, n)))
        .collect::<Vec<_>>()
        .join("")
}

fn concatenated_a_t(xml: &str) -> String {
    let parsed = roxmltree::Document::parse(xml).unwrap();
    parsed
        .descendants()
        .filter(|n| n.has_tag_name("t"))
        .filter_map(|n| n.text())
        .collect::<Vec<_>>()
        .join("")
}

fn sample_semantic_strings(doc: &k2f_paint::OpenedDocument) -> Vec<String> {
    let mut ids = Vec::new();
    for_each_node(doc.semantic_root(), &mut |n| {
        if n.role == "math" || matches!(n.content, NodeContent::Math(_)) {
            return;
        }
        if let Some(t) = node_text(n) {
            if !t.is_empty() && !t.contains('\n') {
                ids.push(n.id.clone());
            }
        }
    });
    assert!(
        ids.len() >= 3,
        "invoice should have at least 3 non-empty text nodes, got {}",
        ids.len()
    );
    ids.into_iter()
        .take(3)
        .map(|id| {
            let node = find_in_trees(doc.semantic_root(), doc.running_blocks(), &id)
                .unwrap_or_else(|| panic!("find_in_trees missed {id}"));
            node_text(node)
                .expect("sampled node has text")
                .to_string()
        })
        .collect()
}

#[test]
fn invoice_slide_xml_contains_semantic_strings() {
    let doc = common::invoice();
    let samples = sample_semantic_strings(&doc);
    let pptx = export_opened(&doc).unwrap();
    let blob = all_a_t_text(&pptx);
    for s in &samples {
        assert!(
            blob.contains(s),
            "missing semantic string {s:?} in concatenated a:t"
        );
    }
}

#[test]
fn text_is_txbody_not_only_picture() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let s1 = common::xml_in(&pptx, "ppt/slides/slide1.xml");
    let doc = roxmltree::Document::parse(&s1).unwrap();
    assert!(
        doc.descendants().any(|n| n.has_tag_name("txBody")),
        "slide1 missing p:txBody"
    );
    assert!(
        doc.descendants()
            .any(|n| n.has_tag_name("t") && n.text().is_some()),
        "slide1 missing a:t"
    );
}

#[test]
fn textbox_count_matches_draw_text_on_invoice() {
    let doc = common::invoice();
    let lock = doc.lock().expect("locked");
    let pptx = export_opened(&doc).unwrap();
    for page in &lock.geometry.pages {
        let plan = lock
            .render_plan
            .pages
            .iter()
            .find(|p| p.index == page.index)
            .expect("render plan");
        let absorbed = common::native_table_member_ids(&doc);
        let draw_text = plan
            .ops
            .iter()
            .filter(|op| match op {
                PaintOp::DrawText { node_id, .. } => !absorbed.contains(node_id),
                _ => false,
            })
            .count();
        let slide_n = page.index + 1;
        let xml = common::xml_in(&pptx, &format!("ppt/slides/slide{slide_n}.xml"));
        let parsed = roxmltree::Document::parse(&xml).unwrap();
        let sp_count = parsed
            .descendants()
            .filter(|n| n.has_tag_name("sp"))
            .filter(|n| {
                n.parent()
                    .map(|p| p.has_tag_name("spTree"))
                    .unwrap_or(false)
            })
            .filter(|n| {
                n.descendants()
                    .any(|c| c.has_tag_name("cNvSpPr") && c.attribute("txBox") == Some("1"))
            })
            .count();
        assert_eq!(
            sp_count,
            draw_text,
            "slide{slide_n}: sp count {sp_count} vs draw_text {draw_text}"
        );
    }
}

#[test]
fn textbox_origin_near_lock_rect() {
    let doc = common::invoice();
    let lock = doc.lock().expect("locked");
    let plan = lock
        .render_plan
        .pages
        .iter()
        .find(|p| p.index == lock.geometry.pages[0].index)
        .expect("page 0 plan");
    let (node_id, rect) = plan
        .ops
        .iter()
        .find_map(|op| match op {
            PaintOp::DrawText { node_id, rect, .. } => {
                let node = find_in_trees(doc.semantic_root(), doc.running_blocks(), node_id)?;
                if node.role == "math" || matches!(node.content, NodeContent::Math(_)) {
                    return None;
                }
                node_text(node)?;
                Some((node_id.clone(), rect.clone()))
            }
            _ => None,
        })
        .expect("invoice page 0 has a DrawText");

    let pptx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&pptx, "ppt/slides/slide1.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let sp = parsed
        .descendants()
        .find(|n| {
            n.has_tag_name("sp")
                && n.descendants()
                    .any(|c| c.has_tag_name("cNvPr") && c.attribute("name") == Some(node_id.as_str()))
        })
        .unwrap_or_else(|| panic!("no textbox named {node_id}"));
    let off = sp
        .descendants()
        .find(|n| n.has_tag_name("off"))
        .expect("a:off");
    let x: i64 = off.attribute("x").unwrap().parse().unwrap();
    let y: i64 = off.attribute("y").unwrap().parse().unwrap();
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
    let ext = sp
        .descendants()
        .find(|n| n.has_tag_name("ext"))
        .expect("a:ext");
    let cx: i64 = ext.attribute("cx").unwrap().parse().unwrap();
    let cy: i64 = ext.attribute("cy").unwrap().parse().unwrap();
    assert!(
        (cx - pt_to_emu(rect.width)).abs() <= 1,
        "cx {cx} vs {} for {node_id}",
        pt_to_emu(rect.width)
    );
    assert!(
        (cy - pt_to_emu(rect.height)).abs() <= 1,
        "cy {cy} vs {} for {node_id}",
        pt_to_emu(rect.height)
    );
}

#[test]
fn resolved_font_not_default_in_invoice() {
    let pptx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&pptx, "ppt/slides/slide1.xml");
    assert!(
        !xml.contains("typeface=\"default\""),
        "embedded default font must be resolved via ttf-parser"
    );
    assert!(xml.contains("typeface=\"Roboto\""));
}

#[test]
fn xml_special_chars_escaped() {
    assert_eq!(escape_xml("a&b<c>"), "a&amp;b&lt;c&gt;");
    assert_eq!(escape_xml("\"'"), "&quot;&apos;");
}

#[test]
fn pptx_still_byte_identical() {
    let doc = common::invoice();
    let a = export_opened(&doc).unwrap();
    let b = export_opened(&doc).unwrap();
    assert_eq!(a, b);
}
