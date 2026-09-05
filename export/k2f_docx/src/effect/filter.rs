use super::detect::box_belongs_to_effect;
use crate::geo::is_full_page_rect;
use k2f_core::{PaintOp, Pt};
use std::collections::HashSet;

/// Which paint ops survive in a chrome lock used only to raster one effect slice.
#[derive(Clone, Debug, Default)]
pub struct ChromeKeep {
    pub effect_ids: HashSet<String>,
    pub math_text_ids: HashSet<String>,
    pub keep_leading_page_background: bool,
    pub page_width: Option<Pt>,
    pub page_height: Option<Pt>,
}

/// Drop body text, content images, native-simple boxes, and table cells from chrome ops.
pub fn filter_chrome_ops(ops: &[PaintOp], keep: &ChromeKeep) -> Vec<PaintOp> {
    ops.iter()
        .enumerate()
        .filter_map(|(i, op)| {
            if i == 0 && keep.keep_leading_page_background && is_leading_page_bg(op, keep) {
                return Some(op.clone());
            }
            match op {
                PaintOp::BackdropBlur { node_id, .. } if keep.effect_ids.contains(node_id) => {
                    Some(op.clone())
                }
                PaintOp::DrawBox { node_id, .. }
                    if box_belongs_to_effect(node_id, &keep.effect_ids) =>
                {
                    Some(op.clone())
                }
                PaintOp::DrawText { node_id, .. } if keep.math_text_ids.contains(node_id) => {
                    Some(op.clone())
                }
                _ => None,
            }
        })
        .collect()
}

fn is_leading_page_bg(op: &PaintOp, keep: &ChromeKeep) -> bool {
    let PaintOp::DrawBox { rect, .. } = op else {
        return false;
    };
    match (keep.page_width, keep.page_height) {
        (Some(w), Some(h)) => is_full_page_rect(w, h, rect),
        _ => true,
    }
}
