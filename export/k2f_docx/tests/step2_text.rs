#![allow(non_snake_case)]
mod common;

use k2f_core::{
    find_in_trees, for_each_node, node_text, GeometryNode, GlyphPosition, Modifier, NodeContent,
    PaintOp, Pt, Rect, SemanticNode, TextGlyphRun, TextPaintStyle,
};
use k2f_docx::{
    escape_xml, export_opened, infer_text_align, pt_to_emu, textbox_from_draw, textbox_wml,
    TextAlign,
};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

fn w_t_blob(xml: &str) -> String {
    let doc = roxmltree::Document::parse(xml).unwrap();
    doc.descendants()
        .filter(|n| n.has_tag_name("t"))
        .filter_map(|n| n.text())
        .collect::<Vec<_>>()
        .join("")
}

fn running_ids(doc: &k2f_paint::OpenedDocument) -> HashSet<String> {
    let mut ids = HashSet::new();
    for rb in doc.running_blocks() {
        for_each_node(&rb.node, &mut |n| {
            ids.insert(n.id.clone());
        });
    }
    ids
}

fn sample_body_strings(doc: &k2f_paint::OpenedDocument) -> Vec<String> {
    let skip = running_ids(doc);
    let mut ids = Vec::new();
    for_each_node(doc.semantic_root(), &mut |n| {
        if skip.contains(&n.id) || n.role == "math" || matches!(n.content, NodeContent::Math(_)) {
            return;
        }
        if let Some(t) = node_text(n) {
            if !t.is_empty() && !t.contains('\n') {
                ids.push(n.id.clone());
            }
        }
    });
    assert!(ids.len() >= 3, "need 3 body strings, got {}", ids.len());
    ids.into_iter()
        .take(3)
        .map(|id| {
            node_text(find_in_trees(doc.semantic_root(), doc.running_blocks(), &id).unwrap())
                .unwrap()
                .to_string()
        })
        .collect()
}

fn glyph(cluster: u32, x_off: i128, x_adv: i128, y: i128) -> GlyphPosition {
    GlyphPosition {
        glyph_id: 1,
        cluster,
        x_offset: Pt(x_off),
        y_offset: Pt(y),
        x_advance: Pt(x_adv),
        y_advance: Pt(0),
    }
}

fn geo(width: i128, glyphs: Vec<GlyphPosition>) -> GeometryNode {
    GeometryNode {
        id: "g".into(),
        x: Pt(0),
        y: Pt(0),
        width: Pt(width),
        height: Pt(40_000),
        glyphs,
        text_runs: vec![],
        fill_rects: vec![],
        children: vec![],
    }
}

fn style(color: &str, size: i128) -> TextPaintStyle {
    TextPaintStyle {
        font_family: "default".into(),
        font_size: Pt(size),
        color: color.into(),
        bold: false,
        italic: false,
        strikethrough: false,
        underline: false,
    }
}

fn node_with(text: &str, modifiers: Vec<Modifier>) -> SemanticNode {
    SemanticNode {
        id: "n".into(),
        role: "body".into(),
        content: NodeContent::Text(text.into()),
        modifiers,
        ..Default::default()
    }
}

fn glyphs_for(text: &str, x0: i128, adv: i128) -> Vec<GlyphPosition> {
    text.chars()
        .enumerate()
        .map(|(i, _)| glyph(i as u32, x0 + i as i128 * adv, adv, 12_000))
        .collect()
}

fn wml_from(
    text: &str,
    modifiers: Vec<Modifier>,
    runs: Vec<TextGlyphRun>,
    glyphs: Vec<GlyphPosition>,
    width: i128,
) -> String {
    let node = node_with(text, modifiers);
    let g = geo(width, glyphs);
    let rect = Rect {
        x: Pt(0),
        y: Pt(0),
        width: Pt(width),
        height: Pt(40_000),
    };
    let tb = textbox_from_draw(&node, &rect, &runs, Some(&g), &BTreeMap::new()).unwrap();
    textbox_wml(&tb)
}

