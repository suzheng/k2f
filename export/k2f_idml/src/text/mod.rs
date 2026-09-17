mod font;
mod lists;
mod runs;
mod tracking;

use crate::align::{
    autosize_reference, body_lines, has_full_width_lock_line, infer_text_align_for, line_gaps,
    lock_break_char_indices, lock_ink_height, should_autosize_width,
    should_autosize_width_for_nobreak, should_pin_lock_breaks, source_glyphs, source_lines,
};
use crate::coord::millipt_to_pt;
use crate::ir::{TextAlign, TextBox, TextRun};
use font::FontCtx;
use k2f_core::{
    GeometryNode, ListMarkerType, Modifier, NodeContent, Rect, SemanticNode, TextGlyphRun,
};
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
    page_tokens: Option<(usize, usize)>,
) -> Option<TextBox> {
    if node.role == "math" || matches!(node.content, NodeContent::Math(_)) {
        return None;
    }
    let raw = k2f_core::node_text(node)?;
    if raw.is_empty() {
        return None;
    }
    let mut runs = runs::runs_from_paint(raw, paint_runs, &node.modifiers, geo, fonts);
    if let Some((total, current)) = page_tokens {
        runs = lists::expand_page_tokens(runs, total, current);
    }
    if runs.is_empty() {
        return None;
    }
    let align = geo
        .map(|g| infer_text_align_for(g, raw, &node.modifiers))
        .unwrap_or(TextAlign::Left);
    let font_size = paint_runs
        .iter()
        .map(|r| r.style.font_size)
        .max_by_key(|p| p.0.abs())
        .unwrap_or(k2f_core::Pt(12_000));
    let nlines = geo
        .map(|g| body_lines(g, &node.modifiers).len())
        .unwrap_or(0);
    let numbered = node.marker_type == Some(ListMarkerType::Number);
    let bullet =
        !numbered && (node.role == "list_item" || node.marker_type == Some(ListMarkerType::Bullet));
    let is_list = bullet || numbered;
    // Literal markers change first-line width; lock wrap points and NoBreak
    // would overset or double-indent. Lists reflow with hanging indent.
    let pin = !is_list && should_pin_lock_breaks(geo, font_size, Some(raw), align, &node.modifiers);
    if pin {
        if let Some(g) = geo {
            insert_lock_breaks(
                &mut runs,
                &lock_break_char_indices(g, raw, &node.modifiers),
            );
        }
    }
    if bullet {
        lists::prepend_literal_bullet(&mut runs);
    } else if numbered {
        lists::prepend_literal_number(&mut runs, list_start.max(1));
    }
    // Tight 2-line display titles keep NoBreak + NBSP (PPTX wrap=none analog).
    // Column wrap must reflow in InDesign: pinning those lines as `<Br/>` and
    // gluing spaces double-wraps when the host frame is a different width.
    let host_reflow = nlines >= 3 || (nlines >= 2 && geo.is_some_and(has_full_width_lock_line));
    let no_break = !is_list && !matches!(align, TextAlign::Justify) && !host_reflow;
    if no_break {
        glue_lock_line_spaces(&mut runs);
    }
    let leading = leading_pt(geo, &node.modifiers);
    for run in &mut runs {
        run.leading_pt = leading;
    }
    let (mut inset_top, mut inset_left, mut inset_bottom, inset_right) = insets(geo, align);
    let mut first_line_indent_pt = infer_first_line_indent(geo, align);
    let mut left_indent_pt = 0.0;
    if is_list {
        // Outer pad → frame inset; marker column → LeftIndent hanging.
        // The literal `•` / `{n}.` already occupies that gutter on line one.
        inset_left = lists::list_outer_pad_pt(geo);
        left_indent_pt = lists::list_hanging_pt(geo);
        first_line_indent_pt = -left_indent_pt;
    }
    // WidthOnly on wrapping multi-line frames can collapse to a narrow column.
    // NoBreak lines cannot wrap, so WidthOnly grows to the longest lock line
    // instead of oversetting. Still gated on tight ink so wide footers do not
    // shrink and re-anchor. UseNoLineBreaks is single-line only: on 2+ lock
    // lines it sizes the frame as one unwrapped paragraph and ItemTransform-
    // grows through a two-column gutter.
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
    let autosize_no_wrap = nlines <= 1;
    // Paint does not clip glyphs whose line origin sits on/past box height.
    // Skip HeightOnly when WidthOnly is on: extra wrap is already prevented.
    // Stacked column body is lock-positioned: HeightOnly shrinks when host
    // wrap uses fewer lines and inflates the gap to the next frame. Author
    // `\n` stacks (titles, affiliations) still HeightOnly so lines are not
    // clipped. Justified copy already stays lock-height.
    let mut autosize_height = nlines >= 2
        && !autosize_width
        && !matches!(align, TextAlign::Justify)
        && (raw.contains('\n') || !host_reflow);
    let mut rect = rect.clone();
    if let Some(ink) = lock_ink_height(geo, font_size) {
        if ink.0 > rect.height.0 {
            rect.height = ink;
        }
    }
    let vert_center = vert_center(geo, &rect, font_size)
        || symmetric_vertical_padding(inset_top, inset_bottom);
    if vert_center {
        inset_top = 0.0;
        inset_bottom = 0.0;
        // HeightOnly shrinks the frame to ink height; CenterAlign needs the
        // full lock rect to vertically center multi-line body in a card.
        autosize_height = false;
    }
    Some(TextBox {
        node_id: node.id.clone(),
        rect,
        runs,
        align,
        inset_top,
        inset_left,
        inset_bottom,
        inset_right,
        first_line_indent_pt,
        left_indent_pt,
        vert_center,
        autosize_width,
        autosize_refer,
        autosize_no_wrap,
        autosize_height,
        no_break,
        semantic_newlines: raw.contains('\n'),
        lock_line_count: nlines,
        // K2F paint puts the first baseline at font ascent when y_offset is 0
        // (`rect.y + ascent`). LeadingOffset would sit at `Leading` instead
        // and add (leading − ascent) of air at the top of every multi-line
        // frame. Ascent matches paint.
        first_baseline_leading_offset: false,
        full_width_lock_line: geo.is_some_and(has_full_width_lock_line),
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
    let leading = leading_pt(geo, &node.modifiers);
    for r in &mut runs {
        r.leading_pt = leading;
    }
    runs
}

pub(crate) fn insets(geo: Option<&GeometryNode>, align: TextAlign) -> (f64, f64, f64, f64) {
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

/// Extra indent of the first lock line vs later lines (Left only).
/// Common left padding stays on the frame (`inset_left`); this is `FirstLineIndent`.
pub(crate) fn infer_first_line_indent(geo: Option<&GeometryNode>, align: TextAlign) -> f64 {
    if !matches!(align, TextAlign::Left) {
        return 0.0;
    }
    let Some(geo) = geo else {
        return 0.0;
    };
    let lines = source_lines(geo);
    if lines.len() < 2 {
        return 0.0;
    }
    let first = lines[0]
        .iter()
        .map(|g| g.x_offset.0)
        .min()
        .unwrap_or(0);
    let rest = lines[1..]
        .iter()
        .flat_map(|line| line.iter().map(|g| g.x_offset.0))
        .min()
        .unwrap_or(first);
    millipt_to_pt((first - rest).max(0))
}

pub(crate) fn leading_pt(geo: Option<&GeometryNode>, modifiers: &[Modifier]) -> Option<f64> {
    let geo = geo?;
    let lines = body_lines(geo, modifiers);
    if lines.len() < 2 {
        return None;
    }
    let mut ys: Vec<i128> = lines
        .iter()
        .filter_map(|line| line.iter().map(|g| g.y_offset.0).min())
        .collect();
    if ys.len() < 2 {
        return None;
    }
    ys.sort_unstable();
    let mut gaps: Vec<i128> = ys
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .filter(|g| *g > 0)
        .collect();
    if gaps.is_empty() {
        return None;
    }
    gaps.sort_unstable();
    Some(millipt_to_pt(gaps[gaps.len() / 2]))
}

/// Lock geometry with equal top/bottom glyph slack is vertically centered in
/// K2F even when the first-line y/h ratio is below the optical-center band.
pub(crate) fn symmetric_vertical_padding(top_pt: f64, bottom_pt: f64) -> bool {
    const MIN_PT: f64 = 2.0;
    if top_pt < MIN_PT || bottom_pt < MIN_PT {
        return false;
    }
    let delta = (top_pt - bottom_pt).abs();
    let slack = top_pt + bottom_pt;
    delta * 10.0 <= slack.max(MIN_PT * 2.0)
}

pub(crate) fn vert_center(
    geo: Option<&GeometryNode>,
    rect: &Rect,
    font_size: k2f_core::Pt,
) -> bool {
    let Some(geo) = geo else {
        return false;
    };
    let lines = source_lines(geo);
    if lines.is_empty() {
        return false;
    }
    let h = geo.height.0;
    if h <= 0 {
        return false;
    }
    let first_y = lines[0]
        .iter()
        .map(|g| g.y_offset.0)
        .min()
        .unwrap_or(0)
        .max(0);
    // Shrink-wrapped pill/badge: equal top/bottom padding around one line.
    // First-glyph y/h ratio stays ~0.2–0.3 (ascent offset), so the old
    // 0.4–0.6 optical-center band misses VIP labels and time pills.
    // Grid-column date/meta lines fill their column width (zero horizontal
    // slack) but can match the symmetric-height formula — keep them top-aligned.
    if lines.len() == 1 {
        let (h_slack, _, _) = line_gaps(&lines[0], geo.width.0);
        if h_slack >= 2_000 {
            let naive_bottom = h.saturating_sub(first_y).saturating_sub(font_size.0.max(0));
            // Shrink-wrapped pills use equal top/bottom padding; line_height_mult >
            // 1.0 makes naive_bottom > first_y even when padding is symmetric.
            let bottom = if naive_bottom > first_y
                && first_y >= 1_000
                && h <= first_y * 2 + font_size.0 + 2_000
            {
                first_y
            } else {
                naive_bottom
            };
            let slack = first_y.saturating_add(bottom);
            let delta = first_y - bottom;
            if slack >= 2_000 && delta.abs() * 5 <= slack && slack * 10 > h {
                return true;
            }
            // Padded bars/badges: theme top/bottom padding match first_y; line
            // height is the remainder (line_height_mult > 1 still fits).
            if first_y >= 1_000 {
                let line_h = h.saturating_sub(first_y.saturating_mul(2));
                let max_h = (first_y + font_size.0).saturating_mul(2).saturating_add(2_000);
                if line_h >= font_size.0.saturating_mul(8) / 10 && h <= max_h {
                    return true;
                }
            }
        }
    }
    if rect.height.0 <= font_size.0.saturating_mul(2) {
        return false;
    }
    let ratio = first_y as f64 / h as f64;
    (0.4..=0.6).contains(&ratio)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::{GlyphPosition, Pt};

    fn glyph(cluster: u32, x_off: i128, x_adv: i128, y: i128) -> GlyphPosition {
        GlyphPosition {
            glyph_id: 1,
            cluster,
            x_offset: Pt(x_off),
            y_offset: Pt(y),
            x_advance: Pt(x_adv),
            y_advance: Pt(0),
        }
    }

    fn pill_geo(height: i128, y: i128) -> GeometryNode {
        GeometryNode {
            id: "badge".into(),
            x: Pt(0),
            y: Pt(0),
            width: Pt(80_000),
            height: Pt(height),
            glyphs: vec![glyph(0, 5_000, 50_000, y)],
            text_runs: vec![],
            fill_rects: vec![],
            children: vec![],
        }
    }

    #[test]
    fn symmetric_pill_with_line_height_mult_centers() {
        // badge role: padding 2000 + line_height 8050 (7pt * 1.15) + padding 2000
        let geo = pill_geo(12_050, 2_000);
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(80_000),
            height: Pt(12_050),
        };
        assert!(
            vert_center(Some(&geo), &rect, Pt(7_000)),
            "line_height_mult > 1 must not block pill centering"
        );
    }

    #[test]
    fn tall_single_line_frame_does_not_center() {
        let geo = pill_geo(30_000, 2_000);
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(80_000),
            height: Pt(30_000),
        };
        assert!(
            !vert_center(Some(&geo), &rect, Pt(7_000)),
            "tall frame is not a shrink-wrapped pill"
        );
    }

    #[test]
    fn padded_bar_with_symmetric_theme_padding_centers() {
        let geo = pill_geo(21_500, 4_000);
        let rect = Rect {
            x: Pt(36_000),
            y: Pt(133_850),
            width: Pt(213_018),
            height: Pt(21_500),
        };
        assert!(
            vert_center(Some(&geo), &rect, Pt(10_000)),
            "symmetric padded bar must vertically center"
        );
    }

    #[test]
    fn flush_left_column_line_stays_top_aligned() {
        let geo = GeometryNode {
            id: "pd.yrs".into(),
            x: Pt(0),
            y: Pt(0),
            width: Pt(144_333),
            height: Pt(10_800),
            glyphs: vec![glyph(0, 0, 80_000, 0)],
            text_runs: vec![],
            fill_rects: vec![],
            children: vec![],
        };
        let rect = Rect {
            x: Pt(225_334),
            y: Pt(553_304),
            width: Pt(144_333),
            height: Pt(10_800),
        };
        assert!(
            !vert_center(Some(&geo), &rect, Pt(8_000)),
            "flush-top column label must not vertically center"
        );
    }

    #[test]
    fn grid_column_meta_with_symmetric_height_stays_top_aligned() {
        let geo = GeometryNode {
            id: "app_harvard.meta".into(),
            x: Pt(0),
            y: Pt(0),
            width: Pt(112_358),
            height: Pt(22_120),
            glyphs: vec![glyph(0, 0, 112_358, 6_380)],
            text_runs: vec![],
            fill_rects: vec![],
            children: vec![],
        };
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(112_358),
            height: Pt(22_120),
        };
        assert!(
            !vert_center(Some(&geo), &rect, Pt(7_800)),
            "full-width grid meta must not vertically center"
        );
    }

    fn line_geo(ys: &[i128]) -> GeometryNode {
        GeometryNode {
            id: "body".into(),
            x: Pt(0),
            y: Pt(0),
            width: Pt(400_000),
            height: Pt(80_000),
            glyphs: ys
                .iter()
                .enumerate()
                .map(|(i, y)| glyph(i as u32, 0, 80_000, *y))
                .collect(),
            text_runs: vec![],
            fill_rects: vec![],
            children: vec![],
        }
    }

    #[test]
    fn symmetric_multi_line_body_block_centers() {
        // doc.body.l.mid lock geometry: 9 lines, ~64pt equal top/bottom slack.
        let ys = [
            64_212, 77_987, 91_762, 119_312, 133_087, 146_862, 174_412, 188_187, 201_962,
        ];
        let mut geo = line_geo(&ys);
        geo.height = Pt(279_950);
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(233_000),
            height: Pt(279_950),
        };
        assert!(
            !vert_center(Some(&geo), &rect, Pt(9_500)),
            "ratio band alone must not center multi-line body"
        );
        let (top, _, bottom, _) = insets(Some(&geo), TextAlign::Left);
        assert!(
            symmetric_vertical_padding(top, bottom),
            "equal top/bottom insets must detect vertical center"
        );
    }

    #[test]
    fn two_line_banner_with_symmetric_padding_centers() {
        let mut geo = line_geo(&[7_000, 18_050]);
        geo.height = Pt(36_100);
        let (top, _, bottom, _) = insets(Some(&geo), TextAlign::Left);
        assert!(
            symmetric_vertical_padding(top, bottom),
            "notice.banner style padded bar must center"
        );
    }

    #[test]
    fn flush_top_body_stays_top_aligned() {
        let ys: Vec<i128> = (0..5).map(|i| i * 14_000).collect();
        let geo = line_geo(&ys);
        let (top, _, bottom, _) = insets(Some(&geo), TextAlign::Left);
        assert!(
            !symmetric_vertical_padding(top, bottom),
            "flush-top column body must not vertically center"
        );
    }

    #[test]
    fn leading_uses_median_gap_not_first_pair() {
        // A superscript-sized first gap must not become paragraph leading.
        let geo = line_geo(&[0, 3_000, 17_000, 31_000]);
        let pt = leading_pt(Some(&geo), &[]).expect("leading");
        assert!(
            (pt - 14.0).abs() < 0.01,
            "median wrap gap is 14pt, got {pt}"
        );
    }
}
