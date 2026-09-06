use super::align::{line_gaps, source_glyphs, source_lines};
use crate::coord::{millipt_to_twips, pt_to_emu};
use crate::ir::TextAlign;
use k2f_core::{GeometryNode, Pt, Rect};

pub(crate) fn insets(geo: Option<&GeometryNode>, align: TextAlign) -> (i64, i64, i64, i64) {
    let Some(geo) = geo else {
        return (0, 0, 0, 0);
    };
    let lines = source_lines(geo);
    if lines.is_empty() {
        return (0, 0, 0, 0);
    }
    let box_w = geo.width.0;
    let min_left = lines
        .iter()
        .map(|g| line_gaps(g, box_w).1)
        .min()
        .unwrap_or(0)
        .max(0);
    let min_right = lines
        .iter()
        .map(|g| line_gaps(g, box_w).2)
        .min()
        .unwrap_or(0)
        .max(0);
    let top = first_line_top_emu(geo);
    match align {
        // Left leftover on the right is editable width, not padding. Writing it as
        // rIns shrinks the Word text frame to the glyph span and clips short words
        // in a wide box (stretching the outer shape then reveals the rest).
        // A line that already fills the padded width needs the leftover left
        // gap as host-metric slack. Keeping it as lIns clips the last glyphs.
        TextAlign::Left => {
            if line_fills_padded_width(min_left, min_right, box_w) {
                (0, top, 0, 0)
            } else {
                (emu(min_left), top, 0, 0)
            }
        }
        TextAlign::Right => {
            if line_fills_padded_width(min_right, min_left, box_w) {
                (0, top, 0, 0)
            } else {
                (0, top, emu(min_right), 0)
            }
        }
        TextAlign::Center | TextAlign::Justify => (0, top, 0, 0),
    }
}

fn line_fills_padded_width(pad: i128, opposite_slack: i128, box_w: i128) -> bool {
    let avail = box_w.saturating_sub(pad);
    if avail <= 0 {
        return false;
    }
    let content = avail.saturating_sub(opposite_slack.max(0));
    content.saturating_mul(100) >= avail.saturating_mul(85)
}

/// Lock `y_offset` on the first line includes role `padding_pt.top`. Office
/// `anchor=t` starts at the box top, so that padding must become `tIns`.
fn first_line_top_emu(geo: &GeometryNode) -> i64 {
    let Some(line) = source_lines(geo).into_iter().next() else {
        return 0;
    };
    let y = line.iter().map(|g| g.y_offset.0).min().unwrap_or(0);
    if y < 1_000 {
        return 0;
    }
    emu(y)
}

pub(crate) fn line_spacing_twips(geo: Option<&GeometryNode>) -> Option<i64> {
    let geo = geo?;
    let lines = source_lines(geo);
    if lines.len() < 2 {
        return None;
    }
    let baseline = |line: &[&k2f_core::GlyphPosition]| -> i128 {
        let mut ys: Vec<i128> = line.iter().map(|g| g.y_offset.0).collect();
        ys.sort_unstable();
        ys[ys.len() / 2]
    };
    let y0 = baseline(&lines[0]);
    let y1 = baseline(&lines[1]);
    let delta = (y1 - y0).abs();
    if delta < 2_000 {
        return None;
    }
    Some(millipt_to_twips(i64::try_from(delta).unwrap_or(0)))
}

pub(crate) fn vert_center(geo: Option<&GeometryNode>, rect: &Rect, font_size: Pt) -> bool {
    let Some(geo) = geo else {
        return false;
    };
    if rect.height.0 <= font_size.0.saturating_mul(2) {
        return false;
    }
    let Some(first) = source_glyphs(geo).into_iter().next() else {
        return false;
    };
    let h = geo.height.0;
    if h <= 0 {
        return false;
    }
    let y = first.y_offset.0;
    y * 10 >= h * 4 && y * 10 <= h * 6
}

fn emu(millipt: i128) -> i64 {
    pt_to_emu(Pt(millipt))
}
