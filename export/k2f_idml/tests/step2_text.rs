mod common;

use k2f_core::{
    find_in_trees, for_each_node, node_text, GeometryNode, GlyphPosition, Modifier, NodeContent,
    PaintOp, Pt, Rect, SemanticNode, TextGlyphRun, TextPaintStyle,
};
use k2f_idml::{
    escape_xml, export_opened, infer_text_align, story_xml, textbox_from_draw, textframe_xml,
    SpreadSpace, TextAlign,
};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

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

fn stories_blob(idml: &[u8]) -> String {
    let mut blob = String::new();
    for name in common::unzip_names(idml) {
        if !name.starts_with("Stories/") {
            continue;
        }
        let xml = common::xml_in(idml, &name);
        let doc = roxmltree::Document::parse(&xml).unwrap();
        for n in doc.descendants() {
            if n.has_tag_name("Content") {
                if let Some(t) = n.text() {
                    blob.push_str(t);
                }
            }
        }
    }
    blob
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

fn story_from(
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
    story_xml(&tb, "kSt0")
}

fn frame_from(
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
    let space = SpreadSpace {
        page_w: 595.0,
        page_h: 842.0,
    };
    textframe_xml(&tb, &space, "kTf0", "kSt0")
}

#[test]
fn invoice_stories_contain_semantic_strings() {
    let doc = common::invoice();
    let samples = sample_body_strings(&doc);
    let blob = stories_blob(&export_opened(&doc).unwrap());
    for s in &samples {
        assert!(blob.contains(s), "missing {s:?} in Stories Content");
    }
}

#[test]
fn textframe_and_parent_story_match() {
    let idml = export_opened(&common::invoice()).unwrap();
    let names = common::unzip_names(&idml);
    let mut frames = 0usize;
    for name in &names {
        if !name.starts_with("Spreads/Spread_") {
            continue;
        }
        let xml = common::xml_in(&idml, name);
        let parsed = roxmltree::Document::parse(&xml).unwrap();
        for tf in parsed.descendants().filter(|n| n.has_tag_name("TextFrame")) {
            frames += 1;
            let parent = tf.attribute("ParentStory").expect("ParentStory");
            let story_name = format!("Stories/Story_{parent}.xml");
            assert!(
                names.iter().any(|n| n == &story_name),
                "missing {story_name}"
            );
            let story = common::xml_in(&idml, &story_name);
            let sdoc = roxmltree::Document::parse(&story).unwrap();
            let self_id = sdoc
                .descendants()
                .find(|n| n.has_tag_name("Story") && n.attribute("Self").is_some())
                .and_then(|n| n.attribute("Self"));
            assert_eq!(self_id, Some(parent), "{story_name} Self");
        }
    }
    assert!(frames > 0, "expected at least one TextFrame");
}

#[test]
fn textbox_center_near_lock_rect() {
    let doc = common::invoice();
    let skip = running_ids(&doc);
    let lock = doc.lock().unwrap();
    let plan = lock.render_plan.pages.iter().find(|p| p.index == 0).unwrap();
    let page = &lock.geometry.pages[0];
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
    let space = SpreadSpace::new(page.width, page.height);
    let (want_tx, want_ty) = space.box_center(&rect);
    let xml = common::xml_in(&export_opened(&doc).unwrap(), "Spreads/Spread_k0.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let tf = parsed
        .descendants()
        .find(|n| n.has_tag_name("TextFrame") && n.attribute("Name") == Some(node_id.as_str()))
        .expect("TextFrame Name=node_id");
    let tfm = tf.attribute("ItemTransform").expect("ItemTransform");
    let parts: Vec<&str> = tfm.split_whitespace().collect();
    assert!(parts.len() >= 6, "{tfm}");
    let tx: f64 = parts[4].parse().unwrap();
    let ty: f64 = parts[5].parse().unwrap();
    assert!((tx - want_tx).abs() <= 0.02, "tx {tx} vs {want_tx}");
    assert!((ty - want_ty).abs() <= 0.02, "ty {ty} vs {want_ty}");
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
        TextAlign::Left
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
fn align_modes_fixture_writes_justification() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/align_modes.K2F");
    if !path.exists() {
        return;
    }
    let idml = export_opened(
        &k2f_paint::OpenedDocument::open(&std::fs::read(&path).unwrap()).unwrap(),
    )
    .unwrap();
    let names = common::unzip_names(&idml);
    for (id, want) in [
        ("txt_start", "LeftAlign"),
        ("txt_center", "CenterAlign"),
        ("txt_end", "RightAlign"),
    ] {
        let mut parent = None;
        for name in &names {
            if !name.starts_with("Spreads/") {
                continue;
            }
            let xml = common::xml_in(&idml, name);
            let parsed = roxmltree::Document::parse(&xml).unwrap();
            if let Some(tf) = parsed
                .descendants()
                .find(|n| n.has_tag_name("TextFrame") && n.attribute("Name") == Some(id))
            {
                parent = tf.attribute("ParentStory").map(str::to_string);
                break;
            }
        }
        let parent = parent.unwrap_or_else(|| panic!("no TextFrame {id}"));
        let story = common::xml_in(&idml, &format!("Stories/Story_{parent}.xml"));
        let parsed = roxmltree::Document::parse(&story).unwrap();
        let got = parsed
            .descendants()
            .find(|n| n.has_tag_name("ParagraphStyleRange"))
            .and_then(|n| n.attribute("Justification"));
        assert_eq!(got, Some(want), "{id} Justification");
    }
}

#[test]
fn paint_runs_keep_per_run_color_and_size() {
    let text = "HelloWorld";
    let xml = story_from(
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
    let ranges: Vec<_> = doc
        .descendants()
        .filter(|n| n.has_tag_name("CharacterStyleRange"))
        .collect();
    let colors: Vec<_> = ranges
        .iter()
        .filter_map(|n| n.attribute("FillColor"))
        .collect();
    let sizes: Vec<_> = ranges
        .iter()
        .filter_map(|n| n.attribute("PointSize"))
        .collect();
    assert!(colors.iter().any(|c| c.contains("FF0000")), "{colors:?}");
    assert!(colors.iter().any(|c| c.contains("0000FF")), "{colors:?}");
    assert!(sizes.iter().any(|s| *s == "12.000"), "{sizes:?}");
    assert!(sizes.iter().any(|s| *s == "8.000"), "{sizes:?}");
}

#[test]
fn subscript_is_position_not_italic() {
    let text = "H2O";
    let xml = story_from(
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
        .find(|n| {
            n.has_tag_name("CharacterStyleRange") && n.attribute("Position") == Some("Subscript")
        })
        .expect("Position=Subscript");
    let fs = sub.attribute("FontStyle").unwrap_or("");
    assert_ne!(fs, "Italic", "subscript must not fake italic: {xml}");
}

#[test]
fn left_align_left_inset_not_zero_when_glyphs_inset() {
    let xml = frame_from(
        "A",
        vec![],
        vec![TextGlyphRun {
            glyph_range: [0, 1],
            style: style("#111111", 12_000),
        }],
        vec![glyph(0, 8_000, 10_000, 12_000)],
        100_000,
    );
    assert!(
        xml.contains("8.000"),
        "left inset should be 8.000 pt, got {xml}"
    );
    let wrapped = format!("<root>{xml}</root>");
    let parsed = roxmltree::Document::parse(&wrapped).expect("frame xml");
    let el = parsed
        .descendants()
        .find(|n| n.has_tag_name("TextFramePreference"))
        .expect("TextFramePreference");
    let inset = el.attribute("InsetSpacing").expect("InsetSpacing");
    let parts: Vec<&str> = inset.split_whitespace().collect();
    assert_eq!(parts.len(), 4, "{inset}");
    assert_eq!(parts[1], "8.000", "left inset {inset}");
}

#[test]
fn running_footer_not_duplicated_on_body_spreads() {
    let doc = common::invoice();
    if doc.running_blocks().is_empty() {
        return;
    }
    let mut footer = None;
    let mut footer_ids = HashSet::new();
    for rb in doc.running_blocks() {
        for_each_node(&rb.node, &mut |n| {
            footer_ids.insert(n.id.clone());
            if footer.is_none() {
                if let Some(t) = node_text(n) {
                    if !t.is_empty() {
                        footer = Some(t.to_string());
                    }
                }
            }
        });
    }
    let Some(footer) = footer else {
        return;
    };
    let idml = export_opened(&doc).unwrap();
    let names = common::unzip_names(&idml);
    for name in &names {
        if !name.starts_with("Spreads/Spread_") {
            continue;
        }
        let xml = common::xml_in(&idml, name);
        assert!(
            !xml.contains(&footer),
            "body spread must not copy running text: {name}"
        );
        let parsed = roxmltree::Document::parse(&xml).unwrap();
        for tf in parsed.descendants().filter(|n| n.has_tag_name("TextFrame")) {
            if let Some(nm) = tf.attribute("Name") {
                assert!(
                    !footer_ids.contains(nm),
                    "running node {nm} must not be a body TextFrame"
                );
            }
        }
    }
    let master = common::xml_in(&idml, "MasterSpreads/MasterSpread_kMaster.xml");
    let mut hit = master.contains(&footer);
    let parsed = roxmltree::Document::parse(&master).unwrap();
    for tf in parsed.descendants().filter(|n| n.has_tag_name("TextFrame")) {
        if let Some(parent) = tf.attribute("ParentStory") {
            let story = common::xml_in(&idml, &format!("Stories/Story_{parent}.xml"));
            if story.contains(&footer)
                || story.contains("AutoPageNumber")
                || stories_blob(&idml).contains(
                    footer
                        .replace("{{page_current}}", "")
                        .replace("{{page_total}}", "")
                        .trim(),
                )
            {
                hit = true;
            }
        }
    }
    assert!(hit, "running footer must appear on master or master stories");
}

#[test]
fn auto_page_number_in_master_if_page_current() {
    let doc = common::invoice();
    let mut has_token = false;
    for rb in doc.running_blocks() {
        for_each_node(&rb.node, &mut |n| {
            if node_text(n).is_some_and(|t| t.contains("{{page_current}}")) {
                has_token = true;
            }
        });
    }
    if !has_token {
        return;
    }
    let idml = export_opened(&doc).unwrap();
    let master = common::xml_in(&idml, "MasterSpreads/MasterSpread_kMaster.xml");
    let parsed = roxmltree::Document::parse(&master).unwrap();
    let mut found = false;
    for tf in parsed.descendants().filter(|n| n.has_tag_name("TextFrame")) {
        let Some(parent) = tf.attribute("ParentStory") else {
            continue;
        };
        let story = common::xml_in(&idml, &format!("Stories/Story_{parent}.xml"));
        if story.contains("AutoPageNumber") {
            found = true;
            assert!(
                !story.contains("{{page_current}}"),
                "must not bake {{{{page_current}}}}"
            );
        }
    }
    assert!(found, "master story must contain AutoPageNumber");
}

#[test]
fn composer_is_single_line() {
    let idml = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&idml, "Spreads/Spread_k0.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let parent = parsed
        .descendants()
        .find(|n| n.has_tag_name("TextFrame"))
        .and_then(|n| n.attribute("ParentStory"))
        .expect("body TextFrame");
    let story = common::xml_in(&idml, &format!("Stories/Story_{parent}.xml"));
    let sdoc = roxmltree::Document::parse(&story).unwrap();
    let para = sdoc
        .descendants()
        .find(|n| n.has_tag_name("ParagraphStyleRange"))
        .expect("ParagraphStyleRange");
    assert_eq!(para.attribute("Hyphenation"), Some("false"));
    let props = para
        .children()
        .find(|n| n.has_tag_name("Properties"))
        .map(|n| n.document().input_text()[n.range()].to_string())
        .unwrap_or_default();
    assert!(
        props.contains("$ID/HL Single"),
        "composer must be HL Single, got {story}"
    );
}

#[test]
fn xml_escape_unit() {
    assert_eq!(escape_xml("a&b<c>"), "a&amp;b&lt;c&gt;");
}

#[test]
fn export_still_byte_identical() {
    let doc = common::invoice();
    assert_eq!(export_opened(&doc).unwrap(), export_opened(&doc).unwrap());
}

#[test]
fn no_k2f_raster_yet() {
    let idml = export_opened(&common::invoice()).unwrap();
    for name in common::unzip_names(&idml) {
        if !name.ends_with(".xml") {
            continue;
        }
        let xml = common::xml_in(&idml, &name);
        assert!(
            !xml.contains("k2f-raster:"),
            "step 2 must not emit rasters in {name}"
        );
    }
}