#[test]
fn invoice_document_contains_semantic_strings() {
    let doc = common::invoice();
    let samples = sample_body_strings(&doc);
    let xml = common::xml_in(&export_opened(&doc).unwrap(), "word/document.xml");
    let blob = w_t_blob(&xml);
    for s in &samples {
        assert!(blob.contains(s), "missing {s:?} in document.xml w:t");
    }
}

#[test]
fn text_is_txbx_not_only_drawing_blip() {
    let docx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    assert!(
        parsed
            .descendants()
            .any(|n| n.has_tag_name("txbx") || n.has_tag_name("txbxContent")),
        "missing wps:txbx / w:txbxContent"
    );
}

#[test]
#[allow(non_snake_case)]
fn textbox_posOffset_near_lock() {
    let doc = common::invoice();
    let skip = running_ids(&doc);
    let lock = doc.lock().unwrap();
    let plan = lock
        .render_plan
        .pages
        .iter()
        .find(|p| p.index == 0)
        .unwrap();
    let (node_id, rect) = plan
        .ops
        .iter()
        .find_map(|op| match op {
            PaintOp::DrawText { node_id, rect, .. } if !skip.contains(node_id) => {
                let n = find_in_trees(doc.semantic_root(), doc.running_blocks(), node_id)?;
                if n.role == "math" || matches!(n.content, NodeContent::Math(_)) {
                    return None;
                }
                node_text(n)?;
                Some((node_id.clone(), rect.clone()))
            }
            _ => None,
        })
        .expect("page 0 DrawText");
    let xml = common::xml_in(&export_opened(&doc).unwrap(), "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let pr = parsed
        .descendants()
        .find(|n| {
            n.has_tag_name("docPr") && common::local_attr(n, "name") == Some(node_id.as_str())
        })
        .expect("docPr");
    let anchor = pr
        .ancestors()
        .find(|n| n.has_tag_name("anchor"))
        .expect("wp:anchor");
    let pos = anchor
        .descendants()
        .find(|n| n.has_tag_name("posOffset"))
        .unwrap();
    let x: i64 = pos.text().unwrap().parse().unwrap();
    assert!(
        (x - pt_to_emu(rect.x)).abs() <= 1,
        "x {x} vs {}",
        pt_to_emu(rect.x)
    );
}

#[test]
fn infer_text_align_unit() {
    let w = 100_000i128;
    assert_eq!(
        infer_text_align(&geo(w, vec![glyph(0, 0, 40_000, 0)]), "A"),
        TextAlign::Left
    );
    assert_eq!(
        infer_text_align(&geo(w, vec![glyph(0, 30_000, 40_000, 0)]), "A"),
        TextAlign::Center
    );
    assert_eq!(
        infer_text_align(&geo(w, vec![glyph(0, 60_000, 40_000, 0)]), "A"),
        TextAlign::Right
    );
    assert_eq!(
        infer_text_align(&geo(w, vec![glyph(0, 4_000, 92_000, 0)]), "A"),
        TextAlign::Center
    );
    assert_eq!(infer_text_align(&geo(w, vec![]), "A"), TextAlign::Left);
    let just = geo(
        w,
        vec![
            glyph(0, 0, 20_000, 0),
            glyph(1, 20_000, 40_000, 0),
            glyph(2, 60_000, 20_000, 0),
            glyph(4, 0, 20_000, 14_000),
            glyph(5, 20_000, 10_000, 14_000),
            glyph(6, 30_000, 20_000, 14_000),
        ],
    );
    assert_eq!(infer_text_align(&just, "A B\nA B"), TextAlign::Justify);
}

#[test]
fn align_modes_fixture_writes_w_jc() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/align_modes.K2F");
    assert!(
        path.exists(),
        "missing {path:?}; pack alignment_text_modes into tests/fixtures/align_modes.K2F"
    );
    let docx =
        export_opened(&k2f_paint::OpenedDocument::open(&std::fs::read(&path).unwrap()).unwrap())
            .unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    for (id, jc) in [
        ("txt_start", "left"),
        ("txt_center", "center"),
        ("txt_end", "right"),
    ] {
        let anchor = parsed
            .descendants()
            .filter(|n| n.has_tag_name("anchor"))
            .find(|anchor| {
                let named = anchor
                    .descendants()
                    .any(|n| n.has_tag_name("docPr") && common::local_attr(&n, "name") == Some(id));
                let txbox = anchor.descendants().any(|n| {
                    n.has_tag_name("cNvSpPr") && common::local_attr(&n, "txBox") == Some("1")
                });
                named && txbox
            })
            .unwrap_or_else(|| panic!("no textbox wp:anchor {id}"));
        let got = anchor
            .descendants()
            .find(|n| n.has_tag_name("jc"))
            .and_then(|n| common::local_attr(&n, "val"));
        assert_eq!(got, Some(jc), "{id} jc");
    }
}

