use crate::geo::find_geo;
use crate::ir::PageElement;
use crate::picture::picture_from_draw;
use crate::shape::shapes_from_box;
use crate::text::{textbox_from_draw_ctx, FontCtx};
use crate::DocxError;
use k2f_core::{for_each_node, LockFile, PaintOp, RunningBlockNode, RunningBlockPosition};
use k2f_paint::OpenedDocument;
use std::collections::{BTreeMap, HashSet};

pub(crate) fn running_ids(running: &[RunningBlockNode]) -> HashSet<String> {
    let mut ids = HashSet::new();
    for rb in running {
        for_each_node(&rb.node, &mut |n| {
            ids.insert(n.id.clone());
        });
    }
    ids
}

pub(crate) fn collect_running(
    doc: &OpenedDocument,
    lock: &LockFile,
    fonts: &FontCtx,
    ids: &HashSet<String>,
    assets: &BTreeMap<String, Vec<u8>>,
    media_n: &mut u32,
) -> Result<(Vec<PageElement>, Vec<PageElement>), DocxError> {
    let mut header = Vec::new();
    let mut footer = Vec::new();
    let mut seen = HashSet::new();
    let root = doc.semantic_root();
    let running = doc.running_blocks();
    let Some(page) = lock.geometry.pages.first() else {
        return Ok((header, footer));
    };
    let Some(plan) = lock
        .render_plan
        .pages
        .iter()
        .find(|p| p.index == page.index)
    else {
        return Ok((header, footer));
    };
    for (i, op) in plan.ops.iter().enumerate() {
        let Some(node_id) = op_node_id(op) else {
            continue;
        };
        if !ids.contains(node_id) {
            continue;
        }
        let kind = op_kind(op);
        if !seen.insert((kind, node_id.to_string())) {
            continue;
        }
        let Some(pos) = position_of(running, node_id) else {
            continue;
        };
        let rel = u32::try_from(i.saturating_mul(10)).unwrap_or(u32::MAX);
        let mut els = Vec::new();
        match op {
            PaintOp::DrawText { rect, runs, .. } => {
                let Some(node) = k2f_core::find_in_trees(root, running, node_id) else {
                    continue;
                };
                let geo = find_geo(&page.root, node_id);
                if let Some(tb) = textbox_from_draw_ctx(node, rect, runs, geo, fonts, rel) {
                    els.push(PageElement::TextBox(tb));
                }
            }
            PaintOp::DrawBox {
                rect, decoration, ..
            } => {
                els.extend(
                    shapes_from_box(node_id, rect, decoration, page.width, page.height, rel)?
                        .into_iter()
                        .map(PageElement::Shape),
                );
            }
            PaintOp::DrawImage { rect, src, .. } => {
                let pic = picture_from_draw(node_id, rect, src, assets, *media_n, rel)?;
                *media_n = media_n.saturating_add(1);
                els.push(PageElement::Picture(pic));
            }
            _ => continue,
        }
        let dest = match pos {
            RunningBlockPosition::Header => &mut header,
            RunningBlockPosition::Footer => &mut footer,
        };
        dest.extend(els);
    }
    Ok((header, footer))
}

fn op_node_id(op: &PaintOp) -> Option<&str> {
    match op {
        PaintOp::DrawText { node_id, .. }
        | PaintOp::DrawBox { node_id, .. }
        | PaintOp::DrawImage { node_id, .. }
        | PaintOp::BackdropBlur { node_id, .. }
        | PaintOp::DrawTableReference { node_id, .. } => Some(node_id.as_str()),
        PaintOp::Unknown => None,
    }
}

fn op_kind(op: &PaintOp) -> u8 {
    match op {
        PaintOp::DrawText { .. } => 0,
        PaintOp::DrawBox { .. } => 1,
        PaintOp::DrawImage { .. } => 2,
        _ => 3,
    }
}

fn position_of(running: &[RunningBlockNode], id: &str) -> Option<RunningBlockPosition> {
    running
        .iter()
        .find_map(|rb| k2f_core::find_node(&rb.node, id).map(|_| rb.position))
}
