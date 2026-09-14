use super::graphic;
use super::ids::{story_self, tf_self};
use super::skeleton;
use super::spread;
use super::story;
use crate::coord::SpreadSpace;
use crate::ir::{DocIR, PageElement, TextBox};
use crate::IdmlError;
use k2f_core::LockFile;
use k2f_paint::OpenedDocument;
use std::collections::BTreeMap;

pub fn build_package(
    doc: &OpenedDocument,
    lock: &LockFile,
    ir: &DocIR,
) -> Result<BTreeMap<String, Vec<u8>>, IdmlError> {
    let n = lock.geometry.pages.len();
    if n == 0 {
        return Err(IdmlError::Write("package has no pages".into()));
    }
    let page0 = &lock.geometry.pages[0];
    let space = SpreadSpace::new(page0.width, page0.height);
    let mut fills = Vec::with_capacity(n);
    let mut swatches = ir.collect_color_hexes();
    for (i, page) in lock.geometry.pages.iter().enumerate() {
        let ops = lock
            .render_plan
            .pages
            .get(i)
            .map(|p| p.ops.as_slice())
            .unwrap_or(&[]);
        let hex = spread::page_fill_hex(page, ops)?;
        if let Some(ref h) = hex {
            swatches.insert(h.clone());
        }
        fills.push(hex);
    }
    let mut next_st = 0usize;
    let mut next_tf = 0usize;
    let mut story_srcs = Vec::new();
    let mut files = BTreeMap::new();
    let mut page_frames = Vec::with_capacity(n);
    for page in &ir.pages {
        let mut frames = String::new();
        emit_text_elements(
            &page.elements,
            &space,
            &mut next_st,
            &mut next_tf,
            &mut story_srcs,
            &mut frames,
            &mut files,
        );
        page_frames.push(frames);
    }
    let mut master_frames = String::new();
    emit_text_elements(
        &ir.master,
        &space,
        &mut next_st,
        &mut next_tf,
        &mut story_srcs,
        &mut master_frames,
        &mut files,
    );
    files.insert(
        "designmap.xml".into(),
        skeleton::designmap_xml(n, &story_srcs).into_bytes(),
    );
    files.insert(
        "META-INF/container.xml".into(),
        skeleton::CONTAINER_XML.as_bytes().to_vec(),
    );
    files.insert(
        "META-INF/metadata.xml".into(),
        skeleton::metadata_xml(doc.title()).into_bytes(),
    );
    files.insert(
        "Resources/Fonts.xml".into(),
        graphic::fonts_xml(&ir.fonts).into_bytes(),
    );
    files.insert(
        "Resources/Graphic.xml".into(),
        graphic::graphic_xml(&swatches).into_bytes(),
    );
    files.insert(
        "Resources/Preferences.xml".into(),
        skeleton::preferences_xml(&space).into_bytes(),
    );
    files.insert(
        "Resources/Styles.xml".into(),
        skeleton::STYLES_XML.as_bytes().to_vec(),
    );
    files.insert(
        "MasterSpreads/MasterSpread_kMaster.xml".into(),
        skeleton::master_xml(&space, &master_frames).into_bytes(),
    );
    files.insert(
        "XML/BackingStory.xml".into(),
        skeleton::BACKING_XML.as_bytes().to_vec(),
    );
    files.insert(
        "XML/Tags.xml".into(),
        skeleton::TAGS_XML.as_bytes().to_vec(),
    );
    for (i, fill) in fills.iter().enumerate() {
        files.insert(
            format!("Spreads/Spread_k{i}.xml"),
            spread::spread_xml(i, &space, fill.as_deref(), &page_frames[i]).into_bytes(),
        );
    }
    Ok(files)
}

fn emit_text_elements(
    elements: &[PageElement],
    space: &SpreadSpace,
    next_st: &mut usize,
    next_tf: &mut usize,
    story_srcs: &mut Vec<String>,
    frames: &mut String,
    files: &mut BTreeMap<String, Vec<u8>>,
) {
    for el in elements {
        let Some(tb) = el.textbox() else {
            continue;
        };
        emit_textbox(tb, space, next_st, next_tf, story_srcs, frames, files);
    }
}

fn emit_textbox(
    tb: &TextBox,
    space: &SpreadSpace,
    next_st: &mut usize,
    next_tf: &mut usize,
    story_srcs: &mut Vec<String>,
    frames: &mut String,
    files: &mut BTreeMap<String, Vec<u8>>,
) {
    let st = story_self(*next_st);
    *next_st += 1;
    let tf = tf_self(*next_tf);
    *next_tf += 1;
    let src = format!("Stories/Story_{st}.xml");
    files.insert(src.clone(), story::story_xml(tb, &st).into_bytes());
    story_srcs.push(src);
    frames.push_str(&spread::textframe_xml(tb, space, &tf, &st));
}
