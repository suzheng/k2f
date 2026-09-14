use crate::geo::find_geo;
use crate::ir::PageElement;
use crate::text::{textbox_from_draw_ctx, TextFonts};
use k2f_core::{for_each_node, LockFile, PaintOp, RunningBlockNode};
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

pub(crate) fn collect_master(
    doc: &OpenedDocument,
    lock: &LockFile,
    fonts: &TextFonts,
    ids: &HashSet<String>,
    list_starts: &BTreeMap<String, u32>,
    total_pages: usize,
) -> Vec<PageElement> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let root = doc.semantic_root();
    let running = doc.running_blocks();
    let Some(page) = lock.geometry.pages.first() else {
        return out;
    };
    let Some(plan) = lock.render_plan.pages.iter().find(|p| p.index == page.index) else {
        return out;
    };
    for op in &plan.ops {
        let PaintOp::DrawText {
            node_id,
            rect,
            runs,
        } = op
        else {
            continue;
        };
        if !ids.contains(node_id) || !seen.insert(node_id.clone()) {
            continue;
        }
        let Some(node) = k2f_core::find_in_trees(root, running, node_id) else {
            continue;
        };
        let geo = find_geo(&page.root, node_id);
        if let Some(tb) = textbox_from_draw_ctx(
            node,
            rect,
            runs,
            geo,
            fonts,
            list_starts.get(&node.id).copied().unwrap_or(1),
            Some(total_pages),
        ) {
            out.push(PageElement::TextBox(tb));
        }
    }
    out
}
