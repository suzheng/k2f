mod font;
mod lists;
mod runs;
mod tracking;

use crate::align::{
    autosize_reference, infer_text_align, lock_break_char_indices, lock_ink_height,
    should_autosize_width, should_autosize_width_for_nobreak, should_pin_lock_breaks,
    source_glyphs, source_lines,
};
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
    let font_size = paint_runs
        .iter()
        .map(|r| r.style.font_size)
        .max_by_key(|p| p.0.abs())
        .unwrap_or(k2f_core::Pt(12_000));
    let pin = should_pin_lock_breaks(geo, font_size, Some(raw), align);
    if pin {
        if let Some(g) = geo {
            insert_lock_breaks(&mut runs, &lock_break_char_indices(g, raw));
        }
    }
    let numbered = node.marker_type == Some(ListMarkerType::Number);
    let bullet =
        !numbered && (node.role == "list_item" || node.marker_type == Some(ListMarkerType::Bullet));
    if bullet {
        lists::prepend_literal_bullet(&mut runs);
    } else if numbered {
        lists::prepend_literal_number(&mut runs, list_start.max(1));
    }
    // Each lock line is already wrapped. Host metrics still wrap at spaces
    // and at paint-run boundaries (color splits); HeightOnly then overprints
    // the next object. NoBreak + NBSP is the IDML analog of PPTX wrap=none.
    // Applies even when semantic newlines already match lock lines (pin=false).
    let no_break = !matches!(align, TextAlign::Justify);
    if no_break {
        glue_lock_line_spaces(&mut runs);
    }
    let leading = leading_pt(geo);
    for run in &mut runs {
        run.leading_pt = leading;
    }
    let (inset_top, inset_left, inset_bottom, inset_right) = insets(geo, align);
    let nlines = geo.map(|g| source_lines(g).len()).unwrap_or(0);
    // WidthOnly on wrapping multi-line frames can collapse to a narrow column.
    // NoBreak lines cannot wrap, so WidthOnly grows to the longest lock line
    // instead of oversetting. Still gated on tight ink so wide footers do not
    // shrink and re-anchor.
    let autosize_width = if no_break {
        (nlines <= 1 && should_autosize_width(geo, align))
            || should_autosize_width_for_nobreak(geo, align)
    } else {
        nlines <= 1 && should_autosize_width(geo, align)
    };
    let autosize_refer = if no_break && matches!(align, TextAlign::Left) {
        "CenterLeftPoint"
    } else {
        autosize_reference(geo, align)
    };
    let autosize_no_wrap = nlines <= 1 || (no_break && autosize_width);
    // Paint does not clip glyphs whose line origin sits on/past box height.
    // Skip HeightOnly when WidthOnly is on: extra wrap is already prevented.
    let autosize_height = nlines >= 2 && !autosize_width;
    let mut rect = rect.clone();
    if let Some(ink) = lock_ink_height(geo, font_size) {
        if ink.0 > rect.height.0 {
            rect.height = ink;
        }
    }
    let vert_center = vert_center(geo, &rect, font_size);
    Some(TextBox {
        node_id: node.id.clone(),
        rect,
        runs,
        align,
        inset_top,
        inset_left,
        inset_bottom,
        inset_right,
        vert_center,
        autosize_width,
        autosize_refer,
        autosize_no_wrap,
        autosize_height,
        no_break,
    })
}

fn glue_lock_line_spaces(runs: &mut [TextRun]) {
    for run in runs {
        if run.auto_page_number {
            continue;
        }
        if run.text.contains(' ') {
            run.text = run.text.replace(' ', "\u{00A0}");
        }
    }
}

fn insert_lock_breaks(runs: &mut Vec<TextRun>, break_chars: &[usize]) {
    if break_chars.is_empty() || runs.is_empty() {
        return;
    }
    let full: String = runs.iter().map(|r| r.text.as_str()).collect();
    let mut bytes: Vec<usize> = break_chars
        .iter()
        .filter_map(|&ci| {
            full.char_indices()
                .nth(ci)
                .map(|(i, _)| i)
                .or_else(|| (ci >= full.chars().count()).then_some(full.len()))
        })
        .collect();
    bytes.sort_unstable();
    bytes.dedup();
    for byte in bytes.into_iter().rev() {
        insert_newline_at_byte(runs, byte);
    }
}

fn insert_newline_at_byte(runs: &mut Vec<TextRun>, byte: usize) {
    let mut pos = 0usize;
    for i in 0..runs.len() {
        let a = pos;
        let b = pos + runs[i].text.len();
        if byte < a || byte > b {
            pos = b;
            continue;
        }
        let off = byte - a;
        if off == 0 {
            if i > 0 && runs[i - 1].text.ends_with('\n') {
                return;
            }
            if runs[i].text.starts_with('\n') {
                return;
            }
            let mut nl = runs[i].clone();
            nl.text = "\n".into();
            nl.auto_page_number = false;
            runs.insert(i, nl);
            return;
        }
        if off == runs[i].text.len() {
            if runs[i].text.ends_with('\n') {
                return;
            }
            if i + 1 < runs.len() && runs[i + 1].text.starts_with('\n') {
                return;
            }
            runs[i].text.push('\n');
            return;
        }
        if !runs[i].text.is_char_boundary(off) {
            return;
        }
        let rest = runs[i].text[off..].to_string();
        runs[i].text.truncate(off);
        if !runs[i].text.ends_with('\n') {
            runs[i].text.push('\n');
        }
        if !rest.is_empty() {
            let mut tail = runs[i].clone();
            tail.text = rest;
            runs.insert(i + 1, tail);
        }
        return;
    }
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
    let top = glyphs
        .iter()
        .map(|g| g.y_offset.0)
        .min()
        .unwrap_or(0)
        .max(0);
    let top_pt = millipt_to_pt(top).max(0.0);
    let lines = source_lines(geo);
    let bottom_pt = if lines.len() >= 2 {
        let last = lines.last().unwrap()[0].y_offset.0;
        let leading = (lines[1][0].y_offset.0 - lines[0][0].y_offset.0).abs();
        millipt_to_pt((geo.height.0 - last - leading).max(0)).max(0.0)
    } else {
        0.0
    };
    match align {
        TextAlign::Left => {
            let left = glyphs
                .iter()
                .map(|g| g.x_offset.0)
                .min()
                .unwrap_or(0)
                .max(0);
            (top_pt, millipt_to_pt(left).max(0.0), bottom_pt, 0.0)
        }
        TextAlign::Right => {
            let box_w = geo.width.0;
            let right = glyphs
                .iter()
                .map(|g| box_w - (g.x_offset.0 + g.x_advance.0))
                .min()
                .unwrap_or(0)
                .max(0);
            (top_pt, 0.0, bottom_pt, millipt_to_pt(right).max(0.0))
        }
        TextAlign::Center | TextAlign::Justify => (top_pt, 0.0, bottom_pt, 0.0),
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
