use crate::coord::{pt_to_emu, pt_to_twips};
use crate::effect::{
    box_is_effect, consume_following_box, effect_box_crop, glass_crop, is_math_node, is_rule_id,
    keep_for_effect, math_crop, rasterize_slice,
};
use crate::geo::{find_geo, page_bg_hex};
use crate::ir::{DocIR, LineDash, PageElement, PageIR, PictureBox, ShapeBox, TextRun};
use crate::picture::picture_from_draw;
use crate::shape::shapes_from_box;
use crate::table::table_ref_placeholder;
use crate::text::{font_ctx, list_start_at, textbox_from_draw_ctx};
use crate::xml::word_hex_color;
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
    let assets = doc.assets();
    let mut media_n = 1u32;
    let mut raster_n = 1u32;
    let list_starts = list_start_at(root);
    let total_pages = pages.len();
    let mut ir_pages = Vec::with_capacity(pages.len());
    for (page_idx, page) in pages.iter().enumerate() {
        let ops = lock
            .render_plan
            .pages
            .get(page_idx)
            .map(|p| p.ops.as_slice())
            .unwrap_or(&[]);
        let mut elements = Vec::new();
        let mut skip_ops = HashSet::new();
        let bg_hex = word_hex_color(&page_bg_hex(page, ops));
        for (i, op) in ops.iter().enumerate() {
            if skip_ops.contains(&i) {
                continue;
            }
            let rel = rel_height(i);
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
                        if let Some(tb) = textbox_from_draw_ctx(
                            node,
                            rect,
                            runs,
                            geo,
                            &fonts,
                            rel,
                            page_idx,
                            total_pages,
                            list_starts.get(&node.id).copied().unwrap_or(1),
                        ) {
                            // Bullets/numbers are literal runs (num_id stays 0).
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
                    // Writer paints pic:pic above every wps:wsp. Images that
                    // later lock paint sits on must use the raster shape path.
                    if crate::geo::image_overlapped_by_later_content(rect, &ops[i + 1..]) {
                        elements.push(PageElement::Raster(pic));
                    } else {
                        elements.push(PageElement::Picture(pic));
                    }
                }
                PaintOp::DrawTableReference { node_id, rect, .. } => {
                    elements.push(PageElement::Shape(table_ref_placeholder(
                        node_id, rect, rel,
                    )));
                }
            }
        }
        // Locks without a full-page paper DrawBox still need a behindDoc wash:
        // Word Dark Mode remaps/hides w:background alone (legal white pages).
        ensure_page_paper_wash(
            &mut elements,
            page.width,
            page.height,
            page_idx,
            &bg_hex,
        );
        assign_textbox_underlays(&mut elements, &bg_hex);
        absorb_self_fill_shapes(&mut elements);
        fold_shell_label_stacks(
            &mut elements,
            pt_to_emu(page.width),
            pt_to_emu(page.height),
        );
        raise_hairline_borders_above_covers(&mut elements);
        demote_outline_frames_below_text(&mut elements);
        unpin_covering_empty_txboxes(&mut elements);
        ir_pages.push(PageIR { bg_hex, elements });
    }
    let page0 = &pages[0];
    let paper = ir_pages
        .first()
        .map(|p| p.bg_hex.clone())
        .unwrap_or_else(|| "FFFFFE".into());
    // Page tokens paint in the body with lock-resolved numbers (same as PPTX).
    // Writer skips header1.xml / footer1.xml when pgMar is 0.
    let (mut header, mut footer) = (Vec::new(), Vec::new());
    assign_textbox_underlays(&mut header, &paper);
    assign_textbox_underlays(&mut footer, &paper);
    absorb_self_fill_shapes(&mut header);
    absorb_self_fill_shapes(&mut footer);
    raise_hairline_borders_above_covers(&mut header);
    raise_hairline_borders_above_covers(&mut footer);
    demote_outline_frames_below_text(&mut header);
    demote_outline_frames_below_text(&mut footer);
    unpin_covering_empty_txboxes(&mut header);
    unpin_covering_empty_txboxes(&mut footer);
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

/// If the lock never painted a full-page paper DrawBox, inject one.
/// `w:background` alone is not enough: Word Dark Mode remaps/hides that slot.
fn ensure_page_paper_wash(
    elements: &mut Vec<PageElement>,
    page_w: k2f_core::Pt,
    page_h: k2f_core::Pt,
    page_idx: usize,
    bg_hex: &str,
) {
    let cx = pt_to_emu(page_w);
    let cy = pt_to_emu(page_h);
    let has_wash = elements.iter().any(|el| {
        let PageElement::Shape(s) = el else {
            return false;
        };
        if !s.behind_doc || s.x_emu != 0 || s.y_emu != 0 || s.cx_emu != cx || s.cy_emu != cy {
            return false;
        }
        s.gradient.is_some() || (s.fill_hex.is_some() && s.fill_alpha >= 255)
    });
    if has_wash {
        return;
    }
    elements.insert(
        0,
        PageElement::Shape(ShapeBox {
            node_id: format!("::page_{page_idx}::background"),
            x_emu: 0,
            y_emu: 0,
            cx_emu: cx,
            cy_emu: cy,
            fill_hex: Some(bg_hex.to_string()),
            fill_alpha: 255,
            gradient: None,
            corner_emu: 0,
            line_hex: None,
            line_alpha: 255,
            line_w_emu: 0,
            line_dash: LineDash::Solid,
            behind_doc: true,
            relative_height: 0,
            pin_empty_txbox: false,
        }),
    );
}

