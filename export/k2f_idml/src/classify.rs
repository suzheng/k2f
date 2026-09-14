use crate::geo::find_geo;
use crate::ir::{DocIR, PageElement, PageIR};
use crate::master;
use crate::text::{list_start_at, textbox_from_draw_ctx, TextFonts};
use crate::IdmlError;
use k2f_core::PaintOp;
use k2f_paint::OpenedDocument;
use std::collections::BTreeSet;

pub fn classify_opened(doc: &OpenedDocument) -> Result<DocIR, IdmlError> {
    let lock = doc.lock().ok_or(IdmlError::Unlocked)?;
    if lock.has_unknown_paint_ops() {
        return Err(IdmlError::UnknownOp);
    }
    let pages = &lock.geometry.pages;
    if pages.is_empty() {
        return Err(IdmlError::Write("lock has no pages".into()));
    }
    let fonts = TextFonts::new(doc.fonts());
    let root = doc.semantic_root();
    let running = doc.running_blocks();
    let running_ids = master::running_ids(running);
    let list_starts = list_start_at(root);
    let total_pages = pages.len();
    let mut ir_pages = Vec::with_capacity(total_pages);
    for (page_idx, page) in pages.iter().enumerate() {
        let ops = lock
            .render_plan
            .pages
            .get(page_idx)
            .map(|p| p.ops.as_slice())
            .unwrap_or(&[]);
        let mut elements = Vec::new();
        for op in ops {
            match op {
                PaintOp::Unknown => return Err(IdmlError::UnknownOp),
                PaintOp::DrawText {
                    node_id,
                    rect,
                    runs,
                } => {
                    if running_ids.contains(node_id) {
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
                        &fonts,
                        list_starts.get(&node.id).copied().unwrap_or(1),
                        None,
                    ) {
                        elements.push(PageElement::TextBox(tb));
                    }
                }
                PaintOp::DrawBox { .. }
                | PaintOp::BackdropBlur { .. }
                | PaintOp::DrawImage { .. }
                | PaintOp::DrawTableReference { .. } => {}
            }
        }
        ir_pages.push(PageIR { elements });
    }
    let master_els = master::collect_master(
        doc,
        lock,
        &fonts,
        &running_ids,
        &list_starts,
        total_pages,
    );
    let mut used = BTreeSet::new();
    used.insert(fonts.default_family().to_string());
    for page in &ir_pages {
        collect_fonts(&mut used, &page.elements);
    }
    collect_fonts(&mut used, &master_els);
    let page0 = &pages[0];
    Ok(DocIR {
        title: doc.title().to_string(),
        page_w_pt: crate::coord::pt_val(page0.width),
        page_h_pt: crate::coord::pt_val(page0.height),
        pages: ir_pages,
        master: master_els,
        fonts: used.into_iter().collect(),
    })
}

fn collect_fonts(out: &mut BTreeSet<String>, els: &[PageElement]) {
    for el in els {
        if let Some(tb) = el.textbox() {
            for run in &tb.runs {
                if !run.font_name.is_empty() {
                    out.insert(run.font_name.clone());
                }
            }
        }
    }
}
