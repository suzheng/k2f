use crate::ir::TextAlign;
use k2f_core::{GeometryNode, GlyphPosition, Pt};

const MIN_SLACK: i128 = 2_000;
/// Historical body default: 7.5pt. Half of 11–12pt faces still land here.
const MAX_LINE_CLUSTER_TOL: i128 = 7_500;
const MIN_LINE_CLUSTER_TOL: i128 = 2_000;

pub fn infer_text_align(geo: &GeometryNode, text: &str) -> TextAlign {
    let lines = source_lines(geo);
    if lines.is_empty() {
        return TextAlign::Left;
    }
    let box_w = geo.width.0;
    if lines.len() >= 2 && is_justify(&lines, text, box_w) {
        return TextAlign::Justify;
    }
    let mut best: Option<(i128, i128, i128)> = None;
    for glyphs in &lines {
        let (slack, left, right) = line_gaps(glyphs, box_w);
        if best.map(|(s, _, _)| slack > s).unwrap_or(true) {
            best = Some((slack, left, right));
        }
    }
    match best {
        Some((_, left, right)) => infer_from_gaps(left, right),
        None => TextAlign::Left,
    }
}

pub(crate) fn source_glyphs(geo: &GeometryNode) -> Vec<&GlyphPosition> {
    geo.glyphs
        .iter()
        .filter(|g| g.cluster != GlyphPosition::CLUSTER_NOT_SOURCE)
        .collect()
}

pub(crate) fn source_lines(geo: &GeometryNode) -> Vec<Vec<&GlyphPosition>> {
    let mut glyphs = source_glyphs(geo);
    if glyphs.is_empty() {
        return Vec::new();
    }
    glyphs.sort_by_key(|g| (g.y_offset.0, g.x_offset.0));
    let tol = line_cluster_tol(geo);

    let mut lines: Vec<Vec<&GlyphPosition>> = Vec::new();
    for g in glyphs {
        if let Some(line) = lines.iter_mut().find(|line| {
            line.iter()
                .any(|existing| (existing.y_offset.0 - g.y_offset.0).abs() <= tol)
        }) {
            line.push(g);
        } else {
            lines.push(vec![g]);
        }
    }
    lines.sort_by_key(|line| {
        let mut ys: Vec<i128> = line.iter().map(|g| g.y_offset.0).collect();
        ys.sort_unstable();
        ys[ys.len() / 2]
    });
    for line in &mut lines {
        line.sort_by_key(|g| g.x_offset.0);
    }
    lines
}

/// Same-line glyphs stay together; wrapped lines of this face must split.
/// Half the lock font, clamped so 5–7pt body (6.2pt leading) is not merged
/// by the historical 7.5pt cap. 11–12pt tests keep that cap.
///
/// Use the **largest** face in the box, not `text_runs[0]`. A leading
/// superscript/subscript run is often smaller; half of that face is too
/// tight and splits raised digits onto their own "lines", so line spacing
/// collapses to the super→body gap (~4pt) and multi-line affiliations crush.
pub(crate) fn body_font_size(geo: &GeometryNode) -> i128 {
    geo.text_runs
        .iter()
        .map(|r| r.style.font_size.0.abs())
        .filter(|n| *n > 0)
        .max()
        .unwrap_or(12_000)
}

fn line_cluster_tol(geo: &GeometryNode) -> i128 {
    (body_font_size(geo) / 2).clamp(MIN_LINE_CLUSTER_TOL, MAX_LINE_CLUSTER_TOL)
}

/// Office wrap on a glyph-tight lock box reflows the last word onto a clipped
/// second line when host bold/metrics are wider than rustybuzz. Keep wrap only
/// when the lock already wrapped *within* a paragraph. One-line-tall boxes
/// (pills, title/date rows) must not wrap. Explicit `\n` lines stay
/// `wrap=none` even in a padded cell that still fits N+1 lines — host wrap
/// of a long second paragraph would paint a third row over the next cell
/// (invoice line-items, two-line labels).
pub(crate) fn should_wrap_lock(
    geo: Option<&GeometryNode>,
    font_size: Pt,
    text: Option<&str>,
) -> bool {
    let Some(geo) = geo else {
        return true;
    };
    let lines = source_lines(geo);
    let paras = text.map(source_paragraphs).unwrap_or(0);
    if lines.len() >= 2 && (paras == 0 || lines.len() > paras) {
        return true;
    }
    if lines.len() >= 2 {
        return false;
    }
    let Some(line) = lines.first() else {
        return true;
    };
    if !fits_n_lines(geo, line, font_size, 2) {
        return false;
    }
    let (slack, _, _) = line_gaps(line, geo.width.0);
    slack >= MIN_SLACK
}

