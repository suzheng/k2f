use super::{keep_node, Layer};
use crate::effect::{
    box_is_effect, consume_following_box, effect_box_crop, glass_crop, is_math_node, is_rule_id,
    keep_for_effect, math_crop, rasterize_slice,
};
use crate::geo::find_geo;
use crate::ir::PageElement;
use crate::picture::picture_from_draw;
use crate::shape::shapes_from_box;
use crate::table::{paint_node_id, table_ref_placeholder, take_table, TakeTable};
use crate::text::{textbox_from_draw_ctx, TextFonts};
use crate::IdmlError;
use k2f_core::{LockFile, Page, PaintOp};
use k2f_paint::OpenedDocument;
use std::collections::{BTreeMap, HashSet};

pub(super) fn scan_ops(
    ops: &[PaintOp],
    page: &Page,
    page_idx: usize,
    lock: &LockFile,
    layer: Layer,
    doc: &OpenedDocument,
    fonts: &TextFonts,
    running_ids: &HashSet<String>,
    list_starts: &BTreeMap<String, u32>,
    tables: &crate::table::TableIndex,
    media_n: &mut u32,
    raster_n: &mut u32,
) -> Result<Vec<PageElement>, IdmlError> {
    let root = doc.semantic_root();
    let running = doc.running_blocks();
    let assets = doc.assets();
    let master_pages = match layer {
        Layer::Body => None,
        Layer::Master { total_pages } => Some(total_pages),
    };
    let mut elements = Vec::new();
    let mut seen_master = HashSet::new();
    let mut emitted_tables = HashSet::new();
    let mut skip = HashSet::new();
    for (i, op) in ops.iter().enumerate() {
        if skip.contains(&i) {
            continue;
        }
        if let Some(nid) = paint_node_id(op) {
            if !keep_node(nid, running_ids, master_pages.is_some(), &mut seen_master) {
                continue;
            }
        }
        match take_table(
            op,
            tables,
            &mut emitted_tables,
            page,
            ops,
            root,
            running,
            fonts,
        )? {
            TakeTable::Skip => continue,
            TakeTable::Built(tbl) => {
                elements.push(PageElement::Table(tbl));
                continue;
            }
            TakeTable::NotMember => {}
        }
        match op {
            PaintOp::Unknown => return Err(IdmlError::UnknownOp),
            PaintOp::BackdropBlur { node_id, rect, .. } => {
                let follow = consume_following_box(ops, i, node_id);
                if let Some(j) = follow {
                    skip.insert(j);
                }
                let crop = glass_crop(rect, ops, follow)?;
                let pic = slice(
                    lock, page_idx, page, ops, node_id, crop, false, raster_n, doc,
                )?;
                elements.push(PageElement::Raster(pic));
            }
            PaintOp::DrawText {
                node_id,
                rect,
                runs,
            } => {
                let Some(node) = k2f_core::find_in_trees(root, running, node_id) else {
                    continue;
                };
                if is_math_node(node) {
                    let crop = math_crop(ops, node_id, rect);
                    let pic = slice(
                        lock, page_idx, page, ops, node_id, crop, true, raster_n, doc,
                    )?;
                    elements.push(PageElement::Raster(pic));
                    continue;
                }
                let geo = find_geo(&page.root, node_id);
                if let Some(tb) = textbox_from_draw_ctx(
                    node,
                    rect,
                    runs,
                    geo,
                    fonts,
                    list_starts.get(&node.id).copied().unwrap_or(1),
                    master_pages,
                ) {
                    elements.push(PageElement::TextBox(tb));
                }
            }
            PaintOp::DrawBox {
                node_id,
                rect,
                decoration,
            } => {
                if is_rule_id(node_id) {
                    continue;
                }
                if box_is_effect(node_id, decoration)? {
                    let crop = effect_box_crop(rect, decoration)?;
                    let pic = slice(
                        lock, page_idx, page, ops, node_id, crop, false, raster_n, doc,
                    )?;
                    elements.push(PageElement::Raster(pic));
                    continue;
                }
                for shape in shapes_from_box(node_id, rect, decoration)? {
                    elements.push(PageElement::Shape(shape));
                }
            }
            PaintOp::DrawImage { node_id, rect, src } => {
                let pic = picture_from_draw(node_id, rect, src, assets, *media_n)?;
                *media_n = media_n.saturating_add(1);
                elements.push(PageElement::Picture(pic));
            }
            PaintOp::DrawTableReference { node_id, rect, .. } => {
                elements.push(PageElement::Shape(table_ref_placeholder(node_id, rect)));
            }
        }
    }
    Ok(elements)
}

fn slice(
    lock: &LockFile,
    page_idx: usize,
    page: &Page,
    ops: &[PaintOp],
    node_id: &str,
    crop: k2f_core::Rect,
    math: bool,
    raster_n: &mut u32,
    doc: &OpenedDocument,
) -> Result<crate::ir::PictureBox, IdmlError> {
    let keep = keep_for_effect(node_id, page, math);
    let pic = rasterize_slice(
        lock,
        page_idx,
        page,
        ops,
        &keep,
        crop,
        node_id,
        *raster_n,
        doc.fonts(),
        doc.assets(),
    )?;
    *raster_n = raster_n.saturating_add(1);
    Ok(pic)
}
