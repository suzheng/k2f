mod crop;
mod detect;
mod filter;

pub use detect::{box_is_effect, is_math_node, is_rule_id};
pub use filter::{filter_chrome_ops, ChromeKeep};

use crate::ir::PictureBox;
use crate::DocxError;
use crop::{
    clamp_rect_to_page, crop_rgb_to_png, expand_rect_for_shadow, picture_from_crop, union_rect,
};
use k2f_core::{LockFile, Page, PaintOp, Rect};
use k2f_paint::{render_lockfile_page_rgb, OFFICIAL_PNG_SCALE};
use std::collections::{BTreeMap, HashSet};

pub fn rasterize_slice(
    lock: &LockFile,
    page_idx: usize,
    page: &Page,
    page_ops: &[PaintOp],
    keep: &ChromeKeep,
    crop: Rect,
    node_id: &str,
    raster_index: u32,
    relative_height: u32,
    fonts: &BTreeMap<String, Vec<u8>>,
    assets: &BTreeMap<String, Vec<u8>>,
) -> Result<PictureBox, DocxError> {
    let mut chrome = lock.clone();
    let filtered = filter_chrome_ops(page_ops, keep);
    let plan = chrome
        .render_plan
        .pages
        .iter_mut()
        .find(|p| p.index == page.index)
        .ok_or_else(|| DocxError::Write("missing render plan for raster".into()))?;
    plan.ops = filtered;
    let (w, h, rgb) =
        render_lockfile_page_rgb(&chrome, page_idx, OFFICIAL_PNG_SCALE, fonts, assets)?;
    let crop = clamp_rect_to_page(crop, page.width, page.height);
    let png = crop_rgb_to_png(&rgb, w, h, &crop, OFFICIAL_PNG_SCALE)?;
    Ok(picture_from_crop(
        node_id,
        &crop,
        png,
        raster_index,
        relative_height,
    ))
}

pub fn keep_for_effect(node_id: &str, page: &Page, math: bool) -> ChromeKeep {
    ChromeKeep {
        effect_ids: HashSet::from([node_id.to_string()]),
        math_text_ids: if math {
            HashSet::from([node_id.to_string()])
        } else {
            HashSet::new()
        },
        keep_leading_page_background: true,
        page_width: Some(page.width),
        page_height: Some(page.height),
    }
}

pub fn consume_following_box(ops: &[PaintOp], i: usize, node_id: &str) -> Option<usize> {
    match ops.get(i + 1) {
        Some(PaintOp::DrawBox { node_id: bid, .. }) if bid == node_id => Some(i + 1),
        _ => None,
    }
}

pub fn glass_crop(
    blur_rect: &Rect,
    ops: &[PaintOp],
    follow: Option<usize>,
) -> Result<Rect, DocxError> {
    let mut crop = blur_rect.clone();
    if let Some(idx) = follow {
        if let PaintOp::DrawBox {
            rect, decoration, ..
        } = &ops[idx]
        {
            crop = union_rect(crop, rect.clone());
            crop = expand_rect_for_shadow(crop, decoration)?;
        }
    }
    Ok(crop)
}

pub fn effect_box_crop(
    rect: &Rect,
    decoration: &k2f_core::BoxDecoration,
) -> Result<Rect, DocxError> {
    expand_rect_for_shadow(rect.clone(), decoration)
}

pub fn math_crop(ops: &[PaintOp], node_id: &str, text_rect: &Rect) -> Rect {
    let mut crop = text_rect.clone();
    let prefix = format!("{node_id}::rule_");
    for op in ops {
        if let PaintOp::DrawBox {
            node_id: bid, rect, ..
        } = op
        {
            if bid.starts_with(&prefix) {
                crop = union_rect(crop, rect.clone());
            }
        }
    }
    crop
}
