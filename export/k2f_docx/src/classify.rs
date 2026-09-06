use crate::coord::{pt_to_emu, pt_to_twips};
use crate::effect::{
    box_is_effect, consume_following_box, effect_box_crop, glass_crop, is_math_node, is_rule_id,
    keep_for_effect, math_crop, rasterize_slice,
};
use crate::geo::{find_geo, page_bg_hex};
use crate::ir::{DocIR, PageElement, PageIR, PictureBox};
use crate::picture::picture_from_draw;
use crate::shape::shapes_from_box;
use crate::table::{index_native_tables, paint_node_id, table_on_page, table_ref_placeholder};
use crate::text::{font_ctx, textbox_from_draw_ctx};
use crate::DocxError;
use k2f_core::{LockFile, Page, PaintOp};
use k2f_paint::OpenedDocument;
use std::collections::{BTreeMap, HashSet};

pub fn classify_opened(doc: &OpenedDocument) -> Result<DocIR, DocxError> {
    let lock = doc.lock().ok_or(DocxError::Unlocked)?;
    if lock.has_unknown_paint_ops() {
        return Err(DocxError::UnknownOp);
    }
    let pages = &lock.geometry.pages;
    if pages.is_empty() {
        return Err(DocxError::Write("lock has no pages".into()));
    }
    let fonts = font_ctx(doc.fonts());
    let root = doc.semantic_root();
    let running = doc.running_blocks();
    let skip_running = crate::header::running_field_ids(running);
    let assets = doc.assets();
    let tables = index_native_tables(root, running);
    let mut media_n = 1u32;
    let mut raster_n = 1u32;
    let mut ir_pages = Vec::with_capacity(pages.len());
    for (page_idx, page) in pages.iter().enumerate() {
        let ops = lock
            .render_plan
            .pages
            .get(page_idx)
            .map(|p| p.ops.as_slice())
            .unwrap_or(&[]);
        let mut elements = Vec::new();
        let mut emitted_tables = HashSet::new();
        let mut skip_ops = HashSet::new();
        let bg_hex = page_bg_hex(page, ops);
        for (i, op) in ops.iter().enumerate() {
            if skip_ops.contains(&i) {
                continue;
            }
            let rel = rel_height(i);
            if let Some(nid) = paint_node_id(op) {
                if skip_running.contains(nid) {
                    continue;
                }
                if let Some(tid) = tables.owner_of(nid) {
                    if emitted_tables.contains(tid) {
                        continue;
                    }
                    if let Some(tbl) =
                        table_on_page(&tables, tid, page, ops, root, running, &fonts, rel)?
                    {
                        emitted_tables.insert(tid.to_string());
                        elements.push(PageElement::Table(tbl));
                        continue;
                    }
                }
            }
            match op {
                PaintOp::Unknown => return Err(DocxError::UnknownOp),
                PaintOp::BackdropBlur { node_id, rect, .. } => {
                    let follow = consume_following_box(ops, i, node_id);
                    if let Some(j) = follow {
                        skip_ops.insert(j);
                    }
                    let crop = glass_crop(rect, ops, follow)?;
                    elements.push(PageElement::Raster(slice(
                        lock,
                        page_idx,
                        page,
                        ops,
                        node_id,
                        crop,
                        false,
                        raster_n,
                        rel,
                        doc.fonts(),
                        assets,
                    )?));
                    raster_n = raster_n.saturating_add(1);
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
                        elements.push(PageElement::Raster(slice(
                            lock,
                            page_idx,
                            page,
                            ops,
                            node_id,
                            crop,
                            true,
                            raster_n,
                            rel,
                            doc.fonts(),
                            assets,
                        )?));
                        raster_n = raster_n.saturating_add(1);
                    } else {
                        let geo = find_geo(&page.root, node_id);
                        if let Some(tb) = textbox_from_draw_ctx(node, rect, runs, geo, &fonts, rel)
                        {
                            elements.push(PageElement::TextBox(tb));
                        }
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
                        elements.push(PageElement::Raster(slice(
                            lock,
                            page_idx,
                            page,
                            ops,
                            node_id,
                            crop,
                            false,
                            raster_n,
                            rel,
                            doc.fonts(),
                            assets,
                        )?));
                        raster_n = raster_n.saturating_add(1);
                    } else {
                        for s in shapes_from_box(
                            node_id,
                            rect,
                            decoration,
                            page.width,
                            page.height,
                            rel,
                            &bg_hex,
                        )? {
                            elements.push(PageElement::Shape(s));
                        }
                    }
                }
                PaintOp::DrawImage { node_id, rect, src } => {
                    if crate::geo::image_occluded_by_later_opaque_box(rect, &ops[i + 1..]) {
                        continue;
                    }
                    let pic = picture_from_draw(node_id, rect, src, assets, media_n, rel)?;
                    media_n = media_n.saturating_add(1);
                    elements.push(PageElement::Picture(pic));
                }
                PaintOp::DrawTableReference { node_id, rect, .. } => {
                    elements.push(PageElement::Shape(table_ref_placeholder(
                        node_id, rect, rel,
                    )));
                }
            }
        }
        assign_textbox_underlays(&mut elements, &bg_hex);
        absorb_self_fill_shapes(&mut elements);
        ir_pages.push(PageIR { bg_hex, elements });
    }
    let page0 = &pages[0];
    let (mut header, mut footer) =
        crate::header::collect_running(doc, lock, &fonts, &skip_running, assets, &mut media_n)?;
    let paper = ir_pages
        .first()
        .map(|p| p.bg_hex.as_str())
        .unwrap_or("FFFFFE");
    assign_textbox_underlays(&mut header, paper);
    assign_textbox_underlays(&mut footer, paper);
    absorb_self_fill_shapes(&mut header);
    absorb_self_fill_shapes(&mut footer);
    Ok(DocIR {
        title: doc.title().to_string(),
        page_width_emu: pt_to_emu(page0.width),
        page_height_emu: pt_to_emu(page0.height),
        page_width_twips: pt_to_twips(page0.width),
        page_height_twips: pt_to_twips(page0.height),
        pages: ir_pages,
        header,
        footer,
    })
}