/// Word Dark Mode inverts text in `noFill` floating boxes. Paint an opaque
/// underlay matching the shape immediately behind the box (or the page paper).
///
/// Skip the paper fallback when the topmost covering layer is chrome
/// (picture, raster, gradient, glass). A full-page paper shape behind a
/// decorative SVG must not win — that white underlay hides the lock art.
/// Chrome-backed boxes stay `fill_hex = None`; the text-box XML emits a
/// fully-transparent solidFill instead of `noFill`.
fn assign_textbox_underlays(elements: &mut [PageElement], paper_hex: &str) {
    enum Behind {
        Solid(String),
        Chrome,
    }
    struct Layer {
        rel: u32,
        x: i64,
        y: i64,
        w: i64,
        h: i64,
        kind: Behind,
    }
    let layers: Vec<Layer> = elements
        .iter()
        .filter_map(|el| match el {
            PageElement::Shape(s)
                if s.gradient.is_none() && s.fill_alpha >= 255 && s.fill_hex.is_some() =>
            {
                s.fill_hex.as_ref().map(|h| Layer {
                    rel: s.relative_height,
                    x: s.x_emu,
                    y: s.y_emu,
                    w: s.cx_emu,
                    h: s.cy_emu,
                    kind: Behind::Solid(h.clone()),
                })
            }
            PageElement::Picture(p) | PageElement::Raster(p) => Some(Layer {
                rel: p.relative_height,
                x: p.x_emu,
                y: p.y_emu,
                w: p.cx_emu,
                h: p.cy_emu,
                kind: Behind::Chrome,
            }),
            PageElement::Shape(s) if s.gradient.is_some() || s.fill_alpha < 255 => Some(Layer {
                rel: s.relative_height,
                x: s.x_emu,
                y: s.y_emu,
                w: s.cx_emu,
                h: s.cy_emu,
                kind: Behind::Chrome,
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
        let mut best: Option<(u32, &Behind)> = None;
        for layer in &layers {
            if layer.rel >= tb.relative_height {
                continue;
            }
            if px >= layer.x
                && py >= layer.y
                && px < layer.x.saturating_add(layer.w)
                && py < layer.y.saturating_add(layer.h)
            {
                if best.map(|(rel, _)| layer.rel >= rel).unwrap_or(true) {
                    best = Some((layer.rel, &layer.kind));
                }
            }
        }
        tb.fill_hex = match best {
            Some((_, Behind::Solid(hex))) => Some(word_hex_color(hex)),
            Some((_, Behind::Chrome)) => None,
            None => Some(word_hex_color(paper_hex)),
        };
        tb.fill_alpha = 255;
    }
}

/// A DrawBox + DrawText for the same node (pills, chips) must be one Word
/// shape. LibreOffice paints the later roundRect on top of the text box and
/// hides white labels. Fold corner radius, fill, and outline into the text
/// box and drop the duplicate fill shape (and its `::stroke` companion).
fn absorb_self_fill_shapes(elements: &mut Vec<PageElement>) {
    #[derive(Clone)]
    struct Decor {
        corner_emu: i64,
        fill_hex: Option<String>,
        fill_alpha: u8,
        line_hex: Option<String>,
        line_alpha: u8,
        line_w_emu: i64,
        line_dash: crate::ir::LineDash,
    }
    let extras: BTreeMap<String, Decor> = elements
        .iter()
        .filter_map(|el| match el {
            PageElement::Shape(s) => Some((
                s.node_id.clone(),
                Decor {
                    corner_emu: s.corner_emu,
                    fill_hex: s.fill_hex.clone(),
                    fill_alpha: s.fill_alpha,
                    line_hex: s.line_hex.clone(),
                    line_alpha: s.line_alpha,
                    line_w_emu: s.line_w_emu,
                    line_dash: s.line_dash,
                },
            )),
            _ => None,
        })
        .collect();
    let mut absorbed = HashSet::new();
    for el in elements.iter_mut() {
        let PageElement::TextBox(tb) = el else {
            continue;
        };
        let Some(shape) = extras.get(&tb.node_id) else {
            continue;
        };
        // Closed outlines may live on `{id}::stroke` after fill/stroke split.
        let stroke = extras.get(&format!("{}::stroke", tb.node_id));
        tb.corner_emu = shape.corner_emu;
        if shape.fill_hex.is_some() {
            tb.fill_hex = shape.fill_hex.clone();
            tb.fill_alpha = shape.fill_alpha;
            // wrap=none sizes the frame to the glyphs, so a navy table
            // header or padded pill collapses to a leftover strip.
            if covering_solid_alpha(shape.fill_alpha) {
                tb.wrap = true;
            }
        }
        let line_src = stroke.unwrap_or(shape);
        if line_src.line_hex.is_some() {
            tb.line_hex = line_src.line_hex.clone();
            tb.line_alpha = line_src.line_alpha;
            tb.line_w_emu = line_src.line_w_emu;
            tb.line_dash = line_src.line_dash;
        }
        absorbed.insert(tb.node_id.clone());
        if stroke.is_some() {
            absorbed.insert(format!("{}::stroke", tb.node_id));
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

/// Parent DrawBox + nested DrawText is the same object in the lock (title
/// banners, table-header cells, module cards). Cards must not go behindDoc
/// (page paper is the only behindDoc fill; hosts z-order that stack by size).
/// In-front fills would cover later labels, so fold the shell into one
/// wrap=square text box.
///
/// Do **not** fold when the shell contains a picture/raster: merge keeps the
/// caption's higher `relativeHeight`, which raises the opaque card fill above
/// the image (blank FIG cards in LibreOffice Writer). Leave card + media +
/// caption as siblings so lock z-order stays intact. Skip side-by-side labels,
/// glass, gradients, and shells that still hold child cards.
fn fold_shell_label_stacks(elements: &mut Vec<PageElement>, page_w_emu: i64, page_h_emu: i64) {
    const TOL: i64 = 12_700;
    let mut i = 0;
    while i < elements.len() {
        let PageElement::Shape(shell) = &elements[i] else {
            i += 1;
            continue;
        };
        if shell.gradient.is_some()
            || shell.fill_hex.is_none()
            || !covering_solid_alpha(shell.fill_alpha)
            || crate::geo::is_thin_fill_emu(shell.cx_emu, shell.cy_emu)
            || (shell.cx_emu + TOL >= page_w_emu && shell.cy_emu + TOL >= page_h_emu)
        {
            i += 1;
            continue;
        }
        let shell_id = shell.node_id.clone();
        let outer = (shell.x_emu, shell.y_emu, shell.cx_emu, shell.cy_emu);
        let mut labels = Vec::new();
        let mut blocked = false;
        for (j, el) in elements.iter().enumerate().skip(i + 1) {
            let Some(bj) = element_aabb(el) else {
                continue;
            };
            if !aabbs_intersect(outer, bj) {
                continue;
            }
            match el {
                PageElement::TextBox(_) if aabb_contains(outer, bj, TOL) => labels.push(j),
                PageElement::Shape(s) if is_shell_companion(&shell_id, &s.node_id) => {}
                // Contained media blocks fold — see fn doc above.
                PageElement::Picture(_) | PageElement::Raster(_)
                    if aabb_contains(outer, bj, TOL) =>
                {
                    blocked = true;
                    break;
                }
                _ => {
                    blocked = true;
                    break;
                }
            }
        }
        if blocked || labels.is_empty() || !labels_are_vertical_stack(elements, &labels) {
            i += 1;
            continue;
        }
        let Some(folded) = merge_labels_into_shell(&elements[i], elements, &labels) else {
            i += 1;
            continue;
        };
        // Drop nested labels and the fill/stroke-split outline companion.
        // Leaving `{id}::stroke` as a sibling puts a full-AABB noFill frame
        // above the folded box after hairline raise (click shield).
        let mut drop: HashSet<usize> = labels.into_iter().collect();
        let stroke_id = format!("{shell_id}::stroke");
        for (j, el) in elements.iter().enumerate() {
            if matches!(el, PageElement::Shape(s) if s.node_id == stroke_id) {
                drop.insert(j);
            }
        }
        let mut next = Vec::with_capacity(elements.len());
        for (k, el) in elements.iter().enumerate() {
            if k == i {
                next.push(PageElement::TextBox(folded.clone()));
            } else if !drop.contains(&k) {
                next.push(el.clone());
            }
        }
        *elements = next;
        i += 1;
    }
}

fn is_shell_companion(shell_id: &str, other: &str) -> bool {
    other == format!("{shell_id}::stroke")
        || other.starts_with(&format!("{shell_id}::edge_"))
}

fn aabb_contains(
    outer: (i64, i64, i64, i64),
    inner: (i64, i64, i64, i64),
    tol: i64,
) -> bool {
    let (ox, oy, ow, oh) = outer;
    let (ix, iy, iw, ih) = inner;
    ix + tol >= ox
        && iy + tol >= oy
        && ix + iw <= ox + ow + tol
        && iy + ih <= oy + oh + tol
}

fn labels_are_vertical_stack(elements: &[PageElement], idxs: &[usize]) -> bool {
    if idxs.len() <= 1 {
        return true;
    }
    let mut boxes: Vec<(i64, i64, i64, i64)> = idxs
        .iter()
        .filter_map(|&j| element_aabb(&elements[j]))
        .collect();
    boxes.sort_by_key(|b| b.1);
    for pair in boxes.windows(2) {
        let (ax, ay, aw, ah) = pair[0];
        let (bx, by, bw, _bh) = pair[1];
        let x_overlap = ax.max(bx) < (ax + aw).min(bx + bw);
        if !x_overlap && by < ay + ah {
            return false;
        }
    }
    true
}

fn merge_labels_into_shell(
    shell_el: &PageElement,
    elements: &[PageElement],
    label_idxs: &[usize],
) -> Option<crate::ir::TextBox> {
    let PageElement::Shape(shell) = shell_el else {
        return None;
    };
    let mut tbs: Vec<&crate::ir::TextBox> = label_idxs
        .iter()
        .filter_map(|&j| match &elements[j] {
            PageElement::TextBox(t) => Some(t),
            _ => None,
        })
        .collect();
    tbs.sort_by_key(|t| t.y_emu);
    let first = tbs.first()?;
    let mut folded = (*first).clone();
    folded.node_id = shell.node_id.clone();
    folded.x_emu = shell.x_emu;
    folded.y_emu = shell.y_emu;
    folded.cx_emu = shell.cx_emu;
    folded.cy_emu = shell.cy_emu;
    // Use the tightest insets that still fit every label. Sizing only to the
    // first (often a narrow kicker) left a wrap column too narrow for later
    // display figures, so host-bold "SOUNDSTAGE" / "23:00 CST" mid-wrapped.
    folded.l_ins_emu = tbs
        .iter()
        .map(|tb| (tb.x_emu - shell.x_emu + tb.l_ins_emu).max(0))
        .min()
        .unwrap_or(0);
    folded.t_ins_emu = (first.y_emu - shell.y_emu + first.t_ins_emu).max(0);
    // Widen to the shell: a kicker-narrow column mid-wraps host-bold tracked
    // labels. Keep the lock left pad and mirror it on the right so the wrap
    // column is the card interior (still inset, not edge-flush).
    let widest_r = tbs
        .iter()
        .map(|tb| {
            (shell.x_emu + shell.cx_emu - tb.x_emu - tb.cx_emu + tb.r_ins_emu).max(0)
        })
        .min()
        .unwrap_or(0);
    folded.r_ins_emu = folded.l_ins_emu.min(widest_r);
    folded.b_ins_emu = tbs
        .last()
        .map(|last| {
            (shell.y_emu + shell.cy_emu - last.y_emu - last.cy_emu + last.b_ins_emu).max(0)
        })
        .unwrap_or(0);
    folded.fill_hex = shell.fill_hex.clone();
    folded.fill_alpha = shell.fill_alpha;
    folded.corner_emu = shell.corner_emu;
    // Line may live on the shell or on `{id}::stroke` after fill/stroke split.
    let line_src = elements
        .iter()
        .find_map(|el| match el {
            PageElement::Shape(s) if s.node_id == format!("{}::stroke", shell.node_id) => Some(s),
            _ => None,
        })
        .filter(|s| s.line_hex.is_some())
        .unwrap_or(shell);
    if line_src.line_hex.is_some() {
        folded.line_hex = line_src.line_hex.clone();
        folded.line_alpha = line_src.line_alpha;
        folded.line_w_emu = line_src.line_w_emu;
        folded.line_dash = line_src.line_dash;
    }
    // Square wrap pins the filled shell extent (wrap=none let Writer clip
    // glyphs inside roundRect pills). One-liner mid-wrap is avoided by the
    // mirrored side pad above.
    folded.wrap = true;
    // Host auto line (~single on the run) fills a tight pill so 7.5pt badges
    // look oversized. Pin single-label shells to face pitch and center them.
    // Multi-label cards: per-paragraph atLeast pitch from each source label
    // (one shared exact pitch crushed mixed faces; host-auto left body leading
    // crushed). Align follows the widest label so a leading centered pill does
    // not center the whole stack.
    if tbs.len() == 1 {
        let pitch = label_line_twips(first);
        folded.line_twips = Some(pitch);
        folded.last_line_twips = Some(pitch);
        folded.para_line_twips = Vec::new();
        folded.para_after_twips = Vec::new();
        folded.vert_center = true;
        folded.t_ins_emu = 0;
        folded.b_ins_emu = 0;
    } else {
        folded.line_twips = None;
        folded.last_line_twips = None;
        folded.align = fold_stack_align(&tbs);
        let (para_line, para_after) = fold_stack_spacing(&tbs);
        folded.para_line_twips = para_line;
        folded.para_after_twips = para_after;
        folded.vert_center = false;
        // Lock leftover under the last label was copied to bodyPr bIns. With
        // restored line pitch (host metrics ≥ rustybuzz), that inset clips
        // the last glyphs while empty padding still shows below the cut.
        // Shell cy already includes the slack — do not reserve it twice.
        folded.b_ins_emu = 0;
        // Widen the wrap column a little so host-bold one-liners (PASSBAND)
        // do not soft-wrap onto a clipped second row inside a tight card.
        let trim = (folded.l_ins_emu / 10).min(folded.r_ins_emu / 10);
        folded.l_ins_emu = folded.l_ins_emu.saturating_sub(trim);
        folded.r_ins_emu = folded.r_ins_emu.saturating_sub(trim);
    }
    folded.runs = Vec::new();
    for (n, tb) in tbs.iter().enumerate() {
        if n > 0 {
            if let Some(template) = tb.runs.first().or(folded.runs.last()) {
                folded.runs.push(newline_run(template));
            }
        }
        // Single-paragraph lock labels must stay one Office line. Host-wider
        // metrics soft-wrap PASSBAND onto a clipped second row inside tight
        // cards; NBSP keeps the lock line intact without changing multi-line
        // bodies (those already have hard `\n` from pin breaks).
        if label_para_count(tb) == 1 {
            folded.runs.extend(nobreak_spaces(tb.runs.iter().cloned()));
        } else {
            folded.runs.extend(tb.runs.iter().cloned());
        }
    }
    Some(folded)
}

/// Widest content column wins — body copy over a leading chip/pill/kicker.
fn fold_stack_align(tbs: &[&crate::ir::TextBox]) -> crate::ir::TextAlign {
    tbs.iter()
        .max_by_key(|tb| {
            let inner = tb
                .cx_emu
                .saturating_sub(tb.l_ins_emu.saturating_add(tb.r_ins_emu));
            (inner, tb.cy_emu)
        })
        .map(|tb| tb.align)
        .unwrap_or(crate::ir::TextAlign::Left)
}

/// Per-paragraph line pitch + inter-label `after` from lock y gaps.
fn fold_stack_spacing(tbs: &[&crate::ir::TextBox]) -> (Vec<Option<i64>>, Vec<i64>) {
    let mut para_line = Vec::new();
    let mut para_after = Vec::new();
    for (i, tb) in tbs.iter().enumerate() {
        let n = label_para_count(tb);
        let pitch = label_stack_pitch(tb);
        let more = i + 1 < tbs.len();
        for p in 0..n {
            let is_last = p + 1 == n;
            // Full pitch on every para (including the stack's last). Face-only
            // last pitch + bodyPr bIns was clipping PASSBAND / last desc lines
            // inside cards that still had empty shell below.
            para_line.push(Some(pitch));
            let after = if is_last && more {
                let next = tbs[i + 1];
                let gap_emu = next
                    .y_emu
                    .saturating_sub(tb.y_emu.saturating_add(tb.cy_emu));
                crate::coord::emu_to_twips(gap_emu).max(0)
            } else {
                0
            };
            para_after.push(after);
        }
    }
    (para_line, para_after)
}

fn label_para_count(tb: &crate::ir::TextBox) -> usize {
    let breaks = tb
        .runs
        .iter()
        .map(|r| r.text.chars().filter(|c| *c == '\n').count())
        .sum::<usize>();
    breaks.saturating_add(1).max(1)
}

/// Inter-line pitch for a folded label. Prefer lock `line_twips`. For a
/// true one-line geometry box (single paragraph, short cy), prefer the box
/// height (`line_height_mult`) over face-size pins so the gap to the next
/// stacked label stays correct.
///
/// Never treat a 2-line cy (~2.2–2.5× face) as one line — that doubled pitch
/// per pinned para and clipped PASSBAND/spatialization inside movement cards.
fn label_stack_pitch(tb: &crate::ir::TextBox) -> i64 {
    let from_box = crate::coord::emu_to_twips(tb.cy_emu).max(20);
    let face = label_line_twips(tb);
    let one_line_box =
        label_para_count(tb) == 1 && from_box <= face.saturating_mul(2);
    if let Some(v) = tb.line_twips {
        if one_line_box {
            return from_box.max(v);
        }
        return v.max(20);
    }
    if one_line_box {
        return from_box.max(face);
    }
    face
}

/// Face size as twips (half-pt×10). Used when pinning a single-label pill.
fn label_line_twips(tb: &crate::ir::TextBox) -> i64 {
    if let Some(v) = tb.last_line_twips.or(tb.line_twips) {
        return v.max(20);
    }
    let half = tb
        .runs
        .iter()
        .filter(|r| r.text != "\n" && !r.text.is_empty())
        .map(|r| i64::from(r.sz_half_points))
        .max()
        .unwrap_or(24);
    (half * 10).max(20)
}

fn newline_run(template: &TextRun) -> TextRun {
    TextRun {
        text: "\n".into(),
        hyperlink: None,
        field: None,
        ..template.clone()
    }
}

fn nobreak_spaces<I>(runs: I) -> Vec<TextRun>
where
    I: IntoIterator<Item = TextRun>,
{
    runs.into_iter()
        .map(|mut r| {
            if r.text.contains(' ') {
                r.text = r.text.replace(' ', "\u{00A0}");
            }
            r
        })
        .collect()
}

/// Opaque text-box underlays (Word Dark Mode) use the full cell AABB and sit
/// above earlier paint. Thin `::edge_*` bars live on that AABB edge, so the
/// underlay erases them. Bump those bars above every overlapping opaque text
/// fill — glyphs stay in the padded interior; only the padding strip
/// re-exposes the rule.
///
/// Do **not** raise full-AABB `{id}::stroke` companions or other closed
/// outlines: that puts a page/card-sized noFill frame above every nested
/// label and steals clicks. Those stay under text via
/// [`demote_outline_frames_below_text`].
fn raise_hairline_borders_above_covers(elements: &mut [PageElement]) {
    let covers: Vec<(u32, (i64, i64, i64, i64))> = elements
        .iter()
        .filter_map(|el| match el {
            PageElement::TextBox(tb)
                if tb.fill_hex.is_some() && covering_solid_alpha(tb.fill_alpha) =>
            {
                Some((
                    tb.relative_height,
                    (tb.x_emu, tb.y_emu, tb.cx_emu, tb.cy_emu),
                ))
            }
            _ => None,
        })
        .collect();
    if covers.is_empty() {
        return;
    }
    for el in elements.iter_mut() {
        let PageElement::Shape(s) = el else {
            continue;
        };
        if !is_thin_edge_bar(s) {
            continue;
        }
        let aabb = (s.x_emu, s.y_emu, s.cx_emu, s.cy_emu);
        let mut max_rel = s.relative_height;
        for &(rel, cover) in &covers {
            if aabbs_intersect(aabb, cover) && rel >= max_rel {
                max_rel = rel.saturating_add(1);
            }
        }
        s.relative_height = max_rel;
    }
}

fn is_thin_edge_bar(s: &crate::ir::ShapeBox) -> bool {
    if s.node_id.contains("::edge_") {
        return true;
    }
    // Stroke-only thin rules (not closed card/page outlines).
    s.line_hex.is_some()
        && s.fill_hex.is_none()
        && s.gradient.is_none()
        && crate::geo::is_thin_fill_emu(s.cx_emu, s.cy_emu)
}

/// Full-AABB outline frames (`{id}::stroke` after fill/stroke split, or
/// stroke-only decorative shells) must sit under overlapping labels so hosts
/// hit-test glyphs instead of the transparent outline. Thin `::edge_*` bars
/// are handled by [`raise_hairline_borders_above_covers`] and stay put.
fn demote_outline_frames_below_text(elements: &mut [PageElement]) {
    let texts: Vec<(u32, (i64, i64, i64, i64))> = elements
        .iter()
        .filter_map(|el| match el {
            PageElement::TextBox(tb) => Some((
                tb.relative_height,
                (tb.x_emu, tb.y_emu, tb.cx_emu, tb.cy_emu),
            )),
            _ => None,
        })
        .collect();
    if texts.is_empty() {
        return;
    }
    for el in elements.iter_mut() {
        let PageElement::Shape(s) = el else {
            continue;
        };
        if !is_covering_outline_frame(s) {
            continue;
        }
        let aabb = (s.x_emu, s.y_emu, s.cx_emu, s.cy_emu);
        let mut min_text = None;
        for &(rel, tb) in &texts {
            if aabbs_intersect(aabb, tb) {
                min_text = Some(min_text.map_or(rel, |m: u32| m.min(rel)));
            }
        }
        let Some(min_rel) = min_text else {
            continue;
        };
        if s.relative_height >= min_rel {
            s.relative_height = min_rel.saturating_sub(1);
        }
    }
}

fn is_covering_outline_frame(s: &crate::ir::ShapeBox) -> bool {
    if s.node_id.contains("::edge_") || crate::geo::is_thin_fill_emu(s.cx_emu, s.cy_emu) {
        return false;
    }
    if s.node_id.ends_with("::stroke") {
        return true;
    }
    // Stroke-only decorative frame (no fill) covering a card/page AABB.
    s.line_hex.is_some() && s.fill_hex.is_none() && s.gradient.is_none()
}

/// Writer paints an empty `txBox` over later labels. Keep the pin only when
/// nothing later intersects (Word size-to-fit). Do not send the shell
/// behindDoc: page paper is the only behindDoc fill. Folded labels already
/// share the shell's stacking context; leftover overlapped shells stay in
/// front without a text frame.
fn unpin_covering_empty_txboxes(elements: &mut [PageElement]) {
    let bounds: Vec<Option<(i64, i64, i64, i64)>> = elements.iter().map(element_aabb).collect();
    for i in 0..elements.len() {
        let Some(bi) = bounds[i] else {
            continue;
        };
        let overlapped =
            (i + 1..elements.len()).any(|j| bounds[j].is_some_and(|bj| aabbs_intersect(bi, bj)));
        if !overlapped {
            continue;
        }
        match &mut elements[i] {
            PageElement::Raster(p) | PageElement::Picture(p) => p.pin_empty_txbox = false,
            PageElement::Shape(s) => s.pin_empty_txbox = false,
            _ => {}
        }
    }
}

/// Writer composites in-front fills over later text. Alpha ≥ 128 hides half
/// or more of the glyphs (nearly-opaque `#RRGGBBAA` cards). Lighter frost
/// stays in front so a 14% glass plaque is not sent under page paper.
fn covering_solid_alpha(fill_alpha: u8) -> bool {
    fill_alpha >= 128
}

fn element_aabb(el: &PageElement) -> Option<(i64, i64, i64, i64)> {
    match el {
        PageElement::TextBox(t) => Some((t.x_emu, t.y_emu, t.cx_emu, t.cy_emu)),
        PageElement::Shape(s) => Some((s.x_emu, s.y_emu, s.cx_emu, s.cy_emu)),
        PageElement::Picture(p) | PageElement::Raster(p) => {
            Some((p.x_emu, p.y_emu, p.cx_emu, p.cy_emu))
        }
        PageElement::Table(t) => Some((t.x_emu, t.y_emu, t.cx_emu, t.cy_emu)),
    }
}

fn aabbs_intersect(a: (i64, i64, i64, i64), b: (i64, i64, i64, i64)) -> bool {
    let (ax, ay, aw, ah) = a;
    let (bx, by, bw, bh) = b;
    ax < bx.saturating_add(bw)
        && bx < ax.saturating_add(aw)
        && ay < by.saturating_add(bh)
        && by < ay.saturating_add(ah)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{LineDash, PictureBox, ShapeBox, TextAlign, TextBox, TextRun};
    use k2f_core::Pt;

    #[test]
    fn injects_paper_wash_when_lock_omits_full_page_box() {
        let w = Pt(595_000);
        let h = Pt(842_000);
        let mut elements = vec![PageElement::TextBox(title_box(10))];
        ensure_page_paper_wash(&mut elements, w, h, 0, "FFFFFE");
        assert_eq!(elements.len(), 2);
        let PageElement::Shape(wash) = &elements[0] else {
            panic!("expected injected paper shape");
        };
        assert!(wash.behind_doc);
        assert_eq!(wash.fill_hex.as_deref(), Some("FFFFFE"));
        assert_eq!(wash.node_id, "::page_0::background");
        assert!(!wash.pin_empty_txbox);
        // Second call must not duplicate.
        ensure_page_paper_wash(&mut elements, w, h, 0, "FFFFFE");
        assert_eq!(elements.len(), 2);
    }

    fn paper_shape(rel: u32) -> ShapeBox {
        ShapeBox {
            node_id: "page".into(),
            x_emu: 0,
            y_emu: 0,
            cx_emu: 12_000_000,
            cy_emu: 7_000_000,
            fill_hex: Some("F8FAFC".into()),
            fill_alpha: 255,
            gradient: None,
            corner_emu: 0,
            line_hex: None,
            line_alpha: 255,
            line_w_emu: 0,
            line_dash: LineDash::Solid,
            behind_doc: true,
            relative_height: rel,
            pin_empty_txbox: true,
        }
    }

    fn glow(rel: u32) -> PictureBox {
        PictureBox {
            node_id: "glow".into(),
            x_emu: 2_000_000,
            y_emu: 500_000,
            cx_emu: 8_000_000,
            cy_emu: 6_000_000,
            media_name: "image1.svg".into(),
            bytes: vec![],
            relative_height: rel,
            pin_empty_txbox: false,
        }
    }

    fn title_box(rel: u32) -> TextBox {
        TextBox {
            node_id: "title".into(),
            x_emu: 3_500_000,
            y_emu: 2_800_000,
            cx_emu: 5_000_000,
            cy_emu: 700_000,
            runs: vec![],
            align: TextAlign::Left,
            bullet: false,
            numbered: false,
            num_id: 0,
            ilvl: 0,
            l_ins_emu: 0,
            hang_emu: 0,
            t_ins_emu: 0,
            r_ins_emu: 0,
            b_ins_emu: 0,
            line_twips: None,
            last_line_twips: None,
            para_line_twips: Vec::new(),
            para_after_twips: Vec::new(),
            vert_center: false,
            preserve_whitespace: false,
            relative_height: rel,
            fill_hex: None,
            fill_alpha: 255,
            wrap: false,
            corner_emu: 0,
            line_hex: None,
            line_alpha: 255,
            line_w_emu: 0,
            line_dash: crate::ir::LineDash::Solid,
        }
    }

    #[test]
    fn underlay_skips_paper_when_picture_is_topmost_behind_text() {
        let mut elements = vec![
            PageElement::Shape(paper_shape(0)),
            PageElement::Picture(glow(20)),
            PageElement::TextBox(title_box(40)),
        ];
        assign_textbox_underlays(&mut elements, "F8FAFC");
        let tb = elements[2].textbox().expect("title");
        assert!(
            tb.fill_hex.is_none(),
            "glow must beat page paper, got {:?}",
            tb.fill_hex
        );
    }

    #[test]
    fn hairline_edge_bar_rises_above_opaque_textbox_underlay() {
        let mut edge = paper_shape(10);
        edge.node_id = "cell::edge_bottom".into();
        edge.behind_doc = false;
        edge.fill_hex = Some("1E3A8A".into());
        edge.x_emu = 1_000_000;
        edge.y_emu = 3_490_000;
        edge.cx_emu = 5_000_000;
        edge.cy_emu = 12_700; // ~1pt hairline
        let mut tb = title_box(40);
        tb.fill_hex = Some("FFFFFE".into());
        tb.x_emu = 1_000_000;
        tb.y_emu = 2_800_000;
        tb.cx_emu = 5_000_000;
        tb.cy_emu = 700_000; // overlaps the bottom rule
        let mut elements = vec![PageElement::Shape(edge), PageElement::TextBox(tb)];
        raise_hairline_borders_above_covers(&mut elements);
        let PageElement::Shape(s) = &elements[0] else {
            panic!("edge bar");
        };
        assert!(
            s.relative_height > 40,
            "edge bar must sit above text underlay, got {}",
            s.relative_height
        );
    }

    #[test]
    fn full_aabb_stroke_is_not_raised_above_text() {
        let mut stroke = paper_shape(11);
        stroke.node_id = "card::stroke".into();
        stroke.behind_doc = false;
        stroke.fill_hex = None;
        stroke.line_hex = Some("1E3A8A".into());
        stroke.line_w_emu = 6_350;
        stroke.x_emu = 500_000;
        stroke.y_emu = 500_000;
        stroke.cx_emu = 6_000_000;
        stroke.cy_emu = 4_000_000;
        let mut tb = title_box(40);
        tb.fill_hex = Some("FFFFFE".into());
        tb.x_emu = 800_000;
        tb.y_emu = 800_000;
        tb.cx_emu = 4_000_000;
        tb.cy_emu = 400_000;
        let mut elements = vec![PageElement::Shape(stroke), PageElement::TextBox(tb)];
        raise_hairline_borders_above_covers(&mut elements);
        demote_outline_frames_below_text(&mut elements);
        let PageElement::Shape(s) = &elements[0] else {
            panic!("stroke");
        };
        assert!(
            s.relative_height < 40,
            "closed outline must stay under nested text, got {}",
            s.relative_height
        );
    }

    #[test]
    fn fold_absorbs_stroke_companion_into_shell() {
        let mut shell = paper_shape(10);
        shell.node_id = "party.box".into();
        shell.behind_doc = false;
        shell.x_emu = 1_000_000;
        shell.y_emu = 1_000_000;
        shell.cx_emu = 4_000_000;
        shell.cy_emu = 2_000_000;
        shell.fill_hex = Some("F3F4F6".into());
        let mut stroke = shell.clone();
        stroke.node_id = "party.box::stroke".into();
        stroke.fill_hex = None;
        stroke.line_hex = Some("111827".into());
        stroke.line_w_emu = 12_700;
        stroke.relative_height = 11;
        let mut label = title_box(20);
        label.node_id = "party.box.title".into();
        label.x_emu = 1_200_000;
        label.y_emu = 1_200_000;
        label.cx_emu = 3_000_000;
        label.cy_emu = 300_000;
        label.runs = vec![TextRun {
            text: "SERVICE PROVIDER".into(),
            font_name: "Roboto".into(),
            sz_half_points: 24,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "111827".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut elements = vec![
            PageElement::Shape(shell),
            PageElement::Shape(stroke),
            PageElement::TextBox(label),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        assert_eq!(elements.len(), 1, "shell+stroke+label → one text box");
        let got = elements[0].textbox().expect("folded");
        assert_eq!(got.fill_hex.as_deref(), Some("F3F4F6"));
        assert_eq!(got.line_hex.as_deref(), Some("111827"));
        assert_eq!(got.line_w_emu, 12_700);
    }

    #[test]
    fn absorb_folds_split_stroke_companion_into_textbox() {
        let mut fill = paper_shape(10);
        fill.node_id = "pill".into();
        fill.behind_doc = false;
        fill.fill_hex = Some("1E3A8A".into());
        fill.x_emu = 100_000;
        fill.y_emu = 100_000;
        fill.cx_emu = 500_000;
        fill.cy_emu = 200_000;
        fill.corner_emu = 50_000;
        let mut stroke = fill.clone();
        stroke.node_id = "pill::stroke".into();
        stroke.fill_hex = None;
        stroke.line_hex = Some("FFFFFF".into());
        stroke.line_w_emu = 6_350;
        let mut tb = title_box(20);
        tb.node_id = "pill".into();
        tb.x_emu = 100_000;
        tb.y_emu = 100_000;
        tb.cx_emu = 500_000;
        tb.cy_emu = 200_000;
        let mut elements = vec![
            PageElement::Shape(fill),
            PageElement::Shape(stroke),
            PageElement::TextBox(tb),
        ];
        absorb_self_fill_shapes(&mut elements);
        assert_eq!(elements.len(), 1, "fill+stroke+label → one text box");
        let got = elements[0].textbox().expect("pill");
        assert_eq!(got.fill_hex.as_deref(), Some("1E3A8A"));
        assert_eq!(got.line_hex.as_deref(), Some("FFFFFF"));
        assert_eq!(got.line_w_emu, 6_350);
        assert_eq!(got.corner_emu, 50_000);
    }

    #[test]
    fn underlay_keeps_paper_when_no_chrome_covers_text() {
        let mut tb = title_box(40);
        tb.x_emu = 0;
        tb.y_emu = 0;
        tb.cx_emu = 100_000;
        tb.cy_emu = 50_000;
        let mut glow = glow(20);
        glow.x_emu = 5_000_000;
        glow.y_emu = 5_000_000;
        let mut elements = vec![
            PageElement::Shape(paper_shape(0)),
            PageElement::Picture(glow),
            PageElement::TextBox(tb),
        ];
        assign_textbox_underlays(&mut elements, "F8FAFC");
        let got = elements[2].textbox().expect("title").fill_hex.as_deref();
        assert_eq!(got, Some("F8FAFC"));
    }

    #[test]
    fn underlay_translucent_card_stays_unfilled() {
        let mut card = paper_shape(10);
        card.x_emu = 3_000_000;
        card.y_emu = 2_500_000;
        card.cx_emu = 6_000_000;
        card.cy_emu = 1_500_000;
        card.fill_hex = Some("0E1326".into());
        card.fill_alpha = 0xE6;
        let mut elements = vec![
            PageElement::Shape(paper_shape(0)),
            PageElement::Shape(card),
            PageElement::TextBox(title_box(40)),
        ];
        assign_textbox_underlays(&mut elements, "070913");
        let tb = elements[2].textbox().expect("title");
        assert!(
            tb.fill_hex.is_none(),
            "translucent card must not paint an opaque underlay strip, got {:?}",
            tb.fill_hex
        );
    }

    #[test]
    fn absorb_copies_pill_outline_into_textbox() {
        let mut pill = paper_shape(10);
        pill.node_id = "badge".into();
        pill.behind_doc = false;
        pill.cx_emu = 900_000;
        pill.cy_emu = 200_000;
        pill.fill_hex = Some("FEF2F2".into());
        pill.corner_emu = 8_000;
        pill.line_hex = Some("DC2626".into());
        pill.line_w_emu = 12_700;
        let mut tb = title_box(20);
        tb.node_id = "badge".into();
        tb.x_emu = pill.x_emu;
        tb.y_emu = pill.y_emu;
        tb.cx_emu = pill.cx_emu;
        tb.cy_emu = pill.cy_emu;
        let mut elements = vec![PageElement::Shape(pill), PageElement::TextBox(tb)];
        absorb_self_fill_shapes(&mut elements);
        assert_eq!(elements.len(), 1);
        let got = elements[0].textbox().expect("folded");
        assert_eq!(got.fill_hex.as_deref(), Some("FEF2F2"));
        assert_eq!(got.line_hex.as_deref(), Some("DC2626"));
        assert_eq!(got.line_w_emu, 12_700);
        assert_eq!(got.corner_emu, 8_000);
        assert!(got.wrap, "filled pill must pin lock extent, got wrap=false");
    }

    #[test]
    fn fold_parent_banner_keeps_fill_and_stacked_labels() {
        let mut banner = paper_shape(10);
        banner.node_id = "p2.sec.banner".into();
        banner.behind_doc = false;
        banner.x_emu = 1_000_000;
        banner.y_emu = 1_000_000;
        banner.cx_emu = 8_000_000;
        banner.cy_emu = 1_200_000;
        banner.fill_hex = Some("090D16".into());
        banner.corner_emu = 50_800;
        let mut tag = title_box(20);
        tag.node_id = "p2.banner.tag".into();
        tag.x_emu = 1_200_000;
        tag.y_emu = 1_100_000;
        tag.cx_emu = 7_500_000;
        tag.cy_emu = 200_000;
        let mut title = title_box(30);
        title.node_id = "p2.banner.title".into();
        title.x_emu = 1_200_000;
        title.y_emu = 1_400_000;
        title.cx_emu = 7_500_000;
        title.cy_emu = 400_000;
        title.runs = vec![TextRun {
            text: "TITLE".into(),
            font_name: "Roboto".into(),
            sz_half_points: 24,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "FFFFFF".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        tag.runs = vec![TextRun {
            text: "TAG".into(),
            font_name: "Roboto".into(),
            sz_half_points: 16,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "FFFFFF".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut elements = vec![
            PageElement::Shape(banner),
            PageElement::TextBox(tag),
            PageElement::TextBox(title),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        assert_eq!(elements.len(), 1, "banner + labels must be one text box");
        let got = elements[0].textbox().expect("folded");
        assert_eq!(got.fill_hex.as_deref(), Some("090D16"));
        assert_eq!(got.cx_emu, 8_000_000);
        assert!(got.wrap, "filled shell must keep wrap=square");
        let blob: String = got.runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(blob, "TAG\nTITLE");
        assert_eq!(
            got.para_line_twips.len(),
            2,
            "multi-label fold pins per-label pitch, got {:?}",
            got.para_line_twips
        );
        assert!(
            got.para_line_twips.iter().all(|p| p.is_some()),
            "each folded para needs a pitch: {:?}",
            got.para_line_twips
        );
        assert_eq!(got.para_after_twips.len(), 2);
        assert!(
            got.para_after_twips[0] > 0,
            "lock gap between stacked labels must become after, got {:?}",
            got.para_after_twips
        );
        assert_eq!(got.para_after_twips[1], 0);
        assert_eq!(got.l_ins_emu, 180_000, "multi-label fold trims side pad 10% for wrap slack");
        assert_eq!(got.r_ins_emu, 180_000, "mirrored side pad after trim");
        assert_eq!(got.b_ins_emu, 0);
    }

    #[test]
    fn fold_stack_align_follows_widest_label_not_leading_pill() {
        let mut card = paper_shape(10);
        card.node_id = "metrics.card".into();
        card.behind_doc = false;
        card.x_emu = 1_000_000;
        card.y_emu = 1_000_000;
        card.cx_emu = 4_000_000;
        card.cy_emu = 2_000_000;
        card.fill_hex = Some("1A1D29".into());
        let mut badge = title_box(20);
        badge.node_id = "metrics.badge".into();
        badge.align = TextAlign::Center;
        badge.x_emu = 1_200_000;
        badge.y_emu = 1_100_000;
        badge.cx_emu = 1_500_000;
        badge.cy_emu = 200_000;
        badge.runs = vec![TextRun {
            text: "BADGE".into(),
            font_name: "Roboto".into(),
            sz_half_points: 16,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "00F0FF".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut body = title_box(30);
        body.node_id = "metrics.body".into();
        body.align = TextAlign::Left;
        body.line_twips = Some(290);
        body.last_line_twips = Some(200);
        body.x_emu = 1_200_000;
        body.y_emu = 1_400_000;
        body.cx_emu = 3_600_000;
        body.cy_emu = 400_000;
        body.runs = vec![
            TextRun {
                text: "Line one of the metric body".into(),
                font_name: "Roboto".into(),
                sz_half_points: 20,
                bold: false,
                italic: false,
                underline: false,
                strike: false,
                color_hex: "FFFFFF".into(),
                hyperlink: None,
                script: crate::ir::ScriptPos::Baseline,
                tracking_twips: 0,
                field: None,
            },
            TextRun {
                text: "\n".into(),
                font_name: "Roboto".into(),
                sz_half_points: 20,
                bold: false,
                italic: false,
                underline: false,
                strike: false,
                color_hex: "FFFFFF".into(),
                hyperlink: None,
                script: crate::ir::ScriptPos::Baseline,
                tracking_twips: 0,
                field: None,
            },
            TextRun {
                text: "Line two".into(),
                font_name: "Roboto".into(),
                sz_half_points: 20,
                bold: false,
                italic: false,
                underline: false,
                strike: false,
                color_hex: "FFFFFF".into(),
                hyperlink: None,
                script: crate::ir::ScriptPos::Baseline,
                tracking_twips: 0,
                field: None,
            },
        ];
        let mut elements = vec![
            PageElement::Shape(card),
            PageElement::TextBox(badge),
            PageElement::TextBox(body),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 6_858_000);
        assert_eq!(elements.len(), 1);
        let got = elements[0].textbox().expect("folded");
        assert_eq!(got.align, TextAlign::Left, "widest body label must win");
        // badge (single-line) → box-height pitch; body lines share lock pitch.
        assert_eq!(
            got.para_line_twips,
            vec![
                Some(crate::coord::emu_to_twips(200_000)),
                Some(290),
                Some(290)
            ]
        );
        assert_eq!(got.b_ins_emu, 0, "multi-label fold must not reserve bIns");
        assert!(got.para_after_twips[0] > 0);
        assert_eq!(got.para_after_twips[1], 0);
        assert_eq!(got.para_after_twips[2], 0);
    }

    #[test]
    fn fold_stack_pitch_ignores_multiline_box_height() {
        // Soft-wrapped body is one para whose cy spans N lines. Using cy as
        // pitch would ~N× the leading (callout quote blow-up).
        let mut body = title_box(10);
        body.line_twips = Some(189);
        body.last_line_twips = Some(140);
        body.cy_emu = 360_000; // ~3× a one-line box
        body.runs = vec![TextRun {
            text: "Soft wrapped body without hard breaks".into(),
            font_name: "Roboto".into(),
            sz_half_points: 14,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "FFFFFF".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        assert_eq!(super::label_stack_pitch(&body), 189);
        // Two-line meta (pinned `\n`) must use lock line pitch, not 2× box.
        let mut meta = title_box(11);
        meta.line_twips = Some(139);
        meta.last_line_twips = Some(116);
        meta.cy_emu = 176_784; // 13.92pt = two 6.96pt lines
        meta.runs = vec![
            TextRun {
                text: "PASSBAND: impulse".into(),
                font_name: "Roboto".into(),
                sz_half_points: 12,
                bold: false,
                italic: false,
                underline: false,
                strike: false,
                color_hex: "FFFFFF".into(),
                hyperlink: None,
                script: crate::ir::ScriptPos::Baseline,
                tracking_twips: 0,
                field: None,
            },
            TextRun {
                text: "\n".into(),
                font_name: "Roboto".into(),
                sz_half_points: 12,
                bold: false,
                italic: false,
                underline: false,
                strike: false,
                color_hex: "FFFFFF".into(),
                hyperlink: None,
                script: crate::ir::ScriptPos::Baseline,
                tracking_twips: 0,
                field: None,
            },
            TextRun {
                text: "spatialization".into(),
                font_name: "Roboto".into(),
                sz_half_points: 12,
                bold: false,
                italic: false,
                underline: false,
                strike: false,
                color_hex: "FFFFFF".into(),
                hyperlink: None,
                script: crate::ir::ScriptPos::Baseline,
                tracking_twips: 0,
                field: None,
            },
        ];
        assert_eq!(super::label_stack_pitch(&meta), 139);
        let mut title = title_box(20);
        title.line_twips = Some(240);
        title.cy_emu = 205_740; // 16.2pt one-line box
        title.runs = vec![TextRun {
            text: "Title".into(),
            font_name: "Roboto".into(),
            sz_half_points: 24,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "FFFFFF".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        assert_eq!(super::label_stack_pitch(&title), 324);
    }

    #[test]
    fn fold_skipped_when_shell_contains_picture() {
        let mut banner = paper_shape(10);
        banner.node_id = "esg.card".into();
        banner.behind_doc = false;
        banner.x_emu = 1_000_000;
        banner.y_emu = 1_000_000;
        banner.cx_emu = 8_000_000;
        banner.cy_emu = 2_000_000;
        banner.fill_hex = Some("F8FAFC".into());
        let mut gauge = glow(20);
        gauge.node_id = "esg.gauge".into();
        gauge.x_emu = 1_200_000;
        gauge.y_emu = 1_400_000;
        gauge.cx_emu = 1_500_000;
        gauge.cy_emu = 1_200_000;
        let mut title = title_box(30);
        title.node_id = "esg.title".into();
        title.x_emu = 3_000_000;
        title.y_emu = 1_200_000;
        title.cx_emu = 5_500_000;
        title.cy_emu = 400_000;
        title.runs = vec![TextRun {
            text: "ESG".into(),
            font_name: "Roboto".into(),
            sz_half_points: 24,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "0A2540".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut body = title_box(40);
        body.node_id = "esg.body".into();
        body.x_emu = 3_000_000;
        body.y_emu = 1_700_000;
        body.cx_emu = 5_500_000;
        body.cy_emu = 800_000;
        body.runs = vec![TextRun {
            text: "88%".into(),
            font_name: "Roboto".into(),
            sz_half_points: 20,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "0A2540".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut elements = vec![
            PageElement::Shape(banner),
            PageElement::Raster(gauge),
            PageElement::TextBox(title),
            PageElement::TextBox(body),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        assert_eq!(
            elements.len(),
            4,
            "media inside shell blocks fold so card fill cannot cover the image, got {elements:?}"
        );
        assert!(matches!(elements[0], PageElement::Shape(_)));
        assert!(matches!(elements[1], PageElement::Raster(_)));
        assert!(matches!(elements[2], PageElement::TextBox(_)));
        assert!(matches!(elements[3], PageElement::TextBox(_)));
    }

    #[test]
    fn overlapping_raster_and_shell_drop_empty_txbox() {
        let mut shell = paper_shape(0);
        shell.behind_doc = false;
        shell.pin_empty_txbox = true;
        let mut plaque = glow(20);
        plaque.pin_empty_txbox = true;
        let mut aside = glow(30);
        aside.x_emu = 20_000_000;
        aside.y_emu = 20_000_000;
        aside.pin_empty_txbox = true;
        let mut elements = vec![
            PageElement::Shape(shell),
            PageElement::Raster(plaque),
            PageElement::TextBox(title_box(40)),
            PageElement::Raster(aside),
        ];
        unpin_covering_empty_txboxes(&mut elements);
        let PageElement::Shape(shell) = &elements[0] else {
            panic!("shell");
        };
        assert!(
            !shell.pin_empty_txbox,
            "overlapped shell must not pin an empty txBox"
        );
        assert!(
            !shell.behind_doc,
            "cards stay in front; only page paper uses behindDoc"
        );
        let PageElement::Raster(plaque) = &elements[1] else {
            panic!("plaque");
        };
        assert!(
            !plaque.pin_empty_txbox,
            "shadow/glass raster under later text must not pin an empty txBox"
        );
        let PageElement::Raster(aside) = &elements[3] else {
            panic!("aside");
        };
        assert!(
            aside.pin_empty_txbox,
            "non-overlapping raster keeps the Word size pin"
        );
    }

    #[test]
    fn overlapping_contrasting_card_goes_behind_doc() {
        let mut card = paper_shape(10);
        card.behind_doc = false;
        card.x_emu = 3_000_000;
        card.y_emu = 2_500_000;
        card.cx_emu = 6_000_000;
        card.cy_emu = 1_500_000;
        card.fill_hex = Some("EFECE5".into());
        let mut bar = paper_shape(15);
        bar.behind_doc = false;
        bar.x_emu = 3_500_000;
        bar.y_emu = 2_800_000;
        bar.cx_emu = 31_750;
        bar.cy_emu = 700_000;
        bar.fill_hex = Some("9C5B28".into());
        let mut elements = vec![
            PageElement::Shape(card),
            PageElement::Shape(bar),
            PageElement::TextBox(title_box(40)),
        ];
        unpin_covering_empty_txboxes(&mut elements);
        let PageElement::Shape(card) = &elements[0] else {
            panic!("card");
        };
        assert!(
            !card.behind_doc,
            "cards stay in front; fold puts labels in the shell"
        );
        assert!(
            !card.pin_empty_txbox,
            "overlapped plaque must not pin an empty txBox"
        );
        let PageElement::Shape(bar) = &elements[1] else {
            panic!("bar");
        };
        assert!(
            !bar.behind_doc,
            "thin quote/edge bar must stay in front, got behind_doc"
        );
    }

    #[test]
    fn overlapping_card_outline_stays_in_front() {
        let mut card = paper_shape(10);
        card.behind_doc = false;
        card.x_emu = 3_000_000;
        card.y_emu = 2_500_000;
        card.cx_emu = 6_000_000;
        card.cy_emu = 1_500_000;
        card.fill_hex = Some("F2EBE0".into());
        card.line_hex = Some("D8CFBC".into());
        card.line_w_emu = 10_160;
        let mut elements = vec![
            PageElement::Shape(card),
            PageElement::TextBox(title_box(40)),
        ];
        unpin_covering_empty_txboxes(&mut elements);
        assert_eq!(elements.len(), 2, "fill+stroke stay one in-front shape");
        let PageElement::Shape(fill) = &elements[0] else {
            panic!("fill");
        };
        assert!(
            !fill.behind_doc,
            "cards stay in front; only page paper uses behindDoc"
        );
        assert_eq!(fill.line_hex.as_deref(), Some("D8CFBC"));
        assert_eq!(fill.line_w_emu, 10_160);
        assert!(
            !fill.pin_empty_txbox,
            "overlapped outline shell must not pin an empty txBox"
        );
    }

    #[test]
    fn overlapping_translucent_card_goes_behind_doc() {
        let mut card = paper_shape(10);
        card.behind_doc = false;
        card.x_emu = 3_000_000;
        card.y_emu = 2_500_000;
        card.cx_emu = 6_000_000;
        card.cy_emu = 1_500_000;
        card.fill_hex = Some("0E1326".into());
        card.fill_alpha = 0xE6;
        card.line_hex = Some("00F0FF".into());
        card.line_w_emu = 9_525;
        let mut elements = vec![
            PageElement::Shape(card),
            PageElement::TextBox(title_box(40)),
        ];
        unpin_covering_empty_txboxes(&mut elements);
        assert_eq!(elements.len(), 2, "fill+stroke stay one in-front shape");
        let PageElement::Shape(fill) = &elements[0] else {
            panic!("fill");
        };
        assert!(
            !fill.behind_doc,
            "cards stay in front; only page paper uses behindDoc"
        );
        assert_eq!(fill.fill_alpha, 0xE6);
        assert_eq!(fill.line_hex.as_deref(), Some("00F0FF"));
        assert!(
            !fill.pin_empty_txbox,
            "overlapped #RRGGBBAA card must not pin an empty txBox"
        );
    }

    #[test]
    fn overlapping_frost_glass_stays_in_front() {
        let mut frost = paper_shape(10);
        frost.behind_doc = false;
        frost.x_emu = 3_000_000;
        frost.y_emu = 2_500_000;
        frost.cx_emu = 6_000_000;
        frost.cy_emu = 1_500_000;
        frost.fill_hex = Some("FFFFFF".into());
        frost.fill_alpha = 0x24;
        let mut elements = vec![
            PageElement::Shape(frost),
            PageElement::TextBox(title_box(40)),
        ];
        unpin_covering_empty_txboxes(&mut elements);
        let PageElement::Shape(frost) = &elements[0] else {
            panic!("frost");
        };
        assert!(
            !frost.behind_doc,
            "light frost must stay in front, got behind_doc"
        );
        assert!(
            !frost.pin_empty_txbox,
            "overlapped frost still drops the empty txBox pin"
        );
    }
}