/// `wrap=none` on a wide center/justify/right frame makes hosts ignore `w:jc`
/// / `algn` and paint at `lIns` (lock-left of the box). Turn wrap on when
/// leftover width shows the alignment. Narrow glyph-tight pills (≥85% full)
/// stay `wrap=none` so they do not clip a second row. Wide near-full center
/// lines (cover subtitles) still need square wrap — otherwise Word shifts
/// them by the full left gap. Multi-line frames use `any`: a short centered
/// second line must force wrap even when line 1 is nearly full. Left stays
/// `wrap=none` — painting at `lIns` is correct.
pub(crate) fn host_wrap(geo: Option<&GeometryNode>, align: TextAlign, wrap: bool) -> bool {
    if wrap {
        return true;
    }
    if matches!(align, TextAlign::Left) {
        return false;
    }
    let Some(geo) = geo else {
        return false;
    };
    let lines = source_lines(geo);
    if lines.is_empty() {
        return false;
    }
    let box_w = geo.width.0;
    lines.iter().any(|line| {
        let (_, left, right) = line_gaps(line, box_w);
        match align {
            // Right: only left leftover proves alignment; right may be a thin
            // pad (< MIN_SLACK) or flush. Same fill check as Right insets.
            TextAlign::Right => {
                left >= MIN_SLACK && !line_fills_padded_width(right, left, box_w)
            }
            TextAlign::Center | TextAlign::Justify => {
                left >= MIN_SLACK
                    && right >= MIN_SLACK
                    && (!line_fills_padded_width(left, right, box_w)
                        || box_w >= WIDE_CENTER_MIN)
            }
            TextAlign::Left => false,
        }
    })
}

/// ~200pt. Page-width / hero frames, not chip/pill boxes (~80–90pt).
const WIDE_CENTER_MIN: i128 = 200_000;

fn line_fills_padded_width(pad: i128, opposite_slack: i128, box_w: i128) -> bool {
    let avail = box_w.saturating_sub(pad.max(0));
    if avail <= 0 {
        return false;
    }
    let content = avail.saturating_sub(opposite_slack.max(0));
    content.saturating_mul(100) >= avail.saturating_mul(85)
}

fn source_paragraphs(text: &str) -> usize {
    text.split('\n').count().max(1)
}

fn fits_n_lines(geo: &GeometryNode, line: &[&GlyphPosition], font_size: Pt, n: usize) -> bool {
    let y = line.iter().map(|g| g.y_offset.0).min().unwrap_or(0).max(0);
    let content_h = geo.height.0.saturating_sub(y);
    let n = i128::try_from(n).unwrap_or(2).max(1);
    content_h >= font_size.0.saturating_mul(n)
}

/// Lock-wrapped paragraph in a frame that cannot fit N+1 lines: host wrap
/// creates a clipped extra row. Caller should insert `\n` at lock line
/// starts and keep wrap off. Justify still wraps (hard breaks would drop
/// last-line raggedness).
pub(crate) fn should_pin_lock_breaks(
    geo: Option<&GeometryNode>,
    font_size: Pt,
    text: Option<&str>,
    align: TextAlign,
) -> bool {
    if matches!(align, TextAlign::Justify) {
        return false;
    }
    let Some(geo) = geo else {
        return false;
    };
    let Some(text) = text else {
        return false;
    };
    let lines = source_lines(geo);
    if lines.len() < 2 || lines.len() <= source_paragraphs(text) {
        return false;
    }
    !fits_n_lines(geo, &lines[0], font_size, lines.len() + 1)
}

/// Lock paint does not clip. A wrapped last line can start at or past
/// `geo.height` when the box was sized for N−1 lines. Office frames clip to
/// `cy`, so expand to that line's ink (`y_offset + font_size`).
pub(crate) fn lock_ink_height(geo: Option<&GeometryNode>, font_size: Pt) -> Option<Pt> {
    let geo = geo?;
    let lines = source_lines(geo);
    let last = lines.last()?;
    let last_y = last.iter().map(|g| g.y_offset.0).min().unwrap_or(0).max(0);
    let ink = last_y.saturating_add(font_size.0.max(0));
    if ink > geo.height.0 {
        Some(Pt(ink))
    } else {
        None
    }
}