/// Word Dark Mode inverts text in `noFill` floating boxes. Paint an opaque
/// underlay matching the shape behind the box (or the page paper).
fn assign_textbox_underlays(elements: &mut [PageElement], paper_hex: &str) {
    let shapes: Vec<(u32, i64, i64, i64, i64, String)> = elements
        .iter()
        .filter_map(|el| match el {
            PageElement::Shape(s) => s.fill_hex.as_ref().map(|h| {
                (
                    s.relative_height,
                    s.x_emu,
                    s.y_emu,
                    s.cx_emu,
                    s.cy_emu,
                    h.clone(),
                )
            }),
            _ => None,
        })
        .collect();
    for el in elements.iter_mut() {
        let PageElement::TextBox(tb) = el else {
            continue;
        };
        let px = tb.x_emu.saturating_add(tb.cx_emu / 2);
        let py = tb.y_emu.saturating_add(tb.cy_emu / 2);
        let mut found = None;
        for (rel, x, y, w, h, hex) in &shapes {
            if *rel >= tb.relative_height {
                continue;
            }
            if px >= *x && py >= *y && px < x.saturating_add(*w) && py < y.saturating_add(*h) {
                found = Some(hex.clone());
            }
        }
        let hex = found.unwrap_or_else(|| paper_hex.to_string());
        tb.fill_hex = Some(pin_underlay_hex(&hex));
    }
}

/// A DrawBox + DrawText for the same node (pills, chips) must be one Word
/// shape. LibreOffice paints the later roundRect on top of the text box and
/// hides white labels. Fold the corner radius into the text box and drop the
/// duplicate fill shape.
fn absorb_self_fill_shapes(elements: &mut Vec<PageElement>) {
    let corners: BTreeMap<String, i64> = elements
        .iter()
        .filter_map(|el| match el {
            PageElement::Shape(s) => Some((s.node_id.clone(), s.corner_emu)),
            _ => None,
        })
        .collect();
    let mut absorbed = HashSet::new();
    for el in elements.iter_mut() {
        let PageElement::TextBox(tb) = el else {
            continue;
        };
        if let Some(corner) = corners.get(&tb.node_id) {
            tb.corner_emu = *corner;
            absorbed.insert(tb.node_id.clone());
        }
    }
    if absorbed.is_empty() {
        return;
    }
    elements.retain(|el| match el {
        PageElement::Shape(s) => !absorbed.contains(&s.node_id),
        _ => true,
    });
}

fn pin_underlay_hex(hex: &str) -> String {
    match hex.to_ascii_uppercase().as_str() {
        "FFFFFF" => "FFFFFE".into(),
        "000000" => "000001".into(),
        other => other.to_string(),
    }
}

fn slice(
    lock: &LockFile,
    page_idx: usize,
    page: &Page,
    ops: &[PaintOp],
    node_id: &str,
    crop: k2f_core::Rect,
    math: bool,
    raster_n: u32,
    rel: u32,
    fonts: &BTreeMap<String, Vec<u8>>,
    assets: &BTreeMap<String, Vec<u8>>,
) -> Result<PictureBox, DocxError> {
    let keep = keep_for_effect(node_id, page, math);
    rasterize_slice(
        lock, page_idx, page, ops, &keep, crop, node_id, raster_n, rel, fonts, assets,
    )
}

fn rel_height(op_index: usize) -> u32 {
    u32::try_from(op_index.saturating_mul(10)).unwrap_or(u32::MAX)
}
