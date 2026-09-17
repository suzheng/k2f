mod scan;

use crate::ir::{DocIR, PageElement, PageIR};
use crate::master;
use crate::table::index_native_tables;
use crate::text::{list_start_at, TextFonts};
use crate::IdmlError;
use k2f_core::LockFile;
use k2f_paint::OpenedDocument;
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub(crate) enum Layer {
    Body,
    Master { #[allow(dead_code)] total_pages: usize },
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
    let tables = index_native_tables(doc.semantic_root(), doc.running_blocks());
    let list_starts = list_start_at(doc.semantic_root());
    let total_pages = pages.len();
    let mut media_n = 1u32;
    let mut raster_n = 1u32;
    let mut ir_pages = Vec::with_capacity(total_pages);
    for (page_idx, page) in pages.iter().enumerate() {
        let ops = lock
            .render_plan
            .pages
            .get(page_idx)
            .map(|p| p.ops.as_slice())
            .unwrap_or(&[]);
        let mut elements = scan::scan_ops(
            ops,
            page,
            page_idx,
            lock,
            Layer::Body,
            doc,
            &fonts,
            &running_ids,
            &list_starts,
            &tables,
            &mut media_n,
            &mut raster_n,
        )?;
        let keep = snapshot_fill_tracking(&elements);
        crate::autosize::apply(&mut elements, page.width.0);
        restore_fill_tracking(&mut elements, &keep);
        ir_pages.push(PageIR { elements });
    }
    let mut master_els = scan_master(
        doc,
        lock,
        &fonts,
        &running_ids,
        &list_starts,
        &tables,
        total_pages,
        &mut media_n,
        &mut raster_n,
    )?;
    let page0 = &pages[0];
    let keep_master = snapshot_fill_tracking(&master_els);
    crate::autosize::apply(&mut master_els, page0.width.0);
    restore_fill_tracking(&mut master_els, &keep_master);
    let mut used = BTreeSet::new();
    used.insert((
        fonts.default_family().to_string(),
        fonts.default_style(),
    ));
    for page in &ir_pages {
        collect_fonts(&mut used, &page.elements);
    }
    collect_fonts(&mut used, &master_els);
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
    tables: &crate::table::TableIndex,
    total_pages: usize,
    media_n: &mut u32,
    raster_n: &mut u32,
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
    scan::scan_ops(
        &plan.ops,
        page,
        0,
        lock,
        Layer::Master { total_pages },
        doc,
        fonts,
        running_ids,
        list_starts,
        tables,
        media_n,
        raster_n,
    )
}

pub(crate) fn keep_node(
    node_id: &str,
    running_ids: &HashSet<String>,
    is_master: bool,
    seen_master: &mut HashSet<String>,
) -> bool {
    let running = running_ids.contains(node_id);
    if is_master {
        // MasterSpread Y does not match body spread Y in InDesign 2026 (headers
        // land mid-page or off the pasteboard). Running items stay on the body
        // spread in paint order, above the page fill.
        let _ = (running, seen_master);
        false
    } else {
        true
    }
}

/// Autosize may drop positive tracking so hug labels do not run through a
/// badge. Letter-spaced titles that already fill the lock box must keep it.
fn snapshot_fill_tracking(elements: &[PageElement]) -> Vec<Vec<i32>> {
    elements
        .iter()
        .map(|el| match el {
            PageElement::TextBox(tb)
                if tb.full_width_lock_line && tb.lock_line_count <= 1 =>
            {
                tb.runs.iter().map(|r| r.tracking).collect()
            }
            _ => Vec::new(),
        })
        .collect()
}

fn restore_fill_tracking(elements: &mut [PageElement], saved: &[Vec<i32>]) {
    for (el, tracks) in elements.iter_mut().zip(saved) {
        if tracks.is_empty() {
            continue;
        }
        let PageElement::TextBox(tb) = el else {
            continue;
        };
        for (run, &t) in tb.runs.iter_mut().zip(tracks) {
            if t != 0 {
                run.tracking = t;
            }
        }
    }
}

fn collect_fonts(out: &mut BTreeSet<(String, String)>, els: &[PageElement]) {
    for el in els {
        match el {
            PageElement::TextBox(tb) => collect_run_fonts(out, &tb.runs),
            PageElement::Table(t) => {
                for row in &t.rows {
                    for cell in &row.cells {
                        collect_run_fonts(out, &cell.runs);
                    }
                }
            }
            _ => {}
        }
    }
}

fn collect_run_fonts(out: &mut BTreeSet<(String, String)>, runs: &[crate::ir::TextRun]) {
    for run in runs {
        if !run.font_name.is_empty() {
            out.insert((run.font_name.clone(), run.idml_font_style()));
        }
    }
}