/// Character indices (in `text`) where lock lines after the first begin.
/// Skip a line that already follows an explicit `\n` — pinning that index
/// would emit a blank Office paragraph (hero titles with a hard break plus wrap).
pub(crate) fn lock_break_char_indices(geo: &GeometryNode, text: &str) -> Vec<usize> {
    let chars: Vec<char> = text.chars().collect();
    source_lines(geo)
        .iter()
        .skip(1)
        .filter_map(|line| line.first().map(|g| g.cluster as usize))
        .filter(|&idx| {
            idx < chars.len() && chars[idx] != '\n' && (idx == 0 || chars[idx - 1] != '\n')
        })
        .collect()
}

pub(crate) fn line_gaps(glyphs: &[&GlyphPosition], box_w: i128) -> (i128, i128, i128) {
    let left = glyphs.iter().map(|g| g.x_offset.0).min().unwrap_or(0);
    let right_edge = glyphs
        .iter()
        .map(|g| g.x_offset.0 + g.x_advance.0)
        .max()
        .unwrap_or(0);
    let right = box_w - right_edge;
    (left.saturating_add(right), left, right)
}

fn infer_from_gaps(left: i128, right: i128) -> TextAlign {
    let slack = left.saturating_add(right);
    if slack < MIN_SLACK {
        return TextAlign::Left;
    }
    let delta = left - right;
    // Equal leftover on both sides is padding (pills, chips), not a wide
    // left-aligned frame. Hosts need that slack for metrics, so center.
    if delta.abs() * 5 <= slack && left >= MIN_SLACK && right >= MIN_SLACK {
        TextAlign::Center
    } else if delta > slack / 4 {
        TextAlign::Right
    } else {
        TextAlign::Left
    }
}

fn is_justify(lines: &[Vec<&GlyphPosition>], text: &str, box_w: i128) -> bool {
    let last = lines.last().expect("non-empty");
    let last_spaces = space_advances(last, text);
    if last_spaces.is_empty() {
        return false;
    }
    let mut sorted = last_spaces.clone();
    sorted.sort_unstable();
    let median = sorted[sorted.len() / 2];
    let (last_slack, _, _) = line_gaps(last, box_w);
    for line in &lines[..lines.len() - 1] {
        let (slack, _, _) = line_gaps(line, box_w);
        for adv in space_advances(line, text) {
            if adv > median.saturating_mul(3) / 2 && last_slack > slack {
                return true;
            }
        }
    }
    false
}

fn space_advances(glyphs: &[&GlyphPosition], text: &str) -> Vec<i128> {
    glyphs
        .iter()
        .filter(|g| char_at(text, g.cluster as usize) == Some(' '))
        .map(|g| g.x_advance.0)
        .collect()
}

