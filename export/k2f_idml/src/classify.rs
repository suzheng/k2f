use crate::geo::find_geo;
use crate::ir::{DocIR, PageElement, PageIR};
use crate::master;
use crate::picture::picture_from_draw;
use crate::shape::shapes_from_box;
use crate::text::{list_start_at, textbox_from_draw_ctx, TextFonts};
use crate::IdmlError;
use k2f_core::{LockFile, Page, PaintOp};
use k2f_paint::OpenedDocument;
use std::collections::{BTreeMap, BTreeSet, HashSet};

enum Layer {
    Body,
    Master { total_pages: usize },
}

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
    let running_ids = master::running_ids(doc.running_blocks());
    let list_starts = list_start_at(doc.semantic_root());
    let total_pages = pages.len();
    let mut media_n = 1u32;
    let mut ir_pages = Vec::with_capacity(total_pages);
    for (page_idx, page) in pages.iter().enumerate() {
        let ops = lock
            .render_plan
            .pages
            .get(page_idx)
            .map(|p| p.ops.as_slice())
            .unwrap_or(&[]);
        ir_pages.push(PageIR {
            elements: scan_ops(
                ops,
                page,
                Layer::Body,
                doc,
                &fonts,
                &running_ids,
                &list_starts,
                &mut media_n,
            )?,
        });
    }
    let master_els = scan_master(
        doc,
        lock,
        &fonts,
        &running_ids,
        &list_starts,
        total_pages,
        &mut media_n,
    )?;
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

fn scan_master(
    doc: &OpenedDocument,
    lock: &LockFile,
    fonts: &TextFonts,
    running_ids: &HashSet<String>,
    list_starts: &BTreeMap<String, u32>,
    total_pages: usize,
    media_n: &mut u32,
) -> Result<Vec<PageElement>, IdmlError> {
    let Some(page) = lock.geometry.pages.first() else {
        return Ok(Vec::new());
    };
    let Some(plan) = lock
        .render_plan
        .pages
        .iter()
        .find(|p| p.index == page.index)
    else {
        return Ok(Vec::new());
    };
    scan_ops(
        &plan.ops,
        page,
        Layer::Master { total_pages },
        doc,
        fonts,
        running_ids,
        list_starts,
        media_n,
    )
}

fn scan_ops(
    ops: &[PaintOp],
    page: &Page,
    layer: Layer,
    doc: &OpenedDocument,
    fonts: &TextFonts,
    running_ids: &HashSet<String>,
    list_starts: &BTreeMap<String, u32>,
    media_n: &mut u32,
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
    for op in ops {
        match op {
            PaintOp::Unknown => return Err(IdmlError::UnknownOp),
            PaintOp::DrawText {
                node_id,
                rect,
                runs,
            } => {
                if !keep_node(
                    node_id,
                    running_ids,
                    master_pages.is_some(),
                    &mut seen_master,
                ) {
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
                if !keep_node(
                    node_id,
                    running_ids,
                    master_pages.is_some(),
                    &mut seen_master,
                ) {
                    continue;
                }
                for shape in shapes_from_box(node_id, rect, decoration)? {
                    elements.push(PageElement::Shape(shape));
                }
            }
            PaintOp::DrawImage { node_id, rect, src } => {
                if !keep_node(
                    node_id,
                    running_ids,
                    master_pages.is_some(),
                    &mut seen_master,
                ) {
                    continue;
                }
                let pic = picture_from_draw(node_id, rect, src, assets, *media_n)?;
                *media_n = media_n.saturating_add(1);
                elements.push(PageElement::Picture(pic));
            }
            PaintOp::BackdropBlur { .. } | PaintOp::DrawTableReference { .. } => {}
        }
    }
    Ok(elements)
}

fn keep_node(
    node_id: &str,
    running_ids: &HashSet<String>,
    is_master: bool,
    seen_master: &mut HashSet<String>,
) -> bool {
    let running = running_ids.contains(node_id);
    if is_master {
        running && seen_master.insert(node_id.to_string())
    } else {
        !running
    }
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
