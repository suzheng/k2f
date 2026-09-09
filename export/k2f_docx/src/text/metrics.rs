use super::align::{line_gaps, source_lines};
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
    let lines = source_lines(geo);
    if lines.is_empty() {
        return false;
    }
    let h = geo.height.0;
    if h <= 0 {
        return false;
    }
    let first_y = lines[0].iter().map(|g| g.y_offset.0).min().unwrap_or(0);
    let last_y = lines[lines.len() - 1]
        .iter()
        .map(|g| g.y_offset.0)
        .min()
        .unwrap_or(first_y);
    let fs = font_size.0.max(1);
    // `y_offset` is baseline. Subtract one face so leftover below matches
    // leftover above for a block the layout engine centered in the inner box.
    let top = first_y;
    let bot_after = (h - last_y - fs).max(0);
    gaps_look_centered(top, bot_after)
}

/// Equal leftover above the first baseline and below the last line box.
/// Catches padded table cells (~30–40% first-baseline) that the old 40–60%
/// band missed, without treating top-padded stretch as center.
fn gaps_look_centered(top: i128, bot_after: i128) -> bool {
    let lo = top.min(bot_after);
    let hi = top.max(bot_after);
    lo >= 2_000 && lo.saturating_mul(2) >= hi
}

fn emu(millipt: i128) -> i64 {
    pt_to_emu(Pt(millipt))
}

#[cfg(test)]
mod tests {
    use super::gaps_look_centered;

    #[test]
    fn padded_one_line_cell_is_centered() {
        // Crystal-clear body: 12.3pt baseline in 34pt cell, 8pt face.
        assert!(gaps_look_centered(12_312, 34_000 - 12_312 - 8_000));
    }

    #[test]
    fn two_line_cell_with_equal_padding_is_centered() {
        assert!(gaps_look_centered(7_000, 34_000 - 17_000 - 8_000));
    }

    #[test]
    fn top_padded_stretch_is_not_centered() {
        assert!(!gaps_look_centered(7_000, 34_000 - 7_000 - 8_000));
    }
}