fn char_at(text: &str, cluster: usize) -> Option<char> {
    text.chars().nth(cluster)
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

    fn geo(width: i128, glyphs: Vec<GlyphPosition>) -> GeometryNode {
        GeometryNode {
            id: "g".into(),
            x: Pt(0),
            y: Pt(0),
            width: Pt(width),
            height: Pt(40_000),
            glyphs,
            text_runs: vec![],
            fill_rects: vec![],
            children: vec![],
        }
    }

    fn with_font(mut g: GeometryNode, size: i128) -> GeometryNode {
        g.text_runs = vec![k2f_core::TextGlyphRun {
            glyph_range: [0, g.glyphs.len()],
            style: k2f_core::TextPaintStyle {
                font_family: "default".into(),
                font_size: Pt(size),
                color: "#000000".into(),
                bold: false,
                italic: false,
                strikethrough: false,
                underline: false,
            },
        }];
        g
    }

    #[test]
    fn wrap_only_when_lock_has_slack_or_multiple_lines() {
        let fs = Pt(10_000);
        let tight = geo(40_000, vec![glyph(0, 0, 40_000, 0)]);
        assert!(!should_wrap_lock(Some(&tight), fs, None));
        let slack = geo(100_000, vec![glyph(0, 0, 40_000, 0)]);
        assert!(should_wrap_lock(Some(&slack), fs, None));
        let wrapped = geo(
            40_000,
            vec![glyph(0, 0, 40_000, 0), glyph(1, 0, 20_000, 14_000)],
        );
        assert!(should_wrap_lock(Some(&wrapped), fs, None));
        assert!(should_wrap_lock(None, fs, None));
    }

    #[test]
    fn wrap_off_when_single_line_frame_is_too_short() {
        let mut short = geo(250_000, vec![glyph(0, 0, 220_000, 500)]);
        short.height = Pt(13_000);
        assert!(!should_wrap_lock(Some(&short), Pt(9_500), None));
        let mut padded = geo(87_000, vec![glyph(0, 6_500, 74_000, 2_500)]);
        padded.height = Pt(13_250);
        assert!(!should_wrap_lock(Some(&padded), Pt(7_500), None));
    }

    #[test]
    fn host_wrap_square_for_wide_center_one_liner() {
        let fs = Pt(12_000);
        let mut wide = geo(100_000, vec![glyph(0, 30_000, 40_000, 0)]);
        wide.height = Pt(13_000);
        assert!(!should_wrap_lock(Some(&wide), fs, None));
        assert!(host_wrap(
            Some(&wide),
            TextAlign::Center,
            should_wrap_lock(Some(&wide), fs, None)
        ));
        let mut left = geo(100_000, vec![glyph(0, 0, 40_000, 0)]);
        left.height = Pt(13_000);
        assert!(!host_wrap(
            Some(&left),
            TextAlign::Left,
            should_wrap_lock(Some(&left), fs, None)
        ));
        let mut pill = geo(87_000, vec![glyph(0, 6_500, 74_000, 0)]);
        pill.height = Pt(13_250);
        assert!(!host_wrap(
            Some(&pill),
            TextAlign::Center,
            should_wrap_lock(Some(&pill), fs, None)
        ));
        // Page-width near-full center line (cover subtitle): still square so
        // Word honors jc — wrap=none would shift by the full left gap.
        let mut wide_full = geo(511_000, vec![glyph(0, 20_500, 470_000, 0)]);
        wide_full.height = Pt(15_000);
        assert!(host_wrap(Some(&wide_full), TextAlign::Center, false));
        // Soft-wrapped hero title: line 1 nearly full, line 2 short centered.
        let mut hero = geo(
            511_000,
            vec![
                glyph(0, 75_000, 361_000, 0),
                glyph(1, 178_000, 155_000, 29_000),
            ],
        );
        hero.height = Pt(58_000);
        assert!(host_wrap(Some(&hero), TextAlign::Center, false));
    }

    #[test]
    fn host_wrap_square_for_wide_right_one_liner() {
        let fs = Pt(7_500);
        // Unfilled table cell: short number, right-aligned, thin right pad.
        // wrap=none made Word ignore w:jc=right (zigzag vs filled alt rows).
        let mut cell = geo(78_000, vec![glyph(0, 50_000, 22_000, 4_200)]);
        cell.height = Pt(17_400);
        assert!(!should_wrap_lock(Some(&cell), fs, Some("$680.0")));
        assert!(host_wrap(
            Some(&cell),
            TextAlign::Right,
            should_wrap_lock(Some(&cell), fs, Some("$680.0"))
        ));
        // Flush-right with negligible right pad still needs wrap.
        let mut flush = geo(78_000, vec![glyph(0, 55_000, 23_000, 4_200)]);
        flush.height = Pt(17_400);
        assert!(host_wrap(Some(&flush), TextAlign::Right, false));
        // Glyph-tight right line stays wrap=none.
        let mut tight = geo(40_000, vec![glyph(0, 2_000, 36_000, 0)]);
        tight.height = Pt(13_000);
        assert!(!host_wrap(Some(&tight), TextAlign::Right, false));
    }

    #[test]
    fn small_font_wrapped_lines_are_not_merged() {
        let fs = Pt(5_200);
        let mut g = with_font(
            geo(
                196_000,
                vec![glyph(0, 0, 190_000, 0), glyph(1, 0, 40_000, 6_240)],
            ),
            5_200,
        );
        g.height = Pt(12_480);
        assert_eq!(source_lines(&g).len(), 2);
        assert!(should_wrap_lock(Some(&g), fs, None));
    }

    #[test]
    fn leading_superscript_does_not_split_or_crush_lines() {
        // academic-serif affiliations: 6.3pt raised digit, 9pt body, 12.87pt leading.
        let mut g = geo(
            483_000,
            vec![
                glyph(0, 0, 4_000, -4_050),
                glyph(1, 4_000, 40_000, 0),
                glyph(2, 0, 4_000, 8_820),
                glyph(3, 4_000, 40_000, 12_870),
                glyph(4, 0, 4_000, 21_690),
                glyph(5, 4_000, 40_000, 25_740),
                glyph(6, 0, 40_000, 38_610),
            ],
        );
        g.height = Pt(51_210);
        g.text_runs = vec![
            k2f_core::TextGlyphRun {
                glyph_range: [0, 1],
                style: k2f_core::TextPaintStyle {
                    font_family: "default".into(),
                    font_size: Pt(6_300),
                    color: "#000000".into(),
                    bold: false,
                    italic: false,
                    strikethrough: false,
                    underline: false,
                },
            },
            k2f_core::TextGlyphRun {
                glyph_range: [1, 7],
                style: k2f_core::TextPaintStyle {
                    font_family: "default".into(),
                    font_size: Pt(9_000),
                    color: "#000000".into(),
                    bold: false,
                    italic: false,
                    strikethrough: false,
                    underline: false,
                },
            },
        ];
        assert_eq!(source_lines(&g).len(), 4, "super digits stay on body lines");
        assert_eq!(body_font_size(&g), 9_000);
    }

    #[test]
    fn wrap_off_when_explicit_lines_fill_the_frame() {
        let fs = Pt(7_200);
        let mut g = geo(
            76_000,
            vec![glyph(0, 0, 70_000, 0), glyph(1, 0, 74_000, 8_000)],
        );
        g.height = Pt(16_000);
        assert!(!should_wrap_lock(
            Some(&g),
            fs,
            Some("bwHPC Symposium\n23.10.2023 - Mannheim")
        ));
        assert!(should_wrap_lock(
            Some(&g),
            fs,
            Some("one paragraph that the lock wrapped onto two lines")
        ));
        g.height = Pt(16_000);
        assert!(should_pin_lock_breaks(
            Some(&g),
            fs,
            Some("one paragraph that the lock wrapped onto two lines"),
            TextAlign::Left
        ));
        assert!(!should_pin_lock_breaks(
            Some(&g),
            fs,
            Some("bwHPC Symposium\n23.10.2023 - Mannheim"),
            TextAlign::Left
        ));
        g.height = Pt(40_000);
        assert!(
            !should_wrap_lock(
                Some(&g),
                fs,
                Some("Crystal Cloud Core Platform Commitment\nDedicated multi-tenant control plane, 99.99% uptime SLA")
            ),
            "padded two-line cells must not host-wrap a long second paragraph"
        );
        assert!(!should_pin_lock_breaks(
            Some(&g),
            fs,
            Some("one paragraph that the lock wrapped onto two lines"),
            TextAlign::Left
        ));
    }

    #[test]
    fn overflowing_last_line_expands_ink_height() {
        let fs = Pt(13_000);
        let mut g = geo(
            250_000,
            vec![
                glyph(0, 0, 200_000, 0),
                glyph(1, 0, 160_000, 18_200),
                glyph(2, 0, 200_000, 36_400),
                glyph(3, 0, 80_000, 54_600),
            ],
        );
        g.height = Pt(54_600);
        assert_eq!(lock_ink_height(Some(&g), fs), Some(Pt(67_600)));
        g.height = Pt(80_000);
        assert_eq!(lock_ink_height(Some(&g), fs), None);
    }

    #[test]
    fn lock_breaks_skip_lines_that_already_follow_newline() {
        let text = "Ab\nCd wrap";
        let g = geo(
            40_000,
            vec![
                glyph(0, 0, 12_000, 0),
                glyph(1, 12_000, 12_000, 0),
                glyph(3, 0, 12_000, 12_000),
                glyph(4, 12_000, 12_000, 12_000),
                glyph(6, 0, 12_000, 24_000),
                glyph(7, 12_000, 12_000, 24_000),
            ],
        );
        assert_eq!(lock_break_char_indices(&g, text), vec![6]);
        let wrap_only = "one two";
        let wrapped = geo(
            40_000,
            vec![glyph(0, 0, 20_000, 0), glyph(4, 0, 20_000, 12_000)],
        );
        assert_eq!(lock_break_char_indices(&wrapped, wrap_only), vec![4]);
    }
}
