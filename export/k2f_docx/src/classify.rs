use crate::coord::{pt_to_emu, pt_to_twips};
use crate::ooxml::collect_package_fonts;
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
                PaintOp::DrawImage {
                    node_id,
                    rect,
                    src,
                    fit,
                    corner_radius_pt,
                } => {
                    if crate::geo::image_occluded_by_later_opaque_box(rect, &ops[i + 1..]) {
                        continue;
                    }
                    let pic = picture_from_draw(
                        node_id,
                        rect,
                        src,
                        assets,
                        media_n,
                        rel,
                        *fit,
                        *corner_radius_pt,
                    )?;
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
        drop_redundant_underline_on_bottom_rules(&mut elements);
        drop_shells_covered_by_later_media(&mut elements);
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
    drop_redundant_underline_on_bottom_rules(&mut header);
    drop_redundant_underline_on_bottom_rules(&mut footer);
    drop_shells_covered_by_later_media(&mut header);
    drop_shells_covered_by_later_media(&mut footer);
    raise_hairline_borders_above_covers(&mut header);
    raise_hairline_borders_above_covers(&mut footer);
    demote_outline_frames_below_text(&mut header);
    demote_outline_frames_below_text(&mut footer);
    unpin_covering_empty_txboxes(&mut header);
    unpin_covering_empty_txboxes(&mut footer);
    Ok(DocIR {
        title: doc.title().to_string(),
        package_fonts: collect_package_fonts(doc.fonts()),
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

/// Heading roles often paint both a text `underline` modifier and a bottom
/// `::edge_*` bar. Native `w:u` in Writer is thick and far from the glyphs,
/// so keep the lock bar and drop the run flag on wrap=none one-liners.
/// Links and wrapped/folded bodies keep `w:u`.
fn drop_redundant_underline_on_bottom_rules(elements: &mut [PageElement]) {
    let ruled: HashSet<String> = elements
        .iter()
        .filter_map(|el| match el {
            PageElement::Shape(s) if s.node_id.ends_with("::edge_bottom") => s
                .node_id
                .strip_suffix("::edge_bottom")
                .map(str::to_string),
            _ => None,
        })
        .collect();
    if ruled.is_empty() {
        return;
    }
    for el in elements {
        let PageElement::TextBox(tb) = el else {
            continue;
        };
        if tb.wrap || !ruled.contains(&tb.node_id) {
            continue;
        }
        if tb
            .runs
            .iter()
            .any(|r| r.text.contains('\n') || r.hyperlink.is_some())
        {
            continue;
        }
        for run in &mut tb.runs {
            run.underline = false;
        }
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
        gradient: Option<crate::ir::GradientFill>,
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
                    gradient: s.gradient.clone(),
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
        if shape.gradient.is_some() {
            tb.gradient = shape.gradient.clone();
            tb.fill_hex = None;
            tb.fill_alpha = 255;
            // wrap=none sizes the frame to the glyphs, so a gradient pill
            // collapses to a leftover strip.
            tb.wrap = true;
        } else if shape.fill_hex.is_some() {
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
    // Writer composites the absorbed txBox fill over sibling `::edge_*` bars
    // on the same AABB, so square rims vanish. Shrink the fill by the bar
    // thickness; the bars keep the lock frame.
    let bar_w: BTreeMap<String, i64> = absorbed
        .iter()
        .map(|id| (id.clone(), edge_bar_thickness(elements, id)))
        .collect();
    for el in elements.iter_mut() {
        let PageElement::TextBox(tb) = el else {
            continue;
        };
        if let Some(&w) = bar_w.get(&tb.node_id) {
            inset_textbox_inside_edge_bars(tb, w);
        }
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
/// Writer paints a sibling non-txBox fill/raster above txBox labels, so hero
/// titles vanish unless they share the shell. `pic:pic` lock images stay
/// above that folded wps (side-by-side banner photos). Do **not** fold when
/// the shell contains a wps raster: merge keeps the caption's higher
/// `relativeHeight`, which raises the opaque card fill above the image
/// (blank FIG cards). Skip side-by-side labels, glass, and shells that still
/// hold child cards.
fn fold_shell_label_stacks(elements: &mut Vec<PageElement>, page_w_emu: i64, page_h_emu: i64) {
    const TOL: i64 = 12_700;
    let mut i = 0;
    while i < elements.len() {
        let Some(outer) = foldable_shell_aabb(&elements[i], page_w_emu, page_h_emu, TOL) else {
            i += 1;
            continue;
        };
        let shell_id = match &elements[i] {
            PageElement::Shape(s) => s.node_id.clone(),
            PageElement::Raster(p) => p.node_id.clone(),
            _ => {
                i += 1;
                continue;
            }
        };
        let mut labels = Vec::new();
        let mut side_pics = Vec::new();
        let mut blocked = false;
        for (j, el) in elements.iter().enumerate().skip(i + 1) {
            let Some(bj) = element_aabb(el) else {
                continue;
            };
            if !aabb_contains(outer, bj, TOL) {
                continue;
            }
            match el {
                // Same-node chips (text + box_decoration) already absorbed
                // their fill. Merging them as labels drops that fill and
                // centers the glyph-padded chip in the parent card.
                PageElement::TextBox(tb) if nested_chip_textbox(tb, &elements[i]) => {}
                PageElement::TextBox(_) => labels.push(j),
                PageElement::Shape(s) if is_shell_companion(&shell_id, &s.node_id) => {}
                PageElement::Shape(s) if nested_decor_allows_fold(s) => {}
                // pic:pic beside labels (hero photo) can stay a sibling.
                // pic:pic stacked with a caption is a FIG card — skip fold.
                PageElement::Picture(_) => side_pics.push(j),
                PageElement::Raster(_) => {
                    blocked = true;
                    break;
                }
                _ => {
                    blocked = true;
                    break;
                }
            }
        }
        let leftover_picture = !side_pics.is_empty()
            && side_pics.iter().all(|&pj| {
                labels.iter().all(|&lj| {
                    let (Some(pb), Some(lb)) = (element_aabb(&elements[pj]), element_aabb(&elements[lj]))
                    else {
                        return false;
                    };
                    !x_overlap(pb, lb)
                })
            });
        if !side_pics.is_empty() && !leftover_picture {
            blocked = true;
        }
        if blocked || labels.is_empty() || !labels_are_vertical_stack(elements, &labels) {
            i += 1;
            continue;
        }
        // A left icon next to a glyph-tight title is also leftover_picture
        // (no x-overlap). Reserving the empty right remainder as rIns squeezed
        // the wrap column to the ink width, so host-wider one-liners mid-wrapped.
        let picture_on_right =
            leftover_picture && pics_right_of_labels(elements, &side_pics, &labels);
        let left_icon_clear = if leftover_picture && !picture_on_right {
            leftover_left_icon_clear(elements, &side_pics, outer.0)
        } else {
            0
        };
        let Some(folded) = merge_labels_into_shell(
            &elements[i],
            elements,
            &labels,
            picture_on_right,
            leftover_picture,
            left_icon_clear,
        )
        else {
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
        raise_pictures_covered_by_shell(elements, outer, folded.relative_height, TOL);
        raise_nested_chips_above_shell(elements, outer, &folded, TOL);
        raise_nested_decor_above_shell(elements, outer, &folded, TOL);
        i += 1;
    }
}

fn is_shell_companion(shell_id: &str, other: &str) -> bool {
    other == format!("{shell_id}::stroke")
        || other.starts_with(&format!("{shell_id}::edge_"))
}

/// Thickness of `{id}::edge_*` bars (EMU). Square rims split to these after
/// `a:ln` on a filled text box is clipped by Writer.
fn edge_bar_thickness(elements: &[PageElement], shell_id: &str) -> i64 {
    let prefix = format!("{shell_id}::edge_");
    elements
        .iter()
        .filter_map(|el| match el {
            PageElement::Shape(s) if s.node_id.starts_with(&prefix) => {
                Some(s.cx_emu.min(s.cy_emu))
            }
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

/// Writer paints an opaque txBox over sibling edge bars that share its AABB.
/// Inset the fill so the bars keep the lock rim. Padding follows the shrink
/// so glyphs stay where the lock put them.
fn inset_textbox_inside_edge_bars(tb: &mut crate::ir::TextBox, bar_w: i64) {
    if bar_w <= 0 || tb.cx_emu <= bar_w * 2 || tb.cy_emu <= bar_w * 2 {
        return;
    }
    tb.x_emu += bar_w;
    tb.y_emu += bar_w;
    tb.cx_emu -= bar_w * 2;
    tb.cy_emu -= bar_w * 2;
    tb.l_ins_emu = (tb.l_ins_emu - bar_w).max(0);
    tb.t_ins_emu = (tb.t_ins_emu - bar_w).max(0);
    tb.r_ins_emu = (tb.r_ins_emu - bar_w).max(0);
    tb.b_ins_emu = (tb.b_ins_emu - bar_w).max(0);
}

/// Nested pill/badge whose opaque fill (or gradient/blip) is not the parent
/// shell. Dark Mode underlays that copy the shell RGB still fold.
fn nested_chip_textbox(tb: &crate::ir::TextBox, shell: &PageElement) -> bool {
    if tb.gradient.is_some() || tb.fill_blip.is_some() {
        return true;
    }
    let Some(hex) = tb.fill_hex.as_deref() else {
        return false;
    };
    if !covering_solid_alpha(tb.fill_alpha) {
        return false;
    }
    match shell {
        PageElement::Raster(_) => true,
        PageElement::Shape(s) => chip_fill_differs(
            hex,
            tb.fill_alpha,
            s.fill_hex.as_deref(),
            s.fill_alpha,
            s.gradient.is_some(),
        ),
        PageElement::TextBox(s) => chip_fill_differs(
            hex,
            tb.fill_alpha,
            s.fill_hex.as_deref(),
            s.fill_alpha,
            s.gradient.is_some() || s.fill_blip.is_some(),
        ),
        _ => false,
    }
}

fn chip_fill_differs(
    hex: &str,
    alpha: u8,
    shell_hex: Option<&str>,
    shell_alpha: u8,
    shell_chrome: bool,
) -> bool {
    if shell_chrome {
        return true;
    }
    match shell_hex {
        None => true,
        Some(shell_hex) => !solid_hex_eq(hex, shell_hex) || alpha != shell_alpha,
    }
}

/// Writer paints earlier anchors on top. A chip painted before body labels
/// keeps a lower `relativeHeight` than the folded card and vanishes under it.
fn raise_nested_chips_above_shell(
    elements: &mut [PageElement],
    outer: (i64, i64, i64, i64),
    folded: &crate::ir::TextBox,
    tol: i64,
) {
    let shell = PageElement::TextBox(folded.clone());
    for el in elements.iter_mut() {
        let PageElement::TextBox(tb) = el else {
            continue;
        };
        if tb.node_id == folded.node_id {
            continue;
        }
        if !aabb_contains(outer, (tb.x_emu, tb.y_emu, tb.cx_emu, tb.cy_emu), tol) {
            continue;
        }
        if !nested_chip_textbox(tb, &shell) {
            continue;
        }
        if tb.relative_height <= folded.relative_height {
            tb.relative_height = folded.relative_height.saturating_add(1);
        }
    }
}

/// Writer paints the folded opaque txBox over leftover nested DrawBoxes
/// (empty checkboxes, small form squares). Compact widgets stay siblings
/// via [`nested_decor_allows_fold`]; promote them above the fill. Covering
/// `{id}::stroke` outline companions stay under labels; thin `::edge_*` bars
/// must sit above the folded fill so Writer does not composite the card over
/// the rim.
fn raise_nested_decor_above_shell(
    elements: &mut [PageElement],
    outer: (i64, i64, i64, i64),
    folded: &crate::ir::TextBox,
    tol: i64,
) {
    for el in elements.iter_mut() {
        let PageElement::Shape(s) = el else {
            continue;
        };
        if s.node_id == folded.node_id {
            continue;
        }
        if is_shell_companion(&folded.node_id, &s.node_id) {
            if s.node_id.contains("::edge_") && s.relative_height <= folded.relative_height {
                s.relative_height = folded.relative_height.saturating_add(1);
            }
            continue;
        }
        let aabb = (s.x_emu, s.y_emu, s.cx_emu, s.cy_emu);
        if !aabb_contains(outer, aabb, tol) || aabb_same(aabb, outer, tol) {
            continue;
        }
        if s.relative_height <= folded.relative_height {
            s.relative_height = folded.relative_height.saturating_add(1);
        }
    }
}

fn solid_hex_eq(a: &str, b: &str) -> bool {
    fn norm(h: &str) -> String {
        let t = h.trim().trim_start_matches('#').to_ascii_uppercase();
        match t.as_str() {
            "FFFFFE" => "FFFFFF".into(),
            "000001" => "000000".into(),
            other => other.to_string(),
        }
    }
    norm(a) == norm(b)
}

/// Writer paints an in-front non-txBox fill above later txBox/raster siblings.
/// Overlay banners paint a parent DrawBox then a same-size child image; after
/// folding titles into the image, that leftover fill is a solid bar over the
/// art. Drop the redundant shell. Smaller FIG photos do not cover the card.
fn drop_shells_covered_by_later_media(elements: &mut Vec<PageElement>) {
    const TOL: i64 = 12_700;
    let bounds: Vec<Option<(i64, i64, i64, i64)>> = elements.iter().map(element_aabb).collect();
    let mut drop: HashSet<usize> = HashSet::new();
    for i in 0..elements.len() {
        let PageElement::Shape(s) = &elements[i] else {
            continue;
        };
        if s.behind_doc || crate::geo::is_thin_fill_emu(s.cx_emu, s.cy_emu) {
            continue;
        }
        let Some(bi) = bounds[i] else {
            continue;
        };
        for j in i + 1..elements.len() {
            if !element_is_covering_media(&elements[j]) {
                continue;
            }
            let Some(bj) = bounds[j] else {
                continue;
            };
            if aabb_same(bi, bj, TOL) {
                drop.insert(i);
                break;
            }
        }
    }
    if drop.is_empty() {
        return;
    }
    *elements = elements
        .iter()
        .enumerate()
        .filter(|(k, _)| !drop.contains(k))
        .map(|(_, el)| el.clone())
        .collect();
}

fn element_is_covering_media(el: &PageElement) -> bool {
    match el {
        PageElement::Picture(_) | PageElement::Raster(_) => true,
        PageElement::TextBox(tb) => tb.fill_blip.is_some(),
        _ => false,
    }
}

fn aabb_same(a: (i64, i64, i64, i64), b: (i64, i64, i64, i64), tol: i64) -> bool {
    aabb_contains(a, b, tol) && aabb_contains(b, a, tol)
}

fn foldable_shell_aabb(
    el: &PageElement,
    page_w_emu: i64,
    page_h_emu: i64,
    tol: i64,
) -> Option<(i64, i64, i64, i64)> {
    let full = |cx: i64, cy: i64| cx + tol >= page_w_emu && cy + tol >= page_h_emu;
    match el {
        PageElement::Shape(s) => {
            if crate::geo::is_thin_fill_emu(s.cx_emu, s.cy_emu) || full(s.cx_emu, s.cy_emu) {
                return None;
            }
            if s.gradient.is_some() {
                return Some((s.x_emu, s.y_emu, s.cx_emu, s.cy_emu));
            }
            if s.fill_hex.is_some() && covering_solid_alpha(s.fill_alpha) {
                return Some((s.x_emu, s.y_emu, s.cx_emu, s.cy_emu));
            }
            None
        }
        PageElement::Raster(p) => {
            if crate::geo::is_thin_fill_emu(p.cx_emu, p.cy_emu) || full(p.cx_emu, p.cy_emu) {
                return None;
            }
            Some((p.x_emu, p.y_emu, p.cx_emu, p.cy_emu))
        }
        _ => None,
    }
}

/// Hairline rules and stroke-only photo frames sit in a banner without being
/// child cards. They stay as siblings; they must not block the label fold.
fn nested_decor_allows_fold(s: &ShapeBox) -> bool {
    crate::geo::is_thin_fill_emu(s.cx_emu, s.cy_emu)
        || (s.fill_hex.is_none() && s.gradient.is_none())
}

fn x_overlap(a: (i64, i64, i64, i64), b: (i64, i64, i64, i64)) -> bool {
    let (ax, _ay, aw, _ah) = a;
    let (bx, _by, bw, _bh) = b;
    ax.max(bx) < (ax + aw).min(bx + bw)
}

/// True when a leftover pic sits to the right of every label (hero photo
/// column). Left icons must not take this path — the empty remainder after a
/// glyph-tight title is padding, not a photo.
/// How far `lIns` must stay so leftover left icons are not covered.
fn leftover_left_icon_clear(
    elements: &[PageElement],
    side_pics: &[usize],
    shell_x: i64,
) -> i64 {
    side_pics
        .iter()
        .filter_map(|&pj| element_aabb(&elements[pj]))
        .map(|(x, _, w, _)| (x.saturating_add(w) - shell_x).max(0))
        .max()
        .unwrap_or(0)
}

fn pics_right_of_labels(
    elements: &[PageElement],
    side_pics: &[usize],
    labels: &[usize],
) -> bool {
    let Some(labels_right) = labels
        .iter()
        .filter_map(|&j| element_aabb(&elements[j]))
        .map(|(x, _, w, _)| x.saturating_add(w))
        .max()
    else {
        return false;
    };
    side_pics.iter().any(|&pj| {
        element_aabb(&elements[pj])
            .map(|(x, _, _, _)| x >= labels_right)
            .unwrap_or(false)
    })
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
    picture_on_right: bool,
    leftover_picture: bool,
    left_icon_clear_emu: i64,
) -> Option<crate::ir::TextBox> {
    let (shell_id, x, y, cx, cy, fill_hex, fill_alpha, gradient, fill_blip, corner_emu, line_shell) =
        match shell_el {
            PageElement::Shape(shell) => (
                shell.node_id.clone(),
                shell.x_emu,
                shell.y_emu,
                shell.cx_emu,
                shell.cy_emu,
                shell.fill_hex.clone(),
                shell.fill_alpha,
                shell.gradient.clone(),
                None,
                shell.corner_emu,
                Some(shell),
            ),
            PageElement::Raster(pic) => (
                pic.node_id.clone(),
                pic.x_emu,
                pic.y_emu,
                pic.cx_emu,
                pic.cy_emu,
                None,
                255,
                None,
                Some(pic.clone()),
                0,
                None,
            ),
            _ => return None,
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
    folded.node_id = shell_id.clone();
    folded.x_emu = x;
    folded.y_emu = y;
    folded.cx_emu = cx;
    folded.cy_emu = cy;
    // Use the tightest insets that still fit every label. Sizing only to the
    // first (often a narrow kicker) left a wrap column too narrow for later
    // display figures, so host-bold "SOUNDSTAGE" / "23:00 CST" mid-wrapped.
    folded.l_ins_emu = tbs
        .iter()
        .map(|tb| (tb.x_emu - x + tb.l_ins_emu).max(0))
        .min()
        .unwrap_or(0);
    folded.t_ins_emu = (first.y_emu - y + first.t_ins_emu).max(0);
    // Widen to the shell: a kicker-narrow column mid-wraps host-bold tracked
    // labels. Keep the lock left pad and mirror it on the right so the wrap
    // column is the card interior (still inset, not edge-flush).
    // Right-side photos only: keep the labels' right remainder so wrap does
    // not flow under the pic:pic column. Left icons already sit in lIns.
    let widest_r = tbs
        .iter()
        .map(|tb| (x + cx - tb.x_emu - tb.cx_emu + tb.r_ins_emu).max(0))
        .min()
        .unwrap_or(0);
    folded.r_ins_emu = if picture_on_right {
        widest_r
    } else {
        folded.l_ins_emu.min(widest_r)
    };
    folded.b_ins_emu = tbs
        .last()
        .map(|last| (y + cy - last.y_emu - last.cy_emu + last.b_ins_emu).max(0))
        .unwrap_or(0);
    folded.fill_hex = fill_hex;
    folded.fill_alpha = fill_alpha;
    folded.gradient = gradient;
    folded.fill_blip = fill_blip;
    folded.corner_emu = corner_emu;
    // Line may live on the shell or on `{id}::stroke` after fill/stroke split.
    let stroke = elements.iter().find_map(|el| match el {
        PageElement::Shape(s) if s.node_id == format!("{shell_id}::stroke") => Some(s),
        _ => None,
    });
    let line_src = stroke
        .filter(|s| s.line_hex.is_some())
        .or(line_shell)
        .filter(|s| s.line_hex.is_some());
    if let Some(src) = line_src {
        folded.line_hex = src.line_hex.clone();
        folded.line_alpha = src.line_alpha;
        folded.line_w_emu = src.line_w_emu;
        folded.line_dash = src.line_dash;
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
    //
    // Quote/callout cards are also single-label (or quote+attribution) but
    // the copy is a wrapping paragraph. Centering + NBSP left one overflowing
    // line and hid the author below.
    if tbs.len() == 1 {
        folded.para_line_twips = Vec::new();
        folded.para_after_twips = Vec::new();
        folded.para_before_twips = Vec::new();
        folded.b_ins_emu = 0;
        if fold_label_is_wrapping_body(first) {
            // Quote/callout: body is inset in a taller shell → keep `anchor=t`
            // and tIns from the body's lock origin (padding + any inner
            // centering). Same-AABB white cells (table/card content) can
            // still use the body's `vert_center`.
            let same_box = (first.y_emu - y).abs() <= 12_700
                && (first.cy_emu - cy).abs() <= 12_700;
            folded.vert_center = same_box && first.vert_center;
            let pitch = first
                .line_twips
                .unwrap_or_else(|| label_line_twips(first));
            folded.line_twips = Some(pitch);
            folded.last_line_twips = Some(pitch);
        } else {
            let pitch = label_line_twips(first);
            folded.line_twips = Some(pitch);
            folded.last_line_twips = Some(pitch);
            folded.vert_center = true;
            folded.t_ins_emu = 0;
        }
        // Icon+label pills size lIns to the label origin, leaving a wrap
        // column exactly the lock text width. Host-wider metrics then
        // mid-wrap one-liners. Trim side pads so the column has slack.
        // Left icons: do not shrink lIns past the icon's right edge (the
        // gap between icon and label is usable slack). Right photos keep
        // the full remainder so wrap stays left of the image.
        let trimmed_l = folded.l_ins_emu - folded.l_ins_emu / 4;
        // Leftover left icons: start the wrap column at the icon's right
        // edge. Keeping the full label-origin lIns made the column exactly
        // the lock text width, so host-wider "PPTX" mid-wrapped. Never go
        // left of the icon. No leftover icon → 25% trim as before.
        folded.l_ins_emu = if leftover_picture && left_icon_clear_emu > 0 {
            left_icon_clear_emu
        } else {
            trimmed_l
        };
        if !picture_on_right {
            folded.r_ins_emu = folded.r_ins_emu - folded.r_ins_emu / 4;
        }
    } else {
        folded.line_twips = None;
        folded.last_line_twips = None;
        folded.align = fold_stack_align(&tbs);
        folded.para_align = fold_stack_para_align(&tbs);
        let (para_line, para_after, para_before) = fold_stack_spacing(&tbs);
        folded.para_line_twips = para_line;
        folded.para_after_twips = para_after;
        folded.para_before_twips = para_before;
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
    let widest = tbs.iter().max_by_key(|tb| tb.cx_emu).copied();
    for (n, tb) in tbs.iter().enumerate() {
        if n > 0 {
            if let Some(template) = tb.runs.first().or(folded.runs.last()) {
                folded.runs.push(newline_run(template));
            }
        }
        // One-liners and lock-pinned titles must not host-soft-wrap. Soft-
        // wrapped bodies (one para, wrap=square) keep ordinary spaces. wrap=none
        // quotes/callouts still need those spaces once the padded card is
        // wrap=square — NBSP + a pinned `\n` re-wrapped the last word ("the")
        // onto its own row, then started the next lock line under it.
        let mut label_runs = if fold_label_is_wrapping_body(tb) {
            restore_wrapping_spaces(tb.runs.iter().cloned())
        } else if !tb.wrap || label_para_count(tb) > 1 {
            nobreak_spaces(tb.runs.iter().cloned())
        } else {
            tb.runs.clone()
        };
        // Shared bodyPr lIns follows the leftmost label (often a title). A
        // leftover-width row beside a checkbox sits further right; hosts
        // ignore w:ind in drawing boxes, so pad that gutter with NBSPs.
        if tbs.len() > 1 {
            let extra_l = (tb.x_emu + tb.l_ins_emu - folded.x_emu - folded.l_ins_emu).max(0);
            // List items already paint a literal `•` / `{n}.` in the marker
            // gutter. NBSP-padding that same gutter double-indents the wrap
            // column so 8.5pt bullets look oversized and clip.
            if extra_l >= FOLD_CENTER_MIN_EMU
                && !tb.bullet
                && !tb.numbered
                && fold_label_align(tb, widest) == crate::ir::TextAlign::Left
            {
                pad_left_nbsp(&mut label_runs, extra_l);
            }
        }
        folded.runs.extend(label_runs);
    }
    // Writer paints this opaque txBox over sibling `::edge_*` bars that share
    // the shell AABB (menu title frames, exhibition cards). Inset the fill so
    // the bars keep the lock rim.
    inset_textbox_inside_edge_bars(
        &mut folded,
        edge_bar_thickness(elements, &shell_id),
    );
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

/// Keep each stacked label's own `w:jc`. A centered notice title folded with
/// left body used to inherit the widest (body) align and paint left.
///
/// Parent `align_items: center` shrink-wraps a title so infer sees a
/// glyph-tight Left box. After fold that Left paints at the body's lIns.
/// A narrower label inset equally from the content column was centered.
fn fold_stack_para_align(tbs: &[&crate::ir::TextBox]) -> Vec<crate::ir::TextAlign> {
    let widest = tbs.iter().max_by_key(|tb| tb.cx_emu).copied();
    let mut out = Vec::new();
    for tb in tbs {
        let align = fold_label_align(tb, widest);
        let n = label_para_count(tb);
        out.extend(std::iter::repeat(align).take(n));
    }
    out
}

/// 1pt — lock leftover below this is padding noise, not a centered inset.
const FOLD_CENTER_MIN_EMU: i64 = 12_700;

fn fold_label_align(
    tb: &crate::ir::TextBox,
    widest: Option<&crate::ir::TextBox>,
) -> crate::ir::TextAlign {
    if tb.bullet || tb.numbered {
        return crate::ir::TextAlign::Left;
    }
    if !matches!(tb.align, crate::ir::TextAlign::Left) {
        return tb.align;
    }
    let Some(widest) = widest else {
        return tb.align;
    };
    if tb.node_id == widest.node_id || tb.cx_emu >= widest.cx_emu {
        return tb.align;
    }
    let inset_l = tb.x_emu.saturating_sub(widest.x_emu);
    let inset_r = widest
        .x_emu
        .saturating_add(widest.cx_emu)
        .saturating_sub(tb.x_emu.saturating_add(tb.cx_emu));
    let slack = inset_l.saturating_add(inset_r);
    if inset_l >= FOLD_CENTER_MIN_EMU
        && inset_r >= FOLD_CENTER_MIN_EMU
        && (inset_l - inset_r).abs().saturating_mul(5) <= slack
    {
        return crate::ir::TextAlign::Center;
    }
    if inset_l >= FOLD_CENTER_MIN_EMU && inset_r < FOLD_CENTER_MIN_EMU {
        // Glyph-tight box sitting on the right of the content column.
        // Leftover-width sibling columns (checkbox + task text) stay Left —
        // upgrading to Right uses the shared card lIns and shifts the copy.
        let remaining = widest.cx_emu.saturating_sub(inset_l);
        if remaining > 0 && tb.cx_emu.saturating_mul(5) >= remaining.saturating_mul(4) {
            return tb.align;
        }
        return crate::ir::TextAlign::Right;
    }
    tb.align
}

/// Per-paragraph line pitch + inter-label `after` from lock y gaps.
fn fold_stack_spacing(
    tbs: &[&crate::ir::TextBox],
) -> (Vec<Option<i64>>, Vec<i64>, Vec<i64>) {
    let mut para_line = Vec::new();
    let mut para_after = Vec::new();
    let mut para_before = Vec::new();
    let mut gap_before_next = 0i64;
    for (i, tb) in tbs.iter().enumerate() {
        let n = label_para_count(tb);
        let pitch = label_stack_pitch(tb);
        let face = label_line_twips(tb);
        let vpad = vertical_pad_twips(tb, face);
        // Compact top-aligned one-liners (heading + padding_bottom rule):
        // exact pitch = box height sits glyphs on the `::edge_bottom` bar.
        // Keep face pitch and put the leftover after the line.
        let below = if vpad == 0
            && !tb.vert_center
            && n == 1
            && compact_fold_label(tb, face)
            && pitch > face
        {
            pitch - face
        } else {
            0
        };
        let line_pitch = if vpad > 0 || below > 0 { face } else { pitch };
        let more = i + 1 < tbs.len();
        for p in 0..n {
            let is_last = p + 1 == n;
            para_before.push(if p == 0 {
                vpad + gap_before_next
            } else {
                0
            });
            gap_before_next = 0;
            // Full pitch on every para (including the stack's last). Face-only
            // last pitch + bodyPr bIns was clipping PASSBAND / last desc lines
            // inside cards that still had empty shell below.
            para_line.push(Some(line_pitch));
            let gap_after = if is_last && more {
                let next = tbs[i + 1];
                let gap_emu = next
                    .y_emu
                    .saturating_sub(tb.y_emu.saturating_add(tb.cy_emu));
                crate::coord::emu_to_twips(gap_emu).max(0)
            } else {
                0
            };
            let after = if vpad > 0 && is_last {
                vpad
            } else if below > 0 && is_last {
                below + gap_after
            } else {
                gap_after
            };
            if vpad > 0 && is_last {
                gap_before_next = gap_after;
            }
            para_after.push(after);
        }
    }
    (para_line, para_after, para_before)
}

/// Lock-centered single-line rows in a folded stack (ruled note lines, pills).
fn vertical_pad_twips(tb: &crate::ir::TextBox, face: i64) -> i64 {
    // Wrapped list items size `cy` to the N-line lock box (face × leading).
    // `(cy − face)/2` is pill padding and would open a huge gap between
    // bullets. Use lock y-gaps (`after`) and inter-line pitch instead.
    if tb.bullet || tb.numbered {
        return 0;
    }
    if label_para_count(tb) != 1 {
        return 0;
    }
    // Wrapping bodies are N lines in a tall box. `(cy − face)/2` is the
    // one-line pill pad and would shove a paragraph into the leftover.
    if fold_label_is_wrapping_body(tb) {
        return 0;
    }
    let cell = crate::coord::emu_to_twips(tb.cy_emu).max(20);
    if cell <= face {
        return 0;
    }
    if !tb.vert_center {
        // Grid / note-line rows: lock cell is taller than one face but glyphs
        // were not flagged centered. Skip compact pills — those keep box-height
        // pitch in folded stacks (badges beside multi-line bodies).
        let pitch = label_stack_pitch(tb);
        if cell <= face || pitch <= face || compact_fold_label(tb, face) {
            return 0;
        }
    }
    (cell - face).max(0) / 2
}

fn compact_fold_label(tb: &crate::ir::TextBox, face: i64) -> bool {
    let half = tb
        .runs
        .iter()
        .filter(|r| r.text != "\n" && !r.text.is_empty())
        .map(|r| i64::from(r.sz_half_points))
        .max()
        .unwrap_or(24);
    let face_emu = half.saturating_mul(6_350);
    face_emu > 0 && tb.cy_emu <= face_emu.saturating_mul(5) / 2
}

fn label_para_count(tb: &crate::ir::TextBox) -> usize {
    if fold_label_is_wrapping_body(tb) {
        return 1;
    }
    raw_para_count(tb)
}

fn raw_para_count(tb: &crate::ir::TextBox) -> usize {
    let breaks = tb
        .runs
        .iter()
        .map(|r| r.text.chars().filter(|c| *c == '\n').count())
        .sum::<usize>();
    breaks.saturating_add(1).max(1)
}

/// Inter-line pitch for a folded label.
///
/// One lock paragraph: extra `cy` is padding (note-line / underline cells,
/// ~2.5–3× face). Use the box height so folded stacks keep the underline
/// gap. wrap=square does **not** mean multi-line — leftover-width one-liners
/// also wrap=square, with `line_twips=None`.
///
/// Soft-wrapped N-line bodies are the exception: wrap=square, one `<w:p>`,
/// and a real lock inter-line `line_twips`. Using `cy` there would ~N× the
/// leading (callout quote blow-up).
///
/// Pinned `\n` (para_count > 1) always uses lock line pitch, not 2× box.
fn label_stack_pitch(tb: &crate::ir::TextBox) -> i64 {
    let from_box = crate::coord::emu_to_twips(tb.cy_emu).max(20);
    let face = label_line_twips(tb);
    if tb.bullet || tb.numbered {
        // Wrapped lists pin U+2028 inside one paragraph, so para_count is 1
        // even when lock had two lines. Box height as pitch would double
        // that leading. Use lock inter-line delta when we have it.
        if let Some(v) = tb.line_twips {
            return v.max(20);
        }
        return from_box.max(face);
    }
    // wrap=none long copy still uses lock leading, not the N-line box height
    // (that spacing shoved the attribution out of quote cards).
    if fold_label_is_wrapping_body(tb) {
        return tb.line_twips.map(|v| v.max(20)).unwrap_or(face);
    }
    if label_para_count(tb) != 1 {
        return tb.line_twips.map(|v| v.max(20)).unwrap_or(face);
    }
    // Soft-wrapped body: one para, wrap on, lock leading from N glyph lines.
    if tb.wrap {
        if let Some(v) = tb.line_twips {
            if from_box > face.saturating_mul(2) {
                return v.max(20);
            }
        }
    }
    from_box.max(face)
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

/// Hosts ignore `w:ind` in drawing text boxes. ~0.25em per NBSP, same as
/// list wrap pad, so a checkbox gutter survives fold into a shared lIns.
fn pad_left_nbsp(runs: &mut Vec<TextRun>, extra_emu: i64) {
    let Some(first) = runs.iter().find(|r| !r.text.is_empty()) else {
        return;
    };
    let em_emu = crate::coord::millipt_to_emu(i64::from(first.sz_half_points.max(1)) * 500);
    let nbsp_emu = (em_emu / 4).max(1);
    let n = ((extra_emu + nbsp_emu / 2) / nbsp_emu).clamp(1, 40) as usize;
    let mut spacer = first.clone();
    spacer.text = "\u{00A0}".repeat(n);
    spacer.hyperlink = None;
    spacer.field = None;
    runs.insert(0, spacer);
}

/// Long paragraph in a padded card (quotes, callouts). Compact pills and
/// ruled one-liners stay below this — they still need NBSP + vertical center.
fn fold_label_is_wrapping_body(tb: &crate::ir::TextBox) -> bool {
    if tb.bullet || tb.numbered {
        return false;
    }
    if tb.wrap && tb.line_twips.is_some() && raw_para_count(tb) == 1 {
        return true;
    }
    let face = label_line_twips(tb);
    let h = crate::coord::emu_to_twips(tb.cy_emu);
    let chars = tb
        .runs
        .iter()
        .map(|r| r.text.chars().filter(|c| !c.is_whitespace()).count())
        .sum::<usize>();
    chars > 40 && h > face.saturating_mul(2)
}

fn restore_wrapping_spaces<I>(runs: I) -> Vec<TextRun>
where
    I: IntoIterator<Item = TextRun>,
{
    let mut out = Vec::new();
    for mut r in runs {
        if r.text.contains('\u{00A0}') {
            r.text = r.text.replace('\u{00A0}', " ");
        }
        // Lock pins insert a `\n` run at the rustybuzz wrap. Drop it so the
        // host reflows one paragraph; keeping the break orphans the last
        // word of line 1 when host metrics are slightly wider.
        if r.text == "\n" {
            continue;
        }
        if r.text.contains('\n') {
            r.text = r.text.replace('\n', " ");
            while r.text.contains("  ") {
                r.text = r.text.replace("  ", " ");
            }
        }
        if !r.text.is_empty() {
            out.push(r);
        }
    }
    out
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
                if (tb.fill_hex.is_some() && covering_solid_alpha(tb.fill_alpha))
                    || tb.gradient.is_some()
                    || tb.fill_blip.is_some() =>
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
        // `{id}::stroke` matches the shell AABB. Other stroke-only shapes
        // include nested checkboxes: demote only when the outline *contains*
        // a label, not when a folded card contains the widget.
        let nested_widget = !s.node_id.ends_with("::stroke");
        let mut min_text = None;
        for &(rel, tb) in &texts {
            let covers_text = if nested_widget {
                aabb_contains(aabb, tb, 12_700) && !aabb_contains(tb, aabb, 12_700)
            } else {
                aabbs_intersect(aabb, tb)
            };
            if covers_text {
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
    // Stroke-only decorative frame (no fill). Nested checkboxes match this
    // too; [`demote_outline_frames_below_text`] keeps widgets that sit
    // *inside* a folded card and only demotes frames around labels.
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

/// Writer z-orders `pic:pic` by `relativeHeight` with `wps:wsp` (not always
/// on top). Fold expands the shell over a leftover icon that was classified
/// as `pic:pic` before the merge, so the gradient pill hid the glyph. Promote
/// those pictures onto the shape stack above the folded box.
fn raise_pictures_covered_by_shell(
    elements: &mut Vec<PageElement>,
    outer: (i64, i64, i64, i64),
    fold_rel: u32,
    tol: i64,
) {
    for i in 0..elements.len() {
        let (contained, below) = match &elements[i] {
            PageElement::Picture(p) | PageElement::Raster(p) => (
                aabb_contains(outer, (p.x_emu, p.y_emu, p.cx_emu, p.cy_emu), tol),
                p.relative_height <= fold_rel,
            ),
            _ => continue,
        };
        if !contained || !below {
            continue;
        }
        match &mut elements[i] {
            PageElement::Picture(p) | PageElement::Raster(p) => {
                p.relative_height = fold_rel.saturating_add(1);
                p.pin_empty_txbox = false;
            }
            _ => {}
        }
        if let PageElement::Picture(p) = elements[i].clone() {
            elements[i] = PageElement::Raster(p);
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
            src_l: 0,
            src_t: 0,
            src_r: 0,
            src_b: 0,
            corner_emu: 0,
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
            para_before_twips: Vec::new(),
            para_align: Vec::new(),
            vert_center: false,
            preserve_whitespace: false,
            relative_height: rel,
            fill_hex: None,
            fill_alpha: 255,
            fill_blip: None,
            gradient: None,
            wrap: false,
            corner_emu: 0,
            line_hex: None,
            line_alpha: 255,
            line_w_emu: 0,
            line_dash: crate::ir::LineDash::Solid,
        }
    }

    #[test]
    fn wrap_none_heading_drops_native_u_when_bottom_bar_exists() {
        let mut tb = title_box(40);
        tb.node_id = "doc.h2".into();
        tb.wrap = false;
        tb.runs = vec![crate::ir::TextRun {
            text: "Heading".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 28,
            bold: true,
            italic: false,
            underline: true,
            strike: false,
            color_hex: "151515".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut bar = paper_shape(10);
        bar.node_id = "doc.h2::edge_bottom".into();
        bar.cy_emu = 12_700;
        let mut elements = vec![PageElement::Shape(bar), PageElement::TextBox(tb)];
        drop_redundant_underline_on_bottom_rules(&mut elements);
        let PageElement::TextBox(got) = &elements[1] else {
            panic!("textbox");
        };
        assert!(
            !got.runs[0].underline,
            "native w:u must yield to the lock bottom bar"
        );
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
            font_family_key: String::new(),
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
    fn fold_insets_fill_so_edge_bars_keep_the_rim() {
        let mut shell = paper_shape(10);
        shell.node_id = "menu.header.box_center".into();
        shell.behind_doc = false;
        shell.x_emu = 1_000_000;
        shell.y_emu = 1_000_000;
        shell.cx_emu = 4_000_000;
        shell.cy_emu = 1_000_000;
        shell.fill_hex = Some("FFFFFE".into());
        let bar_w = 9_525;
        let mut top = shell.clone();
        top.node_id = "menu.header.box_center::edge_top".into();
        top.fill_hex = Some("D32F2F".into());
        top.cy_emu = bar_w;
        let mut label = title_box(20);
        label.node_id = "menu.header.title_main".into();
        label.x_emu = 1_200_000;
        label.y_emu = 1_200_000;
        label.cx_emu = 3_000_000;
        label.cy_emu = 400_000;
        label.l_ins_emu = 50_000;
        label.runs = vec![TextRun {
            text: "CRIMSON BISTRO".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 36,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "111111".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut elements = vec![
            PageElement::Shape(shell),
            PageElement::Shape(top),
            PageElement::TextBox(label),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        assert_eq!(elements.len(), 2, "fill folds; edge bar stays, got {elements:?}");
        let got = elements
            .iter()
            .find_map(PageElement::textbox)
            .expect("folded");
        assert_eq!(got.x_emu, 1_000_000 + bar_w);
        assert_eq!(got.y_emu, 1_000_000 + bar_w);
        assert_eq!(got.cx_emu, 4_000_000 - bar_w * 2);
        assert_eq!(got.cy_emu, 1_000_000 - bar_w * 2);
        assert!(
            elements
                .iter()
                .any(|el| matches!(el, PageElement::Shape(s) if s.node_id.ends_with("::edge_top"))),
            "rim bar must remain a sibling, got {elements:?}"
        );
        let edge = elements
            .iter()
            .find_map(|el| match el {
                PageElement::Shape(s) if s.node_id.ends_with("::edge_top") => Some(s.relative_height),
                _ => None,
            })
            .expect("edge");
        assert!(
            edge > got.relative_height,
            "rim bar must stack above folded fill, got edge={edge} fold={}",
            got.relative_height
        );
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
    fn absorb_copies_gradient_into_same_node_textbox() {
        let mut pill = paper_shape(10);
        pill.node_id = "chip".into();
        pill.behind_doc = false;
        pill.fill_hex = None;
        pill.gradient = Some(crate::ir::GradientFill {
            angle_degrees: 135,
            stops: vec![
                crate::ir::GradientStopFill {
                    pos: 0,
                    hex: "FF8A00".into(),
                    alpha: 255,
                },
                crate::ir::GradientStopFill {
                    pos: 1000,
                    hex: "FF5E00".into(),
                    alpha: 255,
                },
            ],
        });
        pill.corner_emu = 50_000;
        pill.x_emu = 100_000;
        pill.y_emu = 100_000;
        pill.cx_emu = 500_000;
        pill.cy_emu = 200_000;
        let mut tb = title_box(20);
        tb.node_id = "chip".into();
        tb.x_emu = pill.x_emu;
        tb.y_emu = pill.y_emu;
        tb.cx_emu = pill.cx_emu;
        tb.cy_emu = pill.cy_emu;
        let mut elements = vec![PageElement::Shape(pill), PageElement::TextBox(tb)];
        absorb_self_fill_shapes(&mut elements);
        assert_eq!(elements.len(), 1, "gradient pill + label → one text box");
        let got = elements[0].textbox().expect("chip");
        assert!(got.gradient.is_some(), "same-node gradient must fold, got {:?}", got.gradient);
        assert!(got.fill_hex.is_none());
        assert!(got.wrap, "gradient pill must pin lock extent");
        assert_eq!(got.corner_emu, 50_000);
    }

    #[test]
    fn fold_raises_leftover_icon_above_shell() {
        let mut pill = paper_shape(10);
        pill.node_id = "format.pptx".into();
        pill.behind_doc = false;
        pill.x_emu = 100_000;
        pill.y_emu = 100_000;
        pill.cx_emu = 800_000;
        pill.cy_emu = 300_000;
        pill.fill_hex = Some("FF8A00".into());
        pill.corner_emu = 50_000;
        let mut icon = glow(15);
        icon.node_id = "format.pptx.icon".into();
        icon.x_emu = 140_000;
        icon.y_emu = 160_000;
        icon.cx_emu = 140_000;
        icon.cy_emu = 140_000;
        let mut label = title_box(20);
        label.node_id = "format.pptx.label".into();
        label.x_emu = 320_000;
        label.y_emu = 160_000;
        label.cx_emu = 500_000;
        label.cy_emu = 140_000;
        label.runs = vec![TextRun {
            text: "PPTX".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 16,
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
        let mut elements = vec![
            PageElement::Shape(pill),
            PageElement::Picture(icon),
            PageElement::TextBox(label),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        let tb = elements
            .iter()
            .find_map(PageElement::textbox)
            .expect("folded pill");
        assert_eq!(tb.fill_hex.as_deref(), Some("FF8A00"));
        let pic = elements.iter().find_map(|el| match el {
            PageElement::Raster(p) | PageElement::Picture(p) => Some(p),
            _ => None,
        });
        let pic = pic.expect("icon must remain");
        assert!(
            matches!(
                elements.iter().find(|el| match el {
                    PageElement::Raster(p) | PageElement::Picture(p) => p.node_id.ends_with(".icon"),
                    _ => false,
                }),
                Some(PageElement::Raster(_))
            ),
            "covered icon must join the wps stack, got {elements:?}"
        );
        assert!(
            pic.relative_height > tb.relative_height,
            "icon must sit above the folded fill, icon={} shell={}",
            pic.relative_height,
            tb.relative_height
        );
        let icon_clear = pic.x_emu + pic.cx_emu - 100_000;
        assert!(
            tb.l_ins_emu >= icon_clear,
            "lIns must stay at/after the icon, lIns={} clear={}",
            tb.l_ins_emu,
            icon_clear
        );
        let inner = tb.cx_emu - tb.l_ins_emu - tb.r_ins_emu;
        assert!(
            inner > 500_000,
            "wrap column must be wider than the lock label, inner={inner}"
        );
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
            font_family_key: String::new(),
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
            font_family_key: String::new(),
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
            font_family_key: String::new(),
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
            font_family_key: String::new(),
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
            font_family_key: String::new(),
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
            font_family_key: String::new(),
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
        assert_eq!(
            got.para_align,
            vec![TextAlign::Center, TextAlign::Left, TextAlign::Left],
            "each folded label keeps its own jc"
        );
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
    fn fold_keeps_list_items_left_even_when_narrower_than_body() {
        // Glyph-tight short bullets sit inset from a wider card body. The
        // leftover looks like Right to fold_label_align; lists must stay Left.
        let mut card = paper_shape(10);
        card.node_id = "phase".into();
        card.behind_doc = false;
        card.x_emu = 1_000_000;
        card.y_emu = 1_000_000;
        card.cx_emu = 4_000_000;
        card.cy_emu = 2_000_000;
        card.fill_hex = Some("FFFFFF".into());
        let mut title = title_box(20);
        title.node_id = "phase.title".into();
        title.align = TextAlign::Left;
        title.x_emu = 1_200_000;
        title.y_emu = 1_100_000;
        title.cx_emu = 3_600_000;
        title.cy_emu = 200_000;
        title.runs = vec![TextRun {
            text: "Strategic Foundations".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 22,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "0F172A".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut item = title_box(30);
        item.node_id = "phase.li".into();
        item.align = TextAlign::Left;
        item.bullet = true;
        item.line_twips = Some(255);
        item.x_emu = 1_800_000;
        item.y_emu = 1_400_000;
        item.cx_emu = 3_000_000;
        item.cy_emu = 200_000;
        item.runs = vec![TextRun {
            text: "•\u{00A0}Outcome-driven OKR mapping".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 18,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "0F172A".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut elements = vec![
            PageElement::Shape(card),
            PageElement::TextBox(title),
            PageElement::TextBox(item),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 6_858_000);
        let got = elements[0].textbox().expect("folded");
        assert_eq!(got.para_align, vec![TextAlign::Left, TextAlign::Left]);
    }

    #[test]
    fn fold_keeps_leftover_width_gutter_column_left() {
        // Title spans the card; task text is leftover-width in the column
        // after a 24pt checkbox gutter. That leftover used to look like Right.
        let mut card = paper_shape(10);
        card.node_id = "sched".into();
        card.behind_doc = false;
        card.x_emu = 1_000_000;
        card.y_emu = 1_000_000;
        card.cx_emu = 4_000_000;
        card.cy_emu = 2_000_000;
        card.fill_hex = Some("F7F0E8".into());
        let mut title = title_box(20);
        title.node_id = "sched.title".into();
        title.align = TextAlign::Left;
        title.x_emu = 1_200_000;
        title.y_emu = 1_100_000;
        title.cx_emu = 3_600_000;
        title.cy_emu = 200_000;
        title.runs = vec![dummy_run("TODAY'S SCHEDULE", 18)];
        let mut task = title_box(40);
        task.node_id = "sched.tx".into();
        task.align = TextAlign::Left;
        task.wrap = true;
        task.x_emu = 1_504_800; // +24pt gutter
        task.y_emu = 1_400_000;
        task.cx_emu = 3_295_200;
        task.cy_emu = 180_000;
        task.runs = vec![dummy_run("07:00 AM · Morning stretch", 18)];
        let mut elements = vec![
            PageElement::Shape(card),
            PageElement::TextBox(title),
            PageElement::TextBox(task),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 6_858_000);
        let got = elements[0].textbox().expect("folded");
        assert_eq!(got.para_align, vec![TextAlign::Left, TextAlign::Left]);
        let blob: String = got.runs.iter().map(|r| r.text.as_str()).collect();
        assert!(
            blob.contains('\u{00A0}') && blob.contains("07:00"),
            "gutter column must keep NBSP pad, got {blob:?}"
        );
    }

    #[test]
    fn nested_checkbox_stays_above_folded_card() {
        let mut card = paper_shape(10);
        card.node_id = "sched".into();
        card.behind_doc = false;
        card.x_emu = 1_000_000;
        card.y_emu = 1_000_000;
        card.cx_emu = 4_000_000;
        card.cy_emu = 2_000_000;
        card.fill_hex = Some("F7F0E8".into());
        let mut title = title_box(20);
        title.node_id = "sched.title".into();
        title.align = TextAlign::Left;
        title.x_emu = 1_200_000;
        title.y_emu = 1_100_000;
        title.cx_emu = 3_600_000;
        title.cy_emu = 200_000;
        title.runs = vec![dummy_run("TODAY'S SCHEDULE", 18)];
        let mut cb = paper_shape(15);
        cb.node_id = "sched.cb".into();
        cb.behind_doc = false;
        cb.fill_hex = None;
        cb.line_hex = Some("3F3F46".into());
        cb.line_w_emu = 12_700;
        cb.corner_emu = 50_800;
        cb.x_emu = 1_200_000;
        cb.y_emu = 1_420_000;
        cb.cx_emu = 203_200; // 16pt square
        cb.cy_emu = 203_200;
        let mut task = title_box(40);
        task.node_id = "sched.tx".into();
        task.align = TextAlign::Left;
        task.wrap = true;
        task.x_emu = 1_504_800;
        task.y_emu = 1_400_000;
        task.cx_emu = 3_295_200;
        task.cy_emu = 180_000;
        task.runs = vec![dummy_run("07:00 AM · Morning stretch", 18)];
        let mut elements = vec![
            PageElement::Shape(card),
            PageElement::TextBox(title),
            PageElement::Shape(cb),
            PageElement::TextBox(task),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 6_858_000);
        demote_outline_frames_below_text(&mut elements);
        let card_rel = elements
            .iter()
            .find_map(|el| match el {
                PageElement::TextBox(tb) if tb.node_id == "sched" => Some(tb.relative_height),
                _ => None,
            })
            .expect("folded card");
        let cb_rel = elements
            .iter()
            .find_map(|el| match el {
                PageElement::Shape(s) if s.node_id == "sched.cb" => Some(s.relative_height),
                _ => None,
            })
            .expect("checkbox must remain a drawing");
        assert!(
            cb_rel > card_rel,
            "checkbox must sit above the folded fill, cb={cb_rel} card={card_rel}"
        );
    }

    #[test]
    fn fold_keeps_nested_chip_fill_and_aabb() {
        // Same-node pill (absorb already copied tag_bg onto the text box)
        // inside a white card with body copy. Fold must keep the chip.
        let mut card = paper_shape(10);
        card.node_id = "mod.box".into();
        card.behind_doc = false;
        card.x_emu = 1_000_000;
        card.y_emu = 1_000_000;
        card.cx_emu = 4_000_000;
        card.cy_emu = 3_000_000;
        card.fill_hex = Some("FFFFFF".into());
        let mut body = title_box(20);
        body.node_id = "mod.lead".into();
        body.fill_hex = Some("FFFFFE".into());
        body.align = TextAlign::Left;
        body.x_emu = 1_200_000;
        body.y_emu = 1_200_000;
        body.cx_emu = 3_600_000;
        body.cy_emu = 400_000;
        body.runs = vec![TextRun {
            text: "Enterprise Core Modernization".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 20,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "1A2332".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut chip = title_box(15);
        chip.node_id = "mod.tag".into();
        chip.fill_hex = Some("E8F1FA".into());
        chip.corner_emu = 50_800;
        chip.align = TextAlign::Center;
        chip.x_emu = 1_200_000;
        chip.y_emu = 3_500_000;
        chip.cx_emu = 1_400_000;
        chip.cy_emu = 180_000;
        chip.runs = vec![TextRun {
            text: "PRIORITY: TIER-1 STRATEGIC".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 14,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "0F2A4A".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut elements = vec![
            PageElement::Shape(card),
            PageElement::TextBox(body),
            PageElement::TextBox(chip),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        let chip = elements
            .iter()
            .find_map(|el| match el {
                PageElement::TextBox(tb) if tb.node_id == "mod.tag" => Some(tb),
                _ => None,
            })
            .expect("chip must stay a sibling");
        assert_eq!(chip.fill_hex.as_deref(), Some("E8F1FA"));
        assert_eq!(chip.x_emu, 1_200_000);
        assert_eq!(chip.cx_emu, 1_400_000);
        assert_eq!(chip.align, TextAlign::Center);
        let card = elements
            .iter()
            .find_map(|el| match el {
                PageElement::TextBox(tb) if tb.node_id == "mod.box" => Some(tb),
                _ => None,
            })
            .expect("card still folds body");
        assert!(
            chip.relative_height > card.relative_height,
            "chip must sit above the folded card, chip={} card={}",
            chip.relative_height,
            card.relative_height
        );
        assert_eq!(card.fill_hex.as_deref(), Some("FFFFFF"));
        let text: String = card
            .runs
            .iter()
            .map(|r| r.text.replace('\u{00A0}', " "))
            .collect();
        assert!(text.contains("Enterprise Core Modernization"));
        assert!(!text.contains("PRIORITY"), "chip label must not merge into the card");
    }

    #[test]
    fn fold_upgrades_glyph_tight_centered_title() {
        // Parent align_items:center shrink-wraps the title. Infer says Left
        // inside that tight box; after fold it must still center vs the body.
        let mut card = paper_shape(10);
        card.node_id = "notice".into();
        card.behind_doc = false;
        card.x_emu = 508_000;
        card.y_emu = 5_400_000;
        card.cx_emu = 6_540_500;
        card.cy_emu = 760_000;
        card.fill_hex = Some("E2E8F0".into());
        let mut title = title_box(20);
        title.node_id = "notice.title".into();
        title.align = TextAlign::Left;
        title.x_emu = 1_856_500;
        title.y_emu = 5_554_000;
        title.cx_emu = 3_843_400;
        title.cy_emu = 127_000;
        title.runs = vec![TextRun {
            text: "CONFIDENTIALITY NOTICE".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 16,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "1E293B".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut body = title_box(30);
        body.node_id = "notice.body".into();
        body.align = TextAlign::Left;
        body.wrap = true;
        body.x_emu = 750_100;
        body.y_emu = 5_732_000;
        body.cx_emu = 6_056_300;
        body.cy_emu = 276_000;
        body.runs = vec![TextRun {
            text: "All evaluations are confidential.".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 16,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "1E293B".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut elements = vec![
            PageElement::Shape(card),
            PageElement::TextBox(title),
            PageElement::TextBox(body),
        ];
        fold_shell_label_stacks(&mut elements, 7_556_500, 10_693_400);
        let got = elements[0].textbox().expect("folded");
        assert_eq!(
            got.para_align,
            vec![TextAlign::Center, TextAlign::Left],
            "shrink-wrapped title must center in the shell, got {:?}",
            got.para_align
        );
        assert_eq!(got.align, TextAlign::Left, "widest body stays left");
    }

    fn dummy_run(text: &str, half_pt: i32) -> TextRun {
        TextRun {
            text: text.into(),
            font_family_key: String::new(),
            font_name: "Georgia".into(),
            sz_half_points: half_pt,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "0F172A".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }
    }

    #[test]
    fn fold_flattens_pinned_callout_wrap_so_the_is_not_orphaned() {
        // Lock wrapped after "the"; pin inserts `\n` + NBSP. Folding into a
        // wrap=square card must drop that break so host reflow keeps
        // "the formal" together instead of leaving "the" on its own row.
        let mut card = paper_shape(10);
        card.node_id = "p1.recital_callout".into();
        card.behind_doc = false;
        card.x_emu = 609_600;
        card.y_emu = 8_343_201;
        card.cx_emu = 6_337_300;
        card.cy_emu = 626_427;
        card.fill_hex = Some("FFFFFF".into());
        let mut title = title_box(20);
        title.node_id = "p1.rc.title".into();
        title.x_emu = 736_600;
        title.y_emu = 8_444_801;
        title.cx_emu = 6_083_300;
        title.cy_emu = 134_937;
        title.runs = vec![dummy_run("PRELIMINARY BILATERAL RECITAL", 16)];
        let mut body = title_box(30);
        body.node_id = "p1.rc.body".into();
        body.wrap = false;
        body.line_twips = Some(207);
        body.last_line_twips = Some(150);
        body.x_emu = 736_600;
        body.y_emu = 8_605_139;
        body.cx_emu = 6_083_300;
        body.cy_emu = 262_890;
        body.runs = vec![
            dummy_run(
                "The\u{00A0}Parties\u{00A0}confirm\u{00A0}that\u{00A0}all\u{00A0}commercial\u{00A0}thresholds\u{00A0}set\u{00A0}forth\u{00A0}in\u{00A0}this\u{00A0}Article\u{00A0}II\u{00A0}constitute\u{00A0}binding\u{00A0}executive\u{00A0}commitments\u{00A0}subject\u{00A0}to\u{00A0}the ",
                15,
            ),
            dummy_run("\n", 15),
            dummy_run(
                "formal\u{00A0}operational\u{00A0}steering\u{00A0}and\u{00A0}dispute\u{00A0}resolution\u{00A0}protocols.",
                15,
            ),
        ];
        assert!(
            super::fold_label_is_wrapping_body(&body),
            "callout body must take the wrapping-body fold path"
        );
        let mut elements = vec![
            PageElement::Shape(card),
            PageElement::TextBox(title),
            PageElement::TextBox(body),
        ];
        fold_shell_label_stacks(&mut elements, 7_556_500, 10_693_400);
        let got = elements[0].textbox().expect("folded");
        let blob: String = got.runs.iter().map(|r| r.text.as_str()).collect();
        assert!(
            blob.contains("the formal"),
            "pinned wrap after 'the' must flatten, got {blob:?}"
        );
        let body = blob.split('\n').nth(1).unwrap_or("");
        assert!(
            body.contains("the formal") && !body.contains('\u{00A0}'),
            "body must reflow with ordinary spaces, got {body:?}"
        );
        assert_eq!(
            blob.matches('\n').count(),
            1,
            "only the title/body split should remain, got {blob:?}"
        );
        assert_eq!(
            got.para_align.len(),
            2,
            "title + one body para, got {:?}",
            got.para_align
        );
    }

    #[test]
    fn fold_stack_pitch_ignores_multiline_box_height() {
        // Soft-wrapped body is one para whose cy spans N lines. Using cy as
        // pitch would ~N× the leading (callout quote blow-up). wrap=square
        // is the real host_wrap path; wrap=none one-liners use box height.
        let mut body = title_box(10);
        body.wrap = true;
        body.line_twips = Some(189);
        body.last_line_twips = Some(140);
        body.cy_emu = 360_000; // ~3× a one-line box
        body.runs = vec![TextRun {
            text: "Soft wrapped body without hard breaks".into(),
            font_family_key: String::new(),
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
            font_family_key: String::new(),
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
            font_family_key: String::new(),
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
            font_family_key: String::new(),
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
            font_family_key: String::new(),
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

    fn quote_run(text: &str, sz: i32) -> TextRun {
        TextRun {
            text: text.into(),
            font_family_key: String::new(),
            font_name: "Georgia".into(),
            sz_half_points: sz,
            bold: false,
            italic: true,
            underline: false,
            strike: false,
            color_hex: "231F1D".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }
    }

    #[test]
    fn fold_quote_card_unwraps_nbsp_and_stays_top_aligned() {
        let mut card = paper_shape(10);
        card.node_id = "quote.card".into();
        card.behind_doc = false;
        card.x_emu = 1_000_000;
        card.y_emu = 1_000_000;
        card.cx_emu = 6_000_000;
        card.cy_emu = 800_000;
        card.fill_hex = Some("F2EBE0".into());
        let mut quote = title_box(20);
        quote.node_id = "quote.text".into();
        quote.wrap = false;
        quote.x_emu = 1_150_000;
        quote.y_emu = 1_080_000;
        quote.cx_emu = 5_700_000;
        quote.cy_emu = 450_000;
        quote.t_ins_emu = 20_000;
        quote.line_twips = Some(220);
        quote.runs = vec![quote_run(
            "The noble harmony of a printed folio ariseth not from superfluous ornament, but from the quiet breath between letter and unprinted vellum.",
            20,
        )];
        quote.runs[0].text = quote.runs[0].text.replace(' ', "\u{00A0}");
        let mut attr = title_box(30);
        attr.node_id = "quote.attr".into();
        attr.wrap = false;
        attr.x_emu = 1_150_000;
        attr.y_emu = 1_560_000;
        attr.cx_emu = 5_700_000;
        attr.cy_emu = 160_000;
        attr.runs = vec![quote_run("— Aldus Manutius, Venice, 1499", 16)];
        let mut elements = vec![
            PageElement::Shape(card),
            PageElement::TextBox(quote),
            PageElement::TextBox(attr),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        let got = elements[0].textbox().expect("folded quote");
        assert!(got.wrap);
        assert!(!got.vert_center, "quote cards stay top-aligned, got ctr");
        let blob: String = got.runs.iter().map(|r| r.text.as_str()).collect();
        let quote_part = blob.split('\n').next().unwrap_or(&blob);
        assert!(
            quote_part.contains(' ') && !quote_part.contains('\u{00A0}'),
            "folded quote must host-wrap, got {blob:?}"
        );
        assert!(blob.contains("Aldus"), "{blob:?}");
        assert_eq!(got.para_line_twips[0], Some(220));
    }

    #[test]
    fn fold_single_quote_is_not_vert_centered() {
        let mut card = paper_shape(10);
        card.node_id = "pull.card".into();
        card.behind_doc = false;
        card.x_emu = 1_000_000;
        card.y_emu = 1_000_000;
        card.cx_emu = 6_000_000;
        card.cy_emu = 700_000;
        card.fill_hex = Some("F2EBE0".into());
        let mut quote = title_box(20);
        quote.node_id = "pull.text".into();
        quote.wrap = false;
        quote.x_emu = 1_150_000;
        quote.y_emu = 1_100_000;
        quote.cx_emu = 5_700_000;
        quote.cy_emu = 500_000;
        quote.t_ins_emu = 25_000;
        quote.line_twips = Some(240);
        quote.runs = vec![quote_run(
            "The book designer must endeavor to achieve complete visual serenity. Every typographical decision should serve the reader.",
            20,
        )];
        quote.runs[0].text = quote.runs[0].text.replace(' ', "\u{00A0}");
        let mut elements = vec![PageElement::Shape(card), PageElement::TextBox(quote)];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        let got = elements[0].textbox().expect("folded pull");
        assert!(got.wrap);
        assert!(!got.vert_center);
        assert!(got.t_ins_emu > 0, "keep lock top pad, got {}", got.t_ins_emu);
        let blob: String = got.runs.iter().map(|r| r.text.as_str()).collect();
        assert!(blob.contains(' ') && !blob.contains('\u{00A0}'), "{blob:?}");
        assert_eq!(got.line_twips, Some(240));
    }

    #[test]
    fn fold_centered_wrapping_body_keeps_lock_top_pad() {
        // White card: header chip is skipped; wrapping body is lock-centered
        // in the remaining box. Expanding to the shell cannot use anchor=ctr
        // (that would center in the whole card). tIns must include the body's
        // origin *and* its inner first-line pad.
        let mut card = paper_shape(10);
        card.node_id = "doc.cards.c1".into();
        card.behind_doc = false;
        card.x_emu = 1_000_000;
        card.y_emu = 1_000_000;
        card.cx_emu = 3_000_000;
        card.cy_emu = 1_200_000;
        card.fill_hex = Some("FFFFFF".into());
        let mut body = title_box(20);
        body.node_id = "doc.cards.c1.b".into();
        body.wrap = true;
        body.vert_center = true;
        body.x_emu = 1_000_000;
        body.y_emu = 1_250_000;
        body.cx_emu = 3_000_000;
        body.cy_emu = 950_000;
        body.t_ins_emu = 80_000;
        body.line_twips = Some(220);
        body.runs = vec![quote_run(
            "Art Direction and Studio Styling Sprint v2.4 Packaging Color Swatches and Web Assets extra copy",
            17,
        )];
        let mut elements = vec![PageElement::Shape(card), PageElement::TextBox(body)];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        let got = elements[0].textbox().expect("folded card body");
        assert!(got.wrap);
        assert!(!got.vert_center, "taller shell must stay top-aligned, got ctr");
        assert_eq!(
            got.t_ins_emu, 330_000,
            "body origin 250k + inner pad 80k, got {}",
            got.t_ins_emu
        );
    }

    #[test]
    fn fold_same_aabb_wrapping_cell_keeps_vert_center() {
        let mut cell = paper_shape(10);
        cell.node_id = "doc.t.r0c2".into();
        cell.behind_doc = false;
        cell.x_emu = 1_000_000;
        cell.y_emu = 1_000_000;
        cell.cx_emu = 2_000_000;
        cell.cy_emu = 500_000;
        cell.fill_hex = Some("FFFFFF".into());
        let mut body = title_box(20);
        body.node_id = "doc.t.r0c2".into();
        body.wrap = true;
        body.vert_center = true;
        body.x_emu = 1_000_000;
        body.y_emu = 1_000_000;
        body.cx_emu = 2_000_000;
        body.cy_emu = 500_000;
        body.t_ins_emu = 50_000;
        body.line_twips = Some(200);
        body.runs = vec![quote_run(
            "Art Direction, Primary Palette and Typography System for the spring collection",
            17,
        )];
        let mut elements = vec![PageElement::Shape(cell), PageElement::TextBox(body)];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        let got = elements[0].textbox().expect("folded cell");
        assert!(got.vert_center, "same-AABB wrapping cell must stay centered");
        assert_eq!(got.t_ins_emu, 50_000, "keep lock first-line pad for Writer tIns");
    }

    #[test]
    fn fold_stack_pitch_uses_padded_nowrap_box_height() {
        // Note-line / underline cells: one wrap=none para in a padded box
        // (~2.7× face). Face-only pitch stacked prompts and left the ruled
        // lines in the leftover card.
        let mut line = title_box(10);
        line.wrap = false;
        line.line_twips = Some(190);
        line.last_line_twips = Some(190);
        line.cy_emu = 327_025; // ~25.75pt cell, 9.5pt face
        line.runs = vec![TextRun {
            text: "Today I am genuinely grateful for:".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 19,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "685141".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        assert_eq!(
            super::label_stack_pitch(&line),
            crate::coord::emu_to_twips(327_025)
        );
        // Wide leftover one-liners wrap=square with no lock leading.
        line.wrap = true;
        line.line_twips = None;
        line.last_line_twips = None;
        assert_eq!(
            super::label_stack_pitch(&line),
            crate::coord::emu_to_twips(327_025),
            "wrap=square padded one-liner must still use box height"
        );
    }

    #[test]
    fn fold_stack_note_line_splits_face_pitch_and_vertical_pad() {
        let mut line = title_box(10);
        line.wrap = false;
        line.line_twips = Some(190);
        line.last_line_twips = Some(190);
        line.cy_emu = 327_025;
        line.runs = vec![TextRun {
            text: "Today I am genuinely grateful for:".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 19,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "685141".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let cell = crate::coord::emu_to_twips(327_025);
        let face = 190;
        let vpad = (cell - face).max(0) / 2;
        line.vert_center = true;
        let (para_line, para_after, para_before) = super::fold_stack_spacing(&[&line]);
        assert_eq!(para_line, vec![Some(face)]);
        assert_eq!(para_before, vec![vpad]);
        assert_eq!(para_after, vec![vpad]);
    }

    #[test]
    fn fold_stack_compact_heading_rule_keeps_gap_as_after() {
        // 14pt face in a 21.2pt box (leading + bottom pad). Exact box pitch
        // sat glyphs on the heading underline; leftover belongs after the line.
        let mut heading = title_box(10);
        heading.wrap = false;
        heading.vert_center = false;
        heading.cy_emu = 269_240;
        heading.y_emu = 0;
        heading.line_twips = None;
        heading.last_line_twips = None;
        heading.runs = vec![TextRun {
            text: "MODULE 01".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 28,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "151515".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut body = title_box(10);
        body.wrap = true;
        body.y_emu = 320_000;
        body.cy_emu = 400_000;
        body.runs = vec![quote_run("body copy", 18)];
        let face = 280;
        let pitch = crate::coord::emu_to_twips(269_240);
        let below = pitch - face;
        let gap = crate::coord::emu_to_twips(320_000 - 269_240);
        let (para_line, para_after, para_before) = super::fold_stack_spacing(&[&heading, &body]);
        assert_eq!(para_line[0], Some(face), "heading must use face pitch, got {:?}", para_line);
        assert_eq!(para_before[0], 0);
        assert_eq!(
            para_after[0],
            below + gap,
            "padding + lock gap must sit after the line, got {:?}",
            para_after
        );
    }

    #[test]
    fn fold_stack_centered_rows_put_inter_label_gap_on_before_not_after() {
        let mut a = title_box(10);
        a.wrap = false;
        a.cy_emu = 327_025;
        a.y_emu = 0;
        a.vert_center = true;
        let mut b = title_box(10);
        b.wrap = false;
        b.cy_emu = 327_025;
        b.y_emu = 400_000;
        b.vert_center = true;
        let cell = crate::coord::emu_to_twips(327_025);
        let face = super::label_line_twips(&a);
        let vpad = (cell - face).max(0) / 2;
        let gap = crate::coord::emu_to_twips(400_000 - 327_025);
        let (para_line, para_after, para_before) = super::fold_stack_spacing(&[&a, &b]);
        assert_eq!(para_line, vec![Some(face), Some(face)]);
        assert_eq!(para_before, vec![vpad, vpad + gap]);
        assert_eq!(para_after, vec![vpad, vpad]);
    }

    #[test]
    fn fold_stack_wrapped_list_items_keep_lock_leading() {
        // 2-line bullets size cy to the lock line box (~2 × leading). Pill
        // vpad `(cy − face)/2` would open ~7pt before/after each item.
        let mut a = title_box(10);
        a.bullet = true;
        a.wrap = false;
        a.line_twips = Some(229);
        a.last_line_twips = Some(170);
        a.cy_emu = 291_465;
        a.y_emu = 0;
        a.runs = vec![TextRun {
            text: "• Zero-copy shared memory".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 17,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "E2E8F0".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut b = a.clone();
        b.node_id = "item2".into();
        b.y_emu = 291_465;
        let (para_line, para_after, para_before) = super::fold_stack_spacing(&[&a, &b]);
        assert_eq!(para_line, vec![Some(229), Some(229)]);
        assert_eq!(para_before, vec![0, 0]);
        assert_eq!(para_after, vec![0, 0]);
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
            font_family_key: String::new(),
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
            font_family_key: String::new(),
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
    fn fold_raster_banner_absorbs_nested_labels() {
        let mut hero = glow(10);
        hero.node_id = "doc.hero".into();
        hero.x_emu = 1_000_000;
        hero.y_emu = 1_000_000;
        hero.cx_emu = 8_000_000;
        hero.cy_emu = 1_200_000;
        hero.media_name = "raster1.png".into();
        let mut title = title_box(20);
        title.node_id = "doc.hero.title".into();
        title.x_emu = 2_500_000;
        title.y_emu = 1_200_000;
        title.cx_emu = 5_000_000;
        title.cy_emu = 400_000;
        title.runs = vec![TextRun {
            text: "WEEKLY SCHEDULE".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 52,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "64748B".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut sub = title_box(30);
        sub.node_id = "doc.hero.subtitle".into();
        sub.x_emu = 2_500_000;
        sub.y_emu = 1_650_000;
        sub.cx_emu = 5_000_000;
        sub.cy_emu = 300_000;
        sub.runs = vec![TextRun {
            text: "Studio".into(),
            font_family_key: String::new(),
            font_name: "Georgia".into(),
            sz_half_points: 26,
            bold: false,
            italic: true,
            underline: false,
            strike: false,
            color_hex: "8E9AA8".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut elements = vec![
            PageElement::Raster(hero),
            PageElement::TextBox(title),
            PageElement::TextBox(sub),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        assert_eq!(elements.len(), 1, "blur raster + labels must be one text box");
        let got = elements[0].textbox().expect("folded");
        assert_eq!(got.node_id, "doc.hero");
        assert_eq!(
            got.fill_blip.as_ref().map(|p| p.media_name.as_str()),
            Some("raster1.png")
        );
        let blob: String = got.runs.iter().map(|r| r.text.as_str()).collect();
        assert!(blob.contains("WEEKLY"), "{blob}");
        assert!(blob.contains("SCHEDULE"), "{blob}");
        assert!(blob.contains("Studio"), "{blob}");
    }

    #[test]
    fn drop_same_size_banner_fill_under_covering_raster() {
        let mut banner = paper_shape(10);
        banner.node_id = "sheet.header".into();
        banner.behind_doc = false;
        banner.x_emu = 0;
        banner.y_emu = 0;
        banner.cx_emu = 8_000_000;
        banner.cy_emu = 1_200_000;
        banner.fill_hex = Some("1A3A2A".into());
        let mut art = glow(20);
        art.node_id = "sheet.header.bg_art".into();
        art.x_emu = 0;
        art.y_emu = 0;
        art.cx_emu = 8_000_000;
        art.cy_emu = 1_200_000;
        art.media_name = "image1.png".into();
        let mut title = title_box(30);
        title.node_id = "sheet.header.title".into();
        title.x_emu = 2_000_000;
        title.y_emu = 400_000;
        title.cx_emu = 4_000_000;
        title.cy_emu = 400_000;
        title.runs = vec![TextRun {
            text: "TITLE".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 32,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "143323".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut elements = vec![
            PageElement::Shape(banner),
            PageElement::Raster(art),
            PageElement::TextBox(title),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        drop_shells_covered_by_later_media(&mut elements);
        assert!(
            !elements
                .iter()
                .any(|el| matches!(el, PageElement::Shape(s) if s.node_id == "sheet.header")),
            "same-size banner fill under covering art must drop, got {elements:?}"
        );
        let got = elements
            .iter()
            .find_map(PageElement::textbox)
            .expect("folded hero");
        assert_eq!(
            got.fill_blip.as_ref().map(|p| p.media_name.as_str()),
            Some("image1.png")
        );
        let blob: String = got.runs.iter().map(|r| r.text.as_str()).collect();
        assert!(blob.contains("TITLE"), "{blob}");
    }

    #[test]
    fn keep_card_fill_when_inner_picture_is_smaller() {
        let mut card = paper_shape(10);
        card.node_id = "tile".into();
        card.behind_doc = false;
        card.x_emu = 1_000_000;
        card.y_emu = 1_000_000;
        card.cx_emu = 3_000_000;
        card.cy_emu = 3_000_000;
        card.fill_hex = Some("FFFDF9".into());
        let mut pic = glow(20);
        pic.node_id = "tile.img".into();
        pic.x_emu = 1_100_000;
        pic.y_emu = 1_100_000;
        pic.cx_emu = 2_800_000;
        pic.cy_emu = 2_000_000;
        let mut elements = vec![PageElement::Shape(card), PageElement::Picture(pic)];
        drop_shells_covered_by_later_media(&mut elements);
        assert_eq!(
            elements.len(),
            2,
            "FIG photo smaller than the card must not drop the shell, got {elements:?}"
        );
    }

    #[test]
    fn fold_banner_keeps_side_picture_and_text_column() {
        let mut banner = paper_shape(10);
        banner.node_id = "doc.header".into();
        banner.behind_doc = false;
        banner.x_emu = 1_000_000;
        banner.y_emu = 1_000_000;
        banner.cx_emu = 8_000_000;
        banner.cy_emu = 2_000_000;
        banner.fill_hex = Some("281B15".into());
        let mut title = title_box(20);
        title.node_id = "doc.header.title".into();
        title.x_emu = 1_200_000;
        title.y_emu = 1_200_000;
        title.cx_emu = 4_000_000;
        title.cy_emu = 400_000;
        title.runs = vec![TextRun {
            text: "MINDFUL".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 36,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "FAF6F0".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut rule = paper_shape(25);
        rule.node_id = "doc.header.rule".into();
        rule.behind_doc = false;
        rule.x_emu = 1_200_000;
        rule.y_emu = 1_650_000;
        rule.cx_emu = 4_000_000;
        rule.cy_emu = 12_700;
        rule.fill_hex = Some("D7C2B2".into());
        let mut pic = glow(40);
        pic.node_id = "doc.header.photo".into();
        pic.x_emu = 6_000_000;
        pic.y_emu = 1_200_000;
        pic.cx_emu = 2_800_000;
        pic.cy_emu = 1_500_000;
        let mut elements = vec![
            PageElement::Shape(banner),
            PageElement::TextBox(title),
            PageElement::Shape(rule),
            PageElement::Picture(pic),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        assert_eq!(elements.len(), 3, "fold keeps rule + pic:pic, got {elements:?}");
        let got = elements
            .iter()
            .find_map(PageElement::textbox)
            .expect("folded banner");
        assert_eq!(got.fill_hex.as_deref(), Some("281B15"));
        assert!(
            got.r_ins_emu > 3_000_000,
            "wrap column must stay left of the photo, r_ins={}",
            got.r_ins_emu
        );
        assert!(elements.iter().any(|el| matches!(el, PageElement::Picture(_))));
        assert!(elements.iter().any(|el| matches!(el, PageElement::Shape(s) if s.node_id.ends_with(".rule"))));
    }

    #[test]
    fn fold_left_icon_banner_keeps_wide_wrap_column() {
        let mut banner = paper_shape(10);
        banner.node_id = "sec.header".into();
        banner.behind_doc = false;
        banner.x_emu = 1_000_000;
        banner.y_emu = 1_000_000;
        banner.cx_emu = 8_000_000;
        banner.cy_emu = 400_000;
        banner.fill_hex = Some("FCE7F3".into());
        let mut icon = glow(20);
        icon.node_id = "sec.bullet".into();
        icon.x_emu = 1_200_000;
        icon.y_emu = 1_100_000;
        icon.cx_emu = 200_000;
        icon.cy_emu = 200_000;
        let mut title = title_box(30);
        title.node_id = "sec.title".into();
        title.x_emu = 1_600_000;
        title.y_emu = 1_080_000;
        title.cx_emu = 2_800_000;
        title.cy_emu = 240_000;
        title.runs = vec![TextRun {
            text: "1. Strategic Overview & Objectives".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 24,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "18181B".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut elements = vec![
            PageElement::Shape(banner),
            PageElement::Picture(icon),
            PageElement::TextBox(title),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        let got = elements
            .iter()
            .find_map(PageElement::textbox)
            .expect("folded banner");
        assert_eq!(got.fill_hex.as_deref(), Some("FCE7F3"));
        assert!(
            got.r_ins_emu < 2_000_000,
            "empty right of a glyph-tight title is not a photo column, r_ins={}",
            got.r_ins_emu
        );
        let inner = got.cx_emu - got.l_ins_emu - got.r_ins_emu;
        assert!(
            inner > 4_000_000,
            "wrap column must fit the lock one-liner, inner={inner}"
        );
        let pic_rel = elements
            .iter()
            .find_map(|el| match el {
                PageElement::Picture(p) | PageElement::Raster(p) if p.node_id == "sec.bullet" => {
                    Some(p.relative_height)
                }
                _ => None,
            })
            .expect("left icon remains");
        assert!(
            pic_rel > got.relative_height,
            "icon must sit above the folded fill, pic={pic_rel} shell={}",
            got.relative_height
        );
    }

    #[test]
    fn fold_skipped_when_caption_stacked_on_picture() {
        let mut card = paper_shape(10);
        card.node_id = "tile".into();
        card.behind_doc = false;
        card.x_emu = 1_000_000;
        card.y_emu = 1_000_000;
        card.cx_emu = 3_000_000;
        card.cy_emu = 3_000_000;
        card.fill_hex = Some("FFFDF9".into());
        let mut pic = glow(20);
        pic.node_id = "tile.img".into();
        pic.x_emu = 1_100_000;
        pic.y_emu = 1_100_000;
        pic.cx_emu = 2_800_000;
        pic.cy_emu = 2_000_000;
        let mut cap = title_box(30);
        cap.node_id = "tile.cap".into();
        cap.x_emu = 1_100_000;
        cap.y_emu = 3_200_000;
        cap.cx_emu = 2_800_000;
        cap.cy_emu = 400_000;
        cap.runs = vec![TextRun {
            text: "01 / MORNING".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 16,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "725F54".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut elements = vec![
            PageElement::Shape(card),
            PageElement::Picture(pic),
            PageElement::TextBox(cap),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        assert_eq!(
            elements.len(),
            3,
            "stacked photo+caption must not fold (FIG), got {elements:?}"
        );
    }

    #[test]
    fn fold_raster_ignores_later_sibling_in_shadow_halo() {
        let mut hero = glow(10);
        hero.node_id = "doc.hero".into();
        hero.x_emu = 1_000_000;
        hero.y_emu = 1_000_000;
        hero.cx_emu = 8_000_000;
        hero.cy_emu = 1_400_000;
        hero.media_name = "raster1.png".into();
        let mut title = title_box(20);
        title.node_id = "doc.hero.title".into();
        title.x_emu = 2_500_000;
        title.y_emu = 1_200_000;
        title.cx_emu = 5_000_000;
        title.cy_emu = 400_000;
        title.runs = vec![TextRun {
            text: "TITLE".into(),
            font_family_key: String::new(),
            font_name: "Roboto".into(),
            sz_half_points: 52,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color_hex: "64748B".into(),
            hyperlink: None,
            script: crate::ir::ScriptPos::Baseline,
            tracking_twips: 0,
            field: None,
        }];
        let mut date = paper_shape(40);
        date.node_id = "doc.date_box".into();
        date.behind_doc = false;
        date.x_emu = 1_000_000;
        date.y_emu = 2_200_000;
        date.cx_emu = 8_000_000;
        date.cy_emu = 500_000;
        date.fill_hex = Some("FFFFFF".into());
        let mut elements = vec![
            PageElement::Raster(hero),
            PageElement::TextBox(title),
            PageElement::Shape(date),
        ];
        fold_shell_label_stacks(&mut elements, 12_240_000, 15_840_000);
        assert!(
            elements.iter().any(|el| el.textbox().is_some_and(|t| t.fill_blip.is_some())),
            "halo overlap with a later card must not block the title fold, got {elements:?}"
        );
        assert!(elements.iter().any(|el| matches!(el, PageElement::Shape(s) if s.node_id == "doc.date_box")));
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
