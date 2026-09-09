use crate::coord::{pt_to_emu, pt_to_twips};
use crate::effect::{
    box_is_effect, consume_following_box, effect_box_crop, glass_crop, is_math_node, is_rule_id,
    keep_for_effect, math_crop, rasterize_slice,
};
use crate::geo::{find_geo, page_bg_hex};
use crate::ir::{DocIR, PageElement, PageIR, PictureBox};
use crate::picture::picture_from_draw;
use crate::shape::shapes_from_box;
use crate::table::{paint_node_id, table_ref_placeholder};
use crate::text::{font_ctx, textbox_from_draw_ctx};
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
    let skip_running = crate::header::running_field_ids(running);
    let assets = doc.assets();
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
        let mut skip_ops = HashSet::new();
        let bg_hex = word_hex_color(&page_bg_hex(page, ops));
        for (i, op) in ops.iter().enumerate() {
            if skip_ops.contains(&i) {
                continue;
            }
            let rel = rel_height(i);
            if let Some(nid) = paint_node_id(op) {
                if skip_running.contains(nid) {
                    continue;
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
/// underlay matching the shape immediately behind the box (or the page paper).
///
/// Skip the paper fallback when the topmost covering layer is chrome
/// (picture, raster, gradient, glass). A full-page paper shape behind a
/// decorative SVG must not win — that white underlay hides the lock art.
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
/// hides white labels. Fold the corner radius and fill into the text box and
/// drop the duplicate fill shape.
fn absorb_self_fill_shapes(elements: &mut Vec<PageElement>) {
    let extras: BTreeMap<String, (i64, Option<String>, u8)> = elements
        .iter()
        .filter_map(|el| match el {
            PageElement::Shape(s) => Some((
                s.node_id.clone(),
                (s.corner_emu, s.fill_hex.clone(), s.fill_alpha),
            )),
            _ => None,
        })
        .collect();
    let mut absorbed = HashSet::new();
    for el in elements.iter_mut() {
        let PageElement::TextBox(tb) = el else {
            continue;
        };
        if let Some((corner, fill, alpha)) = extras.get(&tb.node_id) {
            tb.corner_emu = *corner;
            if fill.is_some() {
                tb.fill_hex = fill.clone();
                tb.fill_alpha = *alpha;
            }
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
    use crate::ir::{LineDash, PictureBox, ShapeBox, TextAlign, TextBox};

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
            line_w_emu: 0,
            line_dash: LineDash::Solid,
            behind_doc: true,
            relative_height: rel,
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
            ilvl: 0,
            l_ins_emu: 0,
            t_ins_emu: 0,
            r_ins_emu: 0,
            b_ins_emu: 0,
            line_twips: None,
            vert_center: false,
            preserve_whitespace: false,
            relative_height: rel,
            fill_hex: None,
            fill_alpha: 255,
            wrap: false,
            corner_emu: 0,
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
}
