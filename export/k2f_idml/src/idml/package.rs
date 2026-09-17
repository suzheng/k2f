use super::graphic;
use super::ids::{img_self, rect_self, story_self, tbl_self, tf_self};
use super::skeleton;
use super::spread;
use super::story;
use crate::coord::SpreadSpace;
use crate::ir::{DocIR, PageElement, TableBox, TextAlign, TextBox};
use crate::IdmlError;
use k2f_core::LockFile;
use k2f_paint::OpenedDocument;
use std::collections::BTreeMap;

struct EmitIds {
    next_st: usize,
    next_tf: usize,
    next_rect: usize,
    next_img: usize,
    next_tbl: usize,
}

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
    let swatches = ir.collect_color_hexes();
    let mut ids = EmitIds {
        next_st: 0,
        next_tf: 0,
        next_rect: 0,
        next_img: 0,
        next_tbl: 0,
    };
    let mut story_srcs = Vec::new();
    let mut files = BTreeMap::new();
    let mut page_frames = Vec::with_capacity(n);
    for page in &ir.pages {
        let mut frames = String::new();
        emit_elements(
            &page.elements,
            &space,
            &mut ids,
            &mut story_srcs,
            &mut frames,
            &mut files,
            false,
        );
        page_frames.push(frames);
    }
    let mut master_frames = String::new();
    emit_elements(
        &ir.master,
        &space,
        &mut ids,
        &mut story_srcs,
        &mut master_frames,
        &mut files,
        true,
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
    for (i, frames) in page_frames.iter().enumerate() {
        files.insert(
            format!("Spreads/Spread_k{i}.xml"),
            spread::spread_xml(i, &space, frames).into_bytes(),
        );
    }
    Ok(files)
}

fn emit_elements(
    elements: &[PageElement],
    space: &SpreadSpace,
    ids: &mut EmitIds,
    story_srcs: &mut Vec<String>,
    frames: &mut String,
    files: &mut BTreeMap<String, Vec<u8>>,
    on_master: bool,
) {
    for el in elements {
        match el {
            PageElement::TextBox(tb) => {
                emit_textbox(tb, space, ids, story_srcs, frames, files, on_master);
            }
            PageElement::Shape(shape) => {
                let id = rect_self(ids.next_rect);
                ids.next_rect += 1;
                frames.push_str(&spread::rectangle_xml_on(shape, space, &id, on_master));
            }
            PageElement::Picture(pic) => {
                emit_picture(pic, space, ids, frames, false, on_master);
            }
            PageElement::Table(tbl) => {
                emit_table(tbl, space, ids, story_srcs, frames, files, on_master);
            }
            PageElement::Raster(pic) => {
                emit_picture(pic, space, ids, frames, true, on_master);
            }
        }
    }
}

fn emit_picture(
    pic: &crate::ir::PictureBox,
    space: &SpreadSpace,
    ids: &mut EmitIds,
    frames: &mut String,
    raster: bool,
    on_master: bool,
) {
    let rid = rect_self(ids.next_rect);
    ids.next_rect += 1;
    let iid = img_self(ids.next_img);
    ids.next_img += 1;
    let xml = if raster {
        spread::raster_xml_on(pic, space, &rid, &iid, on_master)
    } else {
        spread::picture_xml_on(pic, space, &rid, &iid, on_master)
    };
    frames.push_str(&xml);
}

fn emit_textbox(
    tb: &TextBox,
    space: &SpreadSpace,
    ids: &mut EmitIds,
    story_srcs: &mut Vec<String>,
    frames: &mut String,
    files: &mut BTreeMap<String, Vec<u8>>,
    on_master: bool,
) {
    let st = story_self(ids.next_st);
    ids.next_st += 1;
    let tf = tf_self(ids.next_tf);
    ids.next_tf += 1;
    let src = format!("Stories/Story_{st}.xml");
    files.insert(src.clone(), story::story_xml(tb, &st).into_bytes());
    story_srcs.push(src);
    frames.push_str(&spread::textframe_xml_on(tb, space, &tf, &st, on_master));
}

fn emit_table(
    tbl: &TableBox,
    space: &SpreadSpace,
    ids: &mut EmitIds,
    story_srcs: &mut Vec<String>,
    frames: &mut String,
    files: &mut BTreeMap<String, Vec<u8>>,
    on_master: bool,
) {
    let st = story_self(ids.next_st);
    ids.next_st += 1;
    let tf = tf_self(ids.next_tf);
    ids.next_tf += 1;
    let tbl_id = tbl_self(ids.next_tbl);
    ids.next_tbl += 1;
    let src = format!("Stories/Story_{st}.xml");
    files.insert(
        src.clone(),
        story::story_table_xml(tbl, &st, &tbl_id).into_bytes(),
    );
    story_srcs.push(src);
    let tb = TextBox {
        node_id: tbl.node_id.clone(),
        rect: tbl.rect.clone(),
        runs: Vec::new(),
        align: TextAlign::Left,
        inset_top: 0.0,
        inset_left: 0.0,
        inset_bottom: 0.0,
        inset_right: 0.0,
        first_line_indent_pt: 0.0,
        vert_center: false,
        autosize_width: false,
        autosize_refer: "CenterLeftPoint",
        autosize_no_wrap: false,
        autosize_height: false,
        no_break: false,
        semantic_newlines: false,
        lock_line_count: 0,
        first_baseline_leading_offset: false,
        full_width_lock_line: false,
    };
    frames.push_str(&spread::textframe_xml_on(&tb, space, &tf, &st, on_master));
}
