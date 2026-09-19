use super::align::{body_font_size, line_gaps, source_glyphs, source_lines};
use crate::coord::{millipt_to_twips, pt_to_emu};
use crate::ir::TextAlign;
use k2f_core::{GeometryNode, Pt, Rect};

/// Left pad before the decorative marker (lock ink min x). Applied as bodyPr
/// `lIns` so the literal marker sits where the lock painted it.
pub(crate) fn list_outer_pad_emu(geo: Option<&GeometryNode>) -> i64 {
    let Some(geo) = geo else {
        return 0;
    };
    let ink_left = geo
        .glyphs
        .iter()
        .map(|g| g.x_offset.0)
        .min()
        .unwrap_or(0)
        .max(0);
    emu(ink_left)
}

/// Lock marker-column width for hanging indent when the marker is a literal
/// run inside the paragraph. Decorative marker glyphs use `CLUSTER_NOT_SOURCE`;
/// body source glyphs start after that column. Using body-left alone would
/// hang wrap lines by (pad + marker), while the literal `{n}.` / `•` already
/// occupies the marker slot on line one — wrap then sits too far right.
pub(crate) fn list_hanging_lock_emu(geo: Option<&GeometryNode>) -> i64 {
    let Some(geo) = geo else {
        return 0;
    };
    let ink_left = geo
        .glyphs
        .iter()
        .map(|g| g.x_offset.0)
        .min()
        .unwrap_or(0)
        .max(0);
    let body_left = source_glyphs(geo)
        .iter()
        .map(|g| g.x_offset.0)
        .min()
        .unwrap_or(ink_left)
        .max(0);
    emu((body_left - ink_left).max(0))
}

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
        // Keep lock left pad on every left-aligned box. Dropping it only when
        // the line fills ≥85% made short cells in the same column indent more
        // than filled neighbors. Overflow is already `overflow`.
        TextAlign::Left => (emu(min_left), top, 0, 0),
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