#[test]
fn paint_runs_keep_per_run_color_and_size() {
    let text = "HelloWorld";
    let xml = wml_from(
        text,
        vec![],
        vec![
            TextGlyphRun {
                glyph_range: [0, 5],
                style: style("#FF0000", 12_000),
            },
            TextGlyphRun {
                glyph_range: [5, 10],
                style: style("#0000FF", 8_000),
            },
        ],
        glyphs_for(text, 0, 8_000),
        100_000,
    );
    let doc = roxmltree::Document::parse(&xml).unwrap();
    let colors: Vec<_> = doc
        .descendants()
        .filter(|n| n.has_tag_name("color"))
        .filter_map(|n| common::local_attr(&n, "val").map(|s| s.to_ascii_uppercase()))
        .collect();
    let sizes: Vec<_> = doc
        .descendants()
        .filter(|n| n.has_tag_name("sz"))
        .filter_map(|n| common::local_attr(&n, "val"))
        .collect();
    assert!(colors.iter().any(|c| c == "FF0000"), "{colors:?}");
    assert!(colors.iter().any(|c| c == "0000FF"), "{colors:?}");
    assert!(sizes.contains(&"24"), "{sizes:?}");
    assert!(sizes.contains(&"16"), "{sizes:?}");
}

#[test]
fn black_text_is_not_word_automatic() {
    let xml = wml_from(
        "Title",
        vec![],
        vec![TextGlyphRun {
            glyph_range: [0, 5],
            style: style("#000000", 12_000),
        }],
        glyphs_for("Title", 0, 8_000),
        100_000,
    );
    assert!(
        xml.contains(r#"w:val="000001""#) && xml.contains(r#"w14:textFill"#),
        "pure black must be stored as RGB fill, not Automatic, got {xml}"
    );
    assert!(
        xml.contains(r#"<a:fontRef idx="minor"><a:srgbClr val="000001"/>"#),
        "textbox style must pin fontRef to RGB, not scheme dk1, got {xml}"
    );
    assert!(
        !xml.contains(r#"w:val="000000""#),
        "Word treats 000000 as Automatic in Dark Mode, got {xml}"
    );
}

#[test]
fn exported_textboxes_paint_opaque_underlay() {
    let docx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    assert!(
        xml.contains("<wps:txbx>") && xml.contains("<a:solidFill>"),
        "Word Dark Mode inverts noFill text boxes; expected opaque underlay, got {xml}"
    );
}

#[test]
fn resolved_font_not_alias_stem_in_invoice() {
    let docx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    assert!(
        xml.contains(r#"w:ascii="Roboto""#),
        "embedded font must use the TTF family name, got {xml}"
    );
    assert!(
        !xml.contains(r#"w:ascii="Roboto-Regular""#) && !xml.contains(r#"w:ascii="default""#),
        "lock alias keys must not leak into Word rFonts, got {xml}"
    );
}

#[test]
fn subscript_is_vertAlign_not_italic() {
    let text = "H2O";
    let xml = wml_from(
        text,
        vec![Modifier {
            range: [1, 2],
            mod_type: "subscript".into(),
            intent: String::new(),
        }],
        vec![
            TextGlyphRun {
                glyph_range: [0, 1],
                style: style("#000000", 12_000),
            },
            TextGlyphRun {
                glyph_range: [1, 2],
                style: style("#000000", 8_400),
            },
            TextGlyphRun {
                glyph_range: [2, 3],
                style: style("#000000", 12_000),
            },
        ],
        glyphs_for(text, 0, 8_000),
        80_000,
    );
    let doc = roxmltree::Document::parse(&xml).unwrap();
    let sub = doc
        .descendants()
        .find(|n| n.has_tag_name("vertAlign") && common::local_attr(n, "val") == Some("subscript"))
        .expect("subscript vertAlign");
    let rpr = sub.parent().expect("rPr");
    assert!(
        !rpr.children().any(|n| n.has_tag_name("i")),
        "subscript must not use italic"
    );
    let sub_sz: i32 = rpr
        .children()
        .find(|n| n.has_tag_name("sz"))
        .and_then(|n| common::local_attr(&n, "val"))
        .unwrap()
        .parse()
        .unwrap();
    let base_sz: i32 = doc
        .descendants()
        .filter(|n| n.has_tag_name("sz"))
        .filter_map(|n| common::local_attr(&n, "val")?.parse().ok())
        .max()
        .unwrap();
    assert!(sub_sz < base_sz, "sub {sub_sz} vs {base_sz}");
}

#[test]
fn italic_emphasis_on_italic_paint_is_not_bold() {
    let text = "Keywords: italic only";
    let mut italic = style("#000000", 9_000);
    italic.italic = true;
    let xml = wml_from(
        text,
        vec![Modifier {
            range: [10, text.len()],
            mod_type: "emphasis".into(),
            intent: "italic".into(),
        }],
        vec![
            TextGlyphRun {
                glyph_range: [0, 10],
                style: style("#000000", 9_000),
            },
            TextGlyphRun {
                glyph_range: [10, text.chars().count()],
                style: italic,
            },
        ],
        glyphs_for(text, 0, 8_000),
        200_000,
    );
    let doc = roxmltree::Document::parse(&xml).unwrap();
    let italic_t = doc
        .descendants()
        .find(|n| n.has_tag_name("t") && n.text() == Some("italic only"))
        .expect("italic run text");
    let rpr = italic_t
        .parent()
        .and_then(|r| r.children().find(|n| n.has_tag_name("rPr")))
        .expect("rPr");
    assert!(
        rpr.children().any(|n| n.has_tag_name("i")),
        "italic emphasis must keep italic, got {xml}"
    );
    assert!(
        !rpr.children().any(|n| n.has_tag_name("b")),
        "italic emphasis must not force bold, got {xml}"
    );
}

#[test]
fn syntax_highlight_keeps_run_colors() {
    let text = "ab";
    let xml = wml_from(
        text,
        vec![Modifier {
            range: [0, 2],
            mod_type: "syntax_highlight".into(),
            intent: "kw".into(),
        }],
        vec![
            TextGlyphRun {
                glyph_range: [0, 1],
                style: style("#AA0000", 11_000),
            },
            TextGlyphRun {
                glyph_range: [1, 2],
                style: style("#00AA00", 11_000),
            },
        ],
        glyphs_for(text, 0, 8_000),
        40_000,
    );
    let colors: Vec<_> = roxmltree::Document::parse(&xml)
        .unwrap()
        .descendants()
        .filter(|n| n.has_tag_name("color"))
        .filter_map(|n| common::local_attr(&n, "val").map(|s| s.to_ascii_uppercase()))
        .collect();
    assert!(colors.iter().any(|c| c == "AA0000"), "{colors:?}");
    assert!(colors.iter().any(|c| c == "00AA00"), "{colors:?}");
}

#[test]
fn left_align_lIns_not_zero_when_glyphs_inset() {
    let text = "A";
    let xml = wml_from(
        text,
        vec![],
        vec![TextGlyphRun {
            glyph_range: [0, 1],
            style: style("#111111", 12_000),
        }],
        vec![glyph(0, 8_000, 10_000, 12_000)],
        100_000,
    );
    let doc = roxmltree::Document::parse(&xml).unwrap();
    let body = doc
        .descendants()
        .find(|n| n.has_tag_name("bodyPr"))
        .expect("bodyPr");
    let l: i64 = common::local_attr(&body, "lIns").unwrap().parse().unwrap();
    let r: i64 = common::local_attr(&body, "rIns").unwrap().parse().unwrap();
    assert_eq!(l, pt_to_emu(Pt(8_000)));
    assert_ne!(l, 0);
    assert_eq!(r, 0);
}

#[test]
fn left_align_rIns_is_zero_when_line_does_not_fill_box() {
    let text = "Title";
    let xml = wml_from(
        text,
        vec![],
        vec![TextGlyphRun {
            glyph_range: [0, 5],
            style: style("#111111", 31_000),
        }],
        glyphs_for(text, 0, 12_000),
        506_000,
    );
    let doc = roxmltree::Document::parse(&xml).unwrap();
    let body = doc
        .descendants()
        .find(|n| n.has_tag_name("bodyPr"))
        .expect("bodyPr");
    let r: i64 = common::local_attr(&body, "rIns").unwrap().parse().unwrap();
    let l: i64 = common::local_attr(&body, "lIns").unwrap().parse().unwrap();
    assert_eq!(r, 0, "unused right gap must not become rIns");
    assert_eq!(l, 0);
}

#[test]
fn running_footer_not_duplicated_in_body() {
    let doc = common::invoice();
    assert!(
        !doc.running_blocks().is_empty(),
        "invoice should have running footer"
    );
    let docx = export_opened(&doc).unwrap();
    let footer = common::xml_in(&docx, "word/footer1.xml");
    assert!(
        footer.contains(" PAGE ") && footer.contains("NUMPAGES"),
        "footer must emit PAGE and NUMPAGES fields, not a split token"
    );
    assert!(
        !footer.contains("{{page"),
        "footer must not leak template tokens"
    );
    let body = common::xml_in(&docx, "word/document.xml");
    assert!(!body.contains("Page 1 of 3") && !body.contains("Page {{page_current}}"));
    assert_eq!(
        body.matches("Page {{page_current}} of {{page_total}}")
            .count(),
        0
    );
}

#[test]
fn page_tokens_split_across_paint_runs_become_fields() {
    let text = "Page {{page_current}} of {{page_total}}";
    let xml = wml_from(
        text,
        vec![],
        vec![
            TextGlyphRun {
                glyph_range: [0, 10],
                style: style("#5F6368", 9_000),
            },
            TextGlyphRun {
                glyph_range: [10, text.chars().count()],
                style: style("#5F6368", 9_000),
            },
        ],
        glyphs_for(text, 0, 8_000),
        200_000,
    );
    assert!(xml.contains(" PAGE "), "{xml}");
    assert!(xml.contains("NUMPAGES"), "{xml}");
    assert!(!xml.contains("{{page"), "{xml}");
}

#[test]
fn xml_escape_unit() {
    assert_eq!(escape_xml("a&b<c>"), "a&amp;b&lt;c&gt;");
}

#[test]
fn paint_range_does_not_emit_unpainted_prefix() {
    let text = "HelloWorld";
    let glyphs: Vec<_> = (5..10)
        .map(|i| glyph(i as u32, (i as i128 - 5) * 8_000, 8_000, 12_000))
        .collect();
    let xml = wml_from(
        text,
        vec![],
        vec![TextGlyphRun {
            glyph_range: [0, 5],
            style: style("#111111", 12_000),
        }],
        glyphs,
        80_000,
    );
    let blob = w_t_blob(&xml);
    assert!(!blob.contains("Hello"), "{blob}");
    assert!(blob.contains("World"), "{blob}");
}

#[test]
fn export_still_byte_identical() {
    let doc = common::invoice();
    assert_eq!(export_opened(&doc).unwrap(), export_opened(&doc).unwrap());
}
