use crate::coord::pt_to_emu;
use crate::effect::{
    box_is_effect, consume_following_box, effect_box_crop, glass_crop, is_math_node, is_rule_id,
    keep_for_effect, math_crop, rasterize_slice,
};
use crate::ir::{DeckIR, SlideElement, SlideIR};
use crate::picture::picture_from_draw;
use crate::shape::shape_from_box;
use crate::table::{index_native_tables, paint_node_id, table_on_page, table_ref_placeholder};
use crate::text::{list_start_at, textbox_from_draw, FontCtx};
use crate::PptxError;
use k2f_core::{Fill, GeometryNode, Page, PaintOp, Pt, Rect};
use k2f_paint::{parse_hex_rgba, resolve_fill, OpenedDocument};
use std::collections::HashSet;

pub fn classify_opened(doc: &OpenedDocument) -> Result<DeckIR, PptxError> {
    let lock = doc.lock().ok_or(PptxError::Unlocked)?;
    if lock.has_unknown_paint_ops() {
        return Err(PptxError::UnknownOp);
    }
    let pages = &lock.geometry.pages;
    if pages.is_empty() {
        return Err(PptxError::Write("lock has no pages".into()));
    }
    let fonts = FontCtx::new(doc.fonts());
    let root = doc.semantic_root();
    let running = doc.running_blocks();
    let assets = doc.assets();
    let tables = index_native_tables(root, running);
    let list_starts = list_start_at(root);
    let mut media_n = 1u32;
    let mut raster_n = 1u32;
    let total_pages = pages.len();
    let mut slides = Vec::with_capacity(total_pages);
    for (page_idx, page) in pages.iter().enumerate() {
        let plan = lock
            .render_plan
            .pages
            .iter()
            .find(|p| p.index == page.index)
            .ok_or(k2f_paint::PaintError::MissingRenderPlan(page.index))?;
        let mut elements = Vec::new();
        let mut emitted_tables = HashSet::new();
        let mut skip = HashSet::new();
        for (i, op) in plan.ops.iter().enumerate() {
            if skip.contains(&i) {
                continue;
            }
            if let Some(tid) = paint_node_id(op).and_then(|id| tables.owner_of(id)) {
                if emitted_tables.contains(tid) {
                    continue;
                }
                if let Some(tbl) =
                    table_on_page(&tables, tid, page, &plan.ops, root, running, &fonts)?
                {
                    emitted_tables.insert(tid.to_string());
                    elements.push(SlideElement::Table(tbl));
                    continue;
                }
            }
            match op {
                PaintOp::Unknown => return Err(PptxError::UnknownOp),
                PaintOp::BackdropBlur { node_id, rect, .. } => {
                    let follow = consume_following_box(&plan.ops, i, node_id);
                    if let Some(j) = follow {
                        skip.insert(j);
                    }
                    let crop = glass_crop(rect, &plan.ops, follow)?;
                    let keep = keep_for_effect(node_id, page, false);
                    let pic = rasterize_slice(
                        lock,
                        page_idx,
                        page,
                        &plan.ops,
                        &keep,
                        crop,
                        node_id,
                        raster_n,
                        doc.fonts(),
                        assets,
                    )?;
                    raster_n = raster_n.saturating_add(1);
                    elements.push(SlideElement::Raster(pic));
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
                        let crop = math_crop(&plan.ops, node_id, rect);
                        let keep = keep_for_effect(node_id, page, true);
                        let pic = rasterize_slice(
                            lock,
                            page_idx,
                            page,
                            &plan.ops,
                            &keep,
                            crop,
                            node_id,
                            raster_n,
                            doc.fonts(),
                            assets,
                        )?;
                        raster_n = raster_n.saturating_add(1);
                        elements.push(SlideElement::Raster(pic));
                    } else {
                        let geo = find_geo(&page.root, node_id);
                        if let Some(tb) = textbox_from_draw(
                            node,
                            rect,
                            runs,
                            geo,
                            &fonts,
                            page_idx,
                            total_pages,
                            list_starts.get(&node.id).copied().unwrap_or(1),
                        ) {
                            elements.push(SlideElement::TextBox(tb));
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
                        let keep = keep_for_effect(node_id, page, false);
                        let pic = rasterize_slice(
                            lock,
                            page_idx,
                            page,
                            &plan.ops,
                            &keep,
                            crop,
                            node_id,
                            raster_n,
                            doc.fonts(),
                            assets,
                        )?;
                        raster_n = raster_n.saturating_add(1);
                        elements.push(SlideElement::Raster(pic));
                    } else if let Some(shape) = shape_from_box(node_id, rect, decoration)? {
                        elements.push(SlideElement::Shape(shape));
                    }
                }
                PaintOp::DrawImage { node_id, rect, src } => {
                    let pic = picture_from_draw(node_id, rect, src, assets, media_n)?;
                    media_n = media_n.saturating_add(1);
                    elements.push(SlideElement::Picture(pic));
                }
                PaintOp::DrawTableReference { node_id, rect, .. } => {
                    elements.push(SlideElement::Shape(table_ref_placeholder(node_id, rect)));
                }
            }
        }
        slides.push(SlideIR {
            width_emu: pt_to_emu(page.width),
            height_emu: pt_to_emu(page.height),
            bg_hex: page_bg_hex(page, &plan.ops),
            elements,
        });
    }
    Ok(DeckIR {
        title: doc.title().to_string(),
        slides,
    })
}

fn page_bg_hex(page: &Page, ops: &[PaintOp]) -> String {
    match ops.first() {
        Some(PaintOp::DrawBox {
            rect, decoration, ..
        }) if is_full_page(page, rect) => match resolve_fill(decoration) {
            Ok(Some(Fill::Solid { color })) => {
                opaque_srgb_hex(&color).unwrap_or_else(|| "FFFFFF".into())
            }
            _ => "FFFFFF".into(),
        },
        _ => "FFFFFF".into(),
    }
}

fn is_full_page(page: &Page, rect: &Rect) -> bool {
    rect.x == Pt(0) && rect.y == Pt(0) && rect.width == page.width && rect.height == page.height
}

fn opaque_srgb_hex(color: &str) -> Option<String> {
    let [r, g, b, a] = parse_hex_rgba(color)?;
    if a < 255 {
        return None;
    }
    Some(format!("{r:02X}{g:02X}{b:02X}"))
}

fn find_geo<'a>(node: &'a GeometryNode, id: &str) -> Option<&'a GeometryNode> {
    if node.id == id {
        return Some(node);
    }
    node.children.iter().find_map(|c| find_geo(c, id))
}