/// Native table cells have a fixed grid width, so the lock-side glyph gap is
/// always padding. The text-box ≥85%-full skip would flush numbers to the
/// cell border (and first-column labels to the left edge).
pub(crate) fn cell_h_insets_emu(geo: Option<&GeometryNode>, align: TextAlign) -> (i64, i64) {
    let Some(geo) = geo else {
        return (0, 0);
    };
    let lines = source_lines(geo);
    if lines.is_empty() {
        return (0, 0);
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
    match align {
        TextAlign::Left => (emu(min_left), 0),
        TextAlign::Right => (0, emu(min_right)),
        TextAlign::Center | TextAlign::Justify => (0, 0),
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
///
/// One-line lock boxes are often the line box (`face × line_height_mult`,
/// typically 1.2–1.5×), with slack below the face. K2F paint still uses
/// ascent from the box top. Office exact-face pitch packs glyphs to the
/// frame top and drops that air — running headers at y=0 sit on the page
/// edge. Put the line-box slack in `tIns` when the first baseline is at
/// the box top. Tall frames keep glyph `y_offset` only.
fn first_line_top_emu(geo: &GeometryNode) -> i64 {
    let lines = source_lines(geo);
    let Some(line) = lines.first() else {
        return 0;
    };
    let y = line.iter().map(|g| g.y_offset.0).min().unwrap_or(0);
    let pad = if y < 1_000 { 0 } else { emu(y) };
    pad + line_box_slack_emu(geo, lines.len(), y)
}

/// Extra `tIns` for a shrink-wrapped one-line frame whose height is the
/// lock line box, not a tall padded card.
///
/// Line boxes are `face × line_height_mult` (typically 1.2–1.35×). Height
/// beyond ~1.4× is `padding_pt.bottom` for a heading rule. Putting all
/// `h − face` in tIns stole that gap and sat glyphs on the underline.
fn line_box_slack_emu(geo: &GeometryNode, nlines: usize, first_y: i128) -> i64 {
    if nlines != 1 || first_y >= 1_000 {
        return 0;
    }
    let fs = body_font_size(geo);
    if fs <= 0 {
        return 0;
    }
    let h = geo.height.0;
    // 7/5 = 1.4×. Running headers (~1.35×) keep slack; padded underline
    // rows (~1.5–1.7×) leave the extra below the face.
    if h <= fs || h > fs.saturating_mul(7) / 5 {
        return 0;
    }
    emu(h - fs)
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
    // Ignore super/sub → body gaps (often ~0.4–0.5× face). Real leading is
    // at least ~0.8× the body face; otherwise Word `exact` crushes lines.
    let min_delta = super::align::body_font_size(geo)
        .saturating_mul(4)
        .saturating_div(5)
        .max(2_000);
    for i in 0..lines.len() - 1 {
        let delta = (baseline(&lines[i + 1]) - baseline(&lines[i])).abs();
        if delta >= min_delta {
            return Some(millipt_to_twips(i64::try_from(delta).unwrap_or(0)));
        }
    }
    None
}

/// Last pinned paragraph uses face size, not inter-line delta. See
/// `lock_ink_height` — trailing Office leading would cover the next node.
pub(crate) fn last_line_twips(font_size: Pt) -> i64 {
    crate::coord::pt_to_twips(font_size).max(20)
}

pub(crate) fn vert_center(geo: Option<&GeometryNode>, rect: &Rect, font_size: Pt) -> bool {
    let Some(geo) = geo else {
        return false;
    };
    // Compact pills (hero format chips ~1.9× face) still have leftover to
    // center in. Reject only when the box is no taller than the face — there
    // is then no vertical slack to distinguish padding from a tight line.
    if rect.height.0 <= font_size.0 {
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
    use super::list_hanging_lock_emu;
    use super::vert_center;
    use crate::coord::emu_to_twips;
    use crate::ir::TextAlign;
    use k2f_core::{GeometryNode, GlyphPosition, Pt, Rect, TextGlyphRun, TextPaintStyle};

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

    fn pill_geo(width: i128, height: i128, y: i128) -> GeometryNode {
        GeometryNode {
            id: "pill".into(),
            x: Pt(0),
            y: Pt(0),
            width: Pt(width),
            height: Pt(height),
            glyphs: vec![GlyphPosition {
                glyph_id: 1,
                cluster: 0,
                x_offset: Pt(12_000),
                y_offset: Pt(y),
                x_advance: Pt(8_000),
                y_advance: Pt(0),
            }],
            text_runs: vec![],
            fill_rects: vec![],
            children: vec![],
        }
    }

    #[test]
    fn compact_format_chip_is_vert_centered() {
        // k2f-hero right labels: 14pt face in a 26.8pt pill, 5pt first baseline.
        // The old 2×-face gate treated this as a tight line and left tIns +
        // anchor=t, so Word sat the glyphs low in the chip.
        let fs = Pt(14_000);
        let h = 26_800;
        let geo = pill_geo(58_794, h, 5_000);
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(58_794),
            height: Pt(h),
        };
        assert!(vert_center(Some(&geo), &rect, fs));
        let tight = pill_geo(58_794, 14_000, 0);
        let tight_rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(58_794),
            height: Pt(14_000),
        };
        assert!(!vert_center(Some(&tight), &tight_rect, fs));
    }

    #[test]
    fn cell_h_insets_keep_right_pad_when_line_fills_the_box() {
        let geo = GeometryNode {
            id: "g".into(),
            x: Pt(0),
            y: Pt(0),
            width: Pt(50_000),
            height: Pt(13_000),
            glyphs: vec![GlyphPosition {
                glyph_id: 1,
                cluster: 0,
                x_offset: Pt(2_000),
                y_offset: Pt(2_500),
                x_advance: Pt(40_000),
                y_advance: Pt(0),
            }],
            text_runs: vec![],
            fill_rects: vec![],
            children: vec![],
        };
        let (l, r) = super::cell_h_insets_emu(Some(&geo), crate::ir::TextAlign::Right);
        assert_eq!((l, r), (0, super::emu(8_000)));
        let (ll, lr) = super::cell_h_insets_emu(Some(&geo), crate::ir::TextAlign::Left);
        assert_eq!((ll, lr), (super::emu(2_000), 0));
    }

    #[test]
    fn list_hanging_lock_without_geo_is_zero() {
        assert_eq!(list_hanging_lock_emu(None), 0);
    }

    fn header_line_geo(height: i128, font: i128, y: i128) -> GeometryNode {
        GeometryNode {
            id: "header.running.left".into(),
            x: Pt(0),
            y: Pt(0),
            width: Pt(372_955),
            height: Pt(height),
            glyphs: vec![GlyphPosition {
                glyph_id: 1,
                cluster: 0,
                x_offset: Pt(0),
                y_offset: Pt(y),
                x_advance: Pt(8_000),
                y_advance: Pt(0),
            }],
            text_runs: vec![TextGlyphRun {
                glyph_range: [0, 1],
                style: TextPaintStyle {
                    font_family: "Roboto-Regular".into(),
                    font_size: Pt(font),
                    color: "#7C8390".into(),
                    bold: false,
                    italic: false,
                    strikethrough: false,
                    underline: false,
                },
            }],
            fill_rects: vec![],
            children: vec![],
        }
    }

    #[test]
    fn line_sized_header_keeps_line_box_slack_as_tins() {
        // Academic running header: 7.5pt face in a 10.125pt line box, y_offset 0.
        // Office was packing to the page top; lock leftover is tIns.
        let geo = header_line_geo(10_125, 7_500, 0);
        let (_, top, _, _) = super::insets(Some(&geo), TextAlign::Left);
        assert_eq!(
            top,
            super::emu(2_625),
            "line-box slack must become tIns, got {top}"
        );
    }

    #[test]
    fn tall_frame_does_not_treat_empty_body_as_tins() {
        let geo = header_line_geo(80_000, 12_000, 0);
        let (_, top, _, _) = super::insets(Some(&geo), TextAlign::Left);
        assert_eq!(
            top, 0,
            "tall card leftover is not line-box slack, got {top}"
        );
    }

    #[test]
    fn padded_underline_row_keeps_gap_below_not_tins() {
        // 14pt face, 1.3 leading + 3pt bottom pad = 21.2pt (~1.51×).
        // That extra is the heading-rule gap, not header air.
        let geo = header_line_geo(21_200, 14_000, 0);
        let (_, top, _, _) = super::insets(Some(&geo), TextAlign::Left);
        assert_eq!(top, 0, "underline padding must stay below glyphs, got {top}");
    }

    #[test]
    fn padded_first_line_still_uses_glyph_y_offset() {
        let geo = header_line_geo(10_125, 7_500, 4_000);
        let (_, top, _, _) = super::insets(Some(&geo), TextAlign::Left);
        assert_eq!(
            top,
            super::emu(4_000),
            "explicit pad stays y_offset, got {top}"
        );
    }

    #[test]
    fn list_hanging_is_marker_column_not_body_left() {
        // Decorative "1." at 15pt, body at 30pt — hang must be 15pt, not 30pt.
        let geo = GeometryNode {
            id: "li".into(),
            x: Pt(0),
            y: Pt(0),
            width: Pt(470_000),
            height: Pt(48_000),
            glyphs: vec![
                GlyphPosition {
                    glyph_id: 1,
                    cluster: GlyphPosition::CLUSTER_NOT_SOURCE,
                    x_offset: Pt(15_000),
                    y_offset: Pt(0),
                    x_advance: Pt(6_000),
                    y_advance: Pt(0),
                },
                GlyphPosition {
                    glyph_id: 2,
                    cluster: GlyphPosition::CLUSTER_NOT_SOURCE,
                    x_offset: Pt(21_000),
                    y_offset: Pt(0),
                    x_advance: Pt(3_000),
                    y_advance: Pt(0),
                },
                GlyphPosition {
                    glyph_id: 3,
                    cluster: 0,
                    x_offset: Pt(30_000),
                    y_offset: Pt(0),
                    x_advance: Pt(8_000),
                    y_advance: Pt(0),
                },
                GlyphPosition {
                    glyph_id: 3,
                    cluster: 1,
                    x_offset: Pt(30_000),
                    y_offset: Pt(24_000),
                    x_advance: Pt(8_000),
                    y_advance: Pt(0),
                },
            ],
            text_runs: vec![],
            fill_rects: vec![],
            children: vec![],
        };
        let hang = list_hanging_lock_emu(Some(&geo));
        assert_eq!(
            emu_to_twips(hang),
            300,
            "15pt marker column, got {hang} emu"
        );
    }
}
