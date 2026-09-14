mod font;
mod lists;
mod runs;

use crate::align::{infer_text_align, source_glyphs, source_lines};
use crate::coord::millipt_to_pt;
use crate::ir::{TextAlign, TextBox, TextRun};
use font::FontCtx;
use k2f_core::{GeometryNode, ListMarkerType, NodeContent, Rect, SemanticNode, TextGlyphRun};
use std::collections::BTreeMap;

pub(crate) use font::FontCtx as TextFonts;
pub(crate) use lists::list_start_at;

pub fn textbox_from_draw(
    node: &SemanticNode,
    rect: &Rect,
    paint_runs: &[TextGlyphRun],
    geo: Option<&GeometryNode>,
    fonts: &BTreeMap<String, Vec<u8>>,
) -> Option<TextBox> {
    textbox_from_draw_ctx(node, rect, paint_runs, geo, &FontCtx::new(fonts), 1, None)
}

pub(crate) fn textbox_from_draw_ctx(
    node: &SemanticNode,
    rect: &Rect,
    paint_runs: &[TextGlyphRun],
    geo: Option<&GeometryNode>,
    fonts: &FontCtx,
    list_start: u32,
    master_pages: Option<usize>,
) -> Option<TextBox> {
    if node.role == "math" || matches!(node.content, NodeContent::Math(_)) {
        return None;
    }
    let raw = k2f_core::node_text(node)?;
    if raw.is_empty() {
        return None;
    }
    let mut runs = runs::runs_from_paint(raw, paint_runs, &node.modifiers, geo, fonts);
    if let Some(total) = master_pages {
        runs = lists::expand_page_tokens(runs, total);
    }
    if runs.is_empty() {
        return None;
    }
    let align = geo
        .map(|g| infer_text_align(g, raw))
        .unwrap_or(TextAlign::Left);
    let numbered = node.marker_type == Some(ListMarkerType::Number);
    let bullet =
        !numbered && (node.role == "list_item" || node.marker_type == Some(ListMarkerType::Bullet));
    if bullet {
        lists::prepend_literal_bullet(&mut runs);
    } else if numbered {
        lists::prepend_literal_number(&mut runs, list_start.max(1));
    }
    let leading = leading_pt(geo);
    for run in &mut runs {
        run.leading_pt = leading;
    }
    let font_size = paint_runs
        .iter()
        .map(|r| r.style.font_size)
        .max_by_key(|p| p.0.abs())
        .unwrap_or(k2f_core::Pt(12_000));
    let (inset_top, inset_left, inset_bottom, inset_right) = insets(geo, align);
    Some(TextBox {
        node_id: node.id.clone(),
        rect: rect.clone(),
        runs,
        align,
        inset_top,
        inset_left,
        inset_bottom,
        inset_right,
        vert_center: vert_center(geo, rect, font_size),
    })
}

pub(crate) fn cell_runs(
    node: Option<&SemanticNode>,
    paint_runs: &[TextGlyphRun],
    geo: Option<&GeometryNode>,
    fonts: &FontCtx,
    header_bold: bool,
) -> Vec<TextRun> {
    let Some(node) = node else {
        return Vec::new();
    };
    let Some(text) = k2f_core::node_text(node) else {
        return Vec::new();
    };
    if text.is_empty() {
        return Vec::new();
    }
    let mut runs = runs::runs_from_paint(text, paint_runs, &node.modifiers, geo, fonts);
    if header_bold && paint_runs.is_empty() {
        for r in &mut runs {
            r.bold = true;
        }
    }
    let leading = leading_pt(geo);
    for r in &mut runs {
        r.leading_pt = leading;
    }
    runs
}

fn insets(geo: Option<&GeometryNode>, align: TextAlign) -> (f64, f64, f64, f64) {
    let Some(geo) = geo else {
        return (0.0, 0.0, 0.0, 0.0);
    };
    let glyphs = source_glyphs(geo);
    if glyphs.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    match align {
        TextAlign::Left => {
            let left = glyphs
                .iter()
                .map(|g| g.x_offset.0)
                .min()
                .unwrap_or(0)
                .max(0);
            (0.0, millipt_to_pt(left).max(0.0), 0.0, 0.0)
        }
        TextAlign::Right => {
            let box_w = geo.width.0;
            let right = glyphs
                .iter()
                .map(|g| box_w - (g.x_offset.0 + g.x_advance.0))
                .min()
                .unwrap_or(0)
                .max(0);
            (0.0, 0.0, 0.0, millipt_to_pt(right).max(0.0))
        }
        TextAlign::Center | TextAlign::Justify => (0.0, 0.0, 0.0, 0.0),
    }
}

pub(crate) fn leading_pt(geo: Option<&GeometryNode>) -> Option<f64> {
    let geo = geo?;
    let lines = source_lines(geo);
    if lines.len() < 2 {
        return None;
    }
    let y0 = lines[0].iter().map(|g| g.y_offset.0).min()?;
    let y1 = lines[1].iter().map(|g| g.y_offset.0).min()?;
    Some(millipt_to_pt((y1 - y0).abs()))
}

pub(crate) fn vert_center(
    geo: Option<&GeometryNode>,
    rect: &Rect,
    font_size: k2f_core::Pt,
) -> bool {
    let Some(geo) = geo else {
        return false;
    };
    if rect.height.0 <= font_size.0.saturating_mul(2) {
        return false;
    }
    let lines = source_lines(geo);
    if lines.is_empty() {
        return false;
    }
    let h = geo.height.0;
    if h <= 0 {
        return false;
    }
    let first_y = lines[0].iter().map(|g| g.y_offset.0).min().unwrap_or(0);
    let ratio = first_y as f64 / h as f64;
    (0.4..=0.6).contains(&ratio)
}
