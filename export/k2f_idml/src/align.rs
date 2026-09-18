use crate::ir::TextAlign;
use k2f_core::{GeometryNode, GlyphPosition, Modifier, Pt};

const MIN_SLACK: i128 = 2_000;

pub fn infer_text_align(geo: &GeometryNode, text: &str) -> TextAlign {
    infer_text_align_for(geo, text, &[])
}

/// Infer alignment from lock ink, ignoring superscript/subscript-only rows.
///
/// Those rows sit on a separate baseline with huge side slack, so max-slack
/// inference would flip a centered author line to Right and a centered
/// affiliation stack to Left.
pub(crate) fn infer_text_align_for(
    geo: &GeometryNode,
    text: &str,
    modifiers: &[Modifier],
) -> TextAlign {
    let mut lines = body_lines(geo, modifiers);
    if lines.is_empty() {
        lines = source_lines(geo);
    }
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
        Some((_, left, right)) => infer_from_gaps(left, right, box_w),
        None => TextAlign::Left,
    }
}

pub(crate) fn source_glyphs(geo: &GeometryNode) -> Vec<&GlyphPosition> {
    geo.glyphs
        .iter()
        .filter(|g| g.cluster != GlyphPosition::CLUSTER_NOT_SOURCE)
        .collect()
}

/// Same baseline = same line (integer millipt). Plan 2.3.1.
pub(crate) fn source_lines(geo: &GeometryNode) -> Vec<Vec<&GlyphPosition>> {
    let mut glyphs = source_glyphs(geo);
    if glyphs.is_empty() {
        return Vec::new();
    }
    glyphs.sort_by_key(|g| (g.y_offset.0, g.x_offset.0));
    let mut lines: Vec<Vec<&GlyphPosition>> = Vec::new();
    for g in glyphs {
        if let Some(line) = lines.last_mut() {
            if line[0].y_offset.0 == g.y_offset.0 {
                line.push(g);
                continue;
            }
        }
        lines.push(vec![g]);
    }
    lines
}

/// Lock rows that contain only superscript/subscript modifier glyphs are not
/// separate wrap lines — they sit above/below the main baseline (author marks).
pub(crate) fn body_lines<'a>(
    geo: &'a GeometryNode,
    modifiers: &[Modifier],
) -> Vec<Vec<&'a GlyphPosition>> {
    source_lines(geo)
        .into_iter()
        .filter(|line| !is_script_only_line(line, modifiers))
        .collect()
}

fn is_script_only_line(line: &[&GlyphPosition], modifiers: &[Modifier]) -> bool {
    if line.is_empty() {
        return true;
    }
    line.iter().all(|g| {
        let c = g.cluster as usize;
        modifiers.iter().any(|m| {
            (m.mod_type == "superscript" || m.mod_type == "subscript")
                && m.range[0] <= c
                && c < m.range[1]
        })
    })
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

fn source_paragraphs(text: &str) -> usize {
    text.split('\n').filter(|s| !s.is_empty()).count().max(1)
}

/// Pin lock wrap points only for tight 2-line display titles.
///
/// Column body that wrapped because the lock frame was full must *not* become
/// hard `<Br/>` in IDML. Host metrics and missing fonts change line width;
/// keeping the lock wrap points then double-wraps ("atmospheres," / "for" on
/// their own lines). Slight reflow is the IDML contract. 3+ lock lines, or any
/// line that fills the frame, is column wrap — not a designed display break.
///
/// Exception: a wrap inside a spaceless token. InDesign will not break that
/// word (Hyphenation is off), so the whole story oversets. The lock already
/// chose the break; pin it.
pub(crate) fn should_pin_lock_breaks(
    geo: Option<&GeometryNode>,
    _font_size: k2f_core::Pt,
    text: Option<&str>,
    align: TextAlign,
    modifiers: &[Modifier],
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
    if text.contains('\n') {
        return false;
    }
    let lines = body_lines(geo, modifiers);
    if lines.len() < 2 || lines.len() <= source_paragraphs(text) {
        return false;
    }
    if has_midword_lock_wrap(geo, text, modifiers) {
        return true;
    }
    if lines.len() >= 3 || has_full_width_lock_line(geo) {
        return false;
    }
    true
}

/// True when a lock line is a single unbreakable token that the lock wrapped.
///
/// Column body wraps at spaces. InDesign can reflow those lines. A spaceless
/// line (RECONSTRUCTION) has no wrap opportunity, so the lock break must be
/// pinned or the whole story oversets.
pub(crate) fn has_midword_lock_wrap(
    geo: &GeometryNode,
    text: &str,
    modifiers: &[Modifier],
) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let mut prev = 0usize;
    for &idx in &lock_break_char_indices(geo, text, modifiers) {
        if idx > 0
            && idx < chars.len()
            && !chars[idx].is_whitespace()
            && !chars[idx - 1].is_whitespace()
            && !chars[prev..idx].iter().any(|c| c.is_whitespace())
        {
            return true;
        }
        prev = idx;
    }
    false
}

const INK_SLACK: i128 = 16_000;
/// Extra millipt of right slack: NoBreak lines still WidthOnly-grow so host
/// metrics do not overset (hide) the line.
const NOBREAK_WIDTH_SLACK: i128 = 48_000;

/// Grow width only when lock ink already fills the frame. Wide left-aligned
/// footers must not WidthOnly-autosize (InDesign re-anchors at ItemTransform).
pub(crate) fn should_autosize_width(geo: Option<&GeometryNode>, align: TextAlign) -> bool {
    ink_tighter_than(geo, align, INK_SLACK)
}

/// Lock paint does not clip. A wrapped last line can start at or past
/// `geo.height` when the box was sized for N−1 lines. IDML frames clip
/// (overset hides the line), so expand to that line's ink (`y_offset + font_size`).
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

pub(crate) fn should_autosize_width_for_nobreak(
    geo: Option<&GeometryNode>,
    align: TextAlign,
) -> bool {
    ink_tighter_than(geo, align, NOBREAK_WIDTH_SLACK)
}

fn ink_tighter_than(geo: Option<&GeometryNode>, align: TextAlign, slack_limit: i128) -> bool {
    if matches!(align, TextAlign::Justify) {
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
    let mut min_left = i128::MAX;
    let mut min_right = i128::MAX;
    let mut min_slack = i128::MAX;
    for line in &lines {
        let (slack, left, right) = line_gaps(line, box_w);
        min_left = min_left.min(left);
        min_right = min_right.min(right);
        min_slack = min_slack.min(slack);
    }
    // Full-width column copy (left ink flush, wide right slack) is not a
    // shrink-wrapped NoBreak label — WidthOnly recenters in the card column.
    // Use NOBREAK_WIDTH_SLACK (48pt): tighter slack still needs host-metric
    // headroom (Bold subject lines overset and hide when NoBreak + no grow).
    if lines.len() == 1 && min_left <= MIN_SLACK {
        let max_edge = lines[0]
            .iter()
            .map(|g| g.x_offset.0 + g.x_advance.0)
            .max()
            .unwrap_or(0);
        // Nearly full-width *column* line with very tight right slack (~22pt)
        // is bibliography-style wrap. Shrink-wrapped display titles (a hug box
        // around the ink, often ~300pt) still need WidthOnly or InDesign hides
        // the NoBreak story. Only skip grow on wide frames.
        if box_w >= 400_000 && max_edge * 20 >= box_w * 17 && min_right < 30_000 {
            return false;
        }
        if min_right > NOBREAK_WIDTH_SLACK {
            return false;
        }
    }
    // Wrapped paragraph body: a lock line that fills the column is host wrap,
    // not shrink-wrapped display ink. WidthOnly then grows about ItemTransform
    // and the glyphs spill past both sides of the original box.
    if lines.len() >= 2 && has_full_width_lock_line(geo) {
        return false;
    }
    min_right < slack_limit || min_slack < slack_limit
}

/// True when any lock line spans the column (host wrap, not tight display ink).
///
/// Exact edge-to-edge is rare: rustybuzz leaves a few pt of right slack on the
/// line that triggered the wrap. Treating that as "tight" enabled WidthOnly,
/// and InDesign grew the frame around its center past the original border.
pub(crate) fn has_full_width_lock_line(geo: &GeometryNode) -> bool {
    let box_w = geo.width.0;
    if box_w <= 0 {
        return false;
    }
    source_lines(geo).iter().any(|line| {
        let (_, left, right) = line_gaps(line, box_w);
        if left > MIN_SLACK {
            return false;
        }
        right <= MIN_SLACK || right.saturating_mul(10) <= box_w
    })
}

pub(crate) fn autosize_reference(geo: Option<&GeometryNode>, align: TextAlign) -> &'static str {
    if let Some(geo) = geo {
        if let Some(line) = source_lines(geo).first() {
            let (_, left, right) = line_gaps(line, geo.width.0);
            if left < INK_SLACK && right < INK_SLACK {
                return "CenterPoint";
            }
        }
    }
    match align {
        TextAlign::Center => "CenterPoint",
        TextAlign::Right => "CenterRightPoint",
        TextAlign::Left | TextAlign::Justify => "CenterLeftPoint",
    }
}

/// Character indices in `text` where lock lines after the first begin.
pub(crate) fn lock_break_char_indices(
    geo: &GeometryNode,
    text: &str,
    modifiers: &[Modifier],
) -> Vec<usize> {
    let chars: Vec<char> = text.chars().collect();
    body_lines(geo, modifiers)
        .iter()
        .skip(1)
        .filter_map(|line| line.first().map(|g| g.cluster as usize))
        .filter(|&idx| {
            idx < chars.len() && chars[idx] != '\n' && (idx == 0 || chars[idx - 1] != '\n')
        })
        .collect()
}

fn infer_from_gaps(left: i128, right: i128, box_w: i128) -> TextAlign {
    let slack = left.saturating_add(right);
    if slack < MIN_SLACK {
        return TextAlign::Left;
    }
    let delta = left - right;
    // Wide table cells keep an 8pt inset. That leftover is not a centered
    // pill: `slack * 10 > box_w` would flip a nearly-full 168pt cell to
    // Center while the short cells in the same column stay Left.
    const WIDE_CELL: i128 = 100_000;
    const CELL_INSET: i128 = 12_000;
    if box_w >= WIDE_CELL && left <= CELL_INSET {
        return if delta > slack / 4 {
            TextAlign::Right
        } else {
            TextAlign::Left
        };
    }
    // Balanced side gaps with slack dominating the box width → pill/badge
    // chrome (VIP label, session pill). `left * 10 > box_w` missed ~7.5%
    // padding on both sides; total slack catches shrink-wrapped badges while
    // full-width lines with tiny equal margins stay Left (plan 2.3.1 table).
    if delta.abs() * 5 <= slack && slack * 10 > box_w {
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

    #[test]
    fn infer_from_plan_table() {
        let w = 100_000i128;
        assert_eq!(
            infer_text_align(&geo(w, vec![glyph(0, 0, 40_000, 0)]), "A"),
            TextAlign::Left
        );
        assert_eq!(
            infer_text_align(&geo(w, vec![glyph(0, 30_000, 40_000, 0)]), "A"),
            TextAlign::Center
        );
        assert_eq!(
            infer_text_align(&geo(w, vec![glyph(0, 60_000, 40_000, 0)]), "A"),
            TextAlign::Right
        );
        assert_eq!(
            infer_text_align(&geo(w, vec![glyph(0, 4_000, 92_000, 0)]), "A"),
            TextAlign::Left
        );
        // Shrink-wrapped pill (~7.5% padding each side): balanced slack
        // dominates box width even when left alone does not.
        assert_eq!(
            infer_text_align(
                &geo(60_104, vec![glyph(0, 4_500, 51_022, 0)]),
                "VIP PASS"
            ),
            TextAlign::Center
        );
        // Nearly-full table cell with 8pt inset: not a centered pill.
        assert_eq!(
            infer_text_align(
                &geo(168_000, vec![glyph(0, 8_000, 148_500, 0)]),
                "Executive Analytics UI & Design System"
            ),
            TextAlign::Left
        );
        assert_eq!(infer_text_align(&geo(w, vec![]), "A"), TextAlign::Left);
    }

    #[test]
    fn script_only_rows_do_not_flip_centered_body() {
        // academic-serif-classic authors: superscripts at the end of a centered
        // line form a max-slack row that would otherwise infer Right.
        let w = 483_000i128;
        let authors = geo(
            w,
            vec![
                glyph(21, 193_583, 8_000, -4_950),
                glyph(0, 71_997, 8_000, 0),
                glyph(50, 406_105, 8_000, 0),
            ],
        );
        let author_mods = vec![Modifier {
            range: [21, 24],
            mod_type: "superscript".into(),
            intent: "default".into(),
        }];
        assert_eq!(
            infer_text_align_for(&authors, "Alexander H. Sterling1,*", &author_mods),
            TextAlign::Center
        );
        // Leading affiliation marks are a near-full-width slack row on the left.
        let aff = geo(
            w,
            vec![
                glyph(0, 54_789, 4_008, -4_050),
                glyph(1, 58_797, 8_000, 0),
                glyph(70, 420_203, 8_000, 0),
            ],
        );
        let aff_mods = vec![Modifier {
            range: [0, 1],
            mod_type: "superscript".into(),
            intent: "default".into(),
        }];
        assert_eq!(
            infer_text_align_for(&aff, "1Department of Computer Science", &aff_mods),
            TextAlign::Center
        );
    }

    #[test]
    fn overflowing_last_line_expands_ink_height() {
        let fs = Pt(13_000);
        let mut g = GeometryNode {
            id: "g".into(),
            x: Pt(0),
            y: Pt(0),
            width: Pt(100_000),
            height: Pt(54_600),
            glyphs: vec![
                glyph(0, 0, 10_000, 0),
                glyph(1, 0, 10_000, 18_200),
                glyph(2, 0, 10_000, 36_400),
                glyph(3, 0, 10_000, 54_600),
            ],
            text_runs: vec![],
            fill_rects: vec![],
            children: vec![],
        };
        assert_eq!(lock_ink_height(Some(&g), fs), Some(Pt(67_600)));
        g.height = Pt(80_000);
        assert_eq!(lock_ink_height(Some(&g), fs), None);
    }

    #[test]
    fn full_width_column_line_skips_nobreak_widthonly() {
        let w = 144_333i128;
        let geo = geo(w, vec![glyph(0, 0, 80_000, 0)]);
        assert!(
            !should_autosize_width_for_nobreak(Some(&geo), TextAlign::Left),
            "flush-left column copy must not WidthOnly-shrink"
        );
    }

    #[test]
    fn wrapped_multiline_body_skips_nobreak_widthonly() {
        // Bibliography-style entry: line 1 fills the column, line 2 is short.
        let w = 483_000i128;
        let geo = geo(
            w,
            vec![glyph(0, 0, 483_000, 0), glyph(0, 0, 101_316, 12_600)],
        );
        assert!(
            !should_autosize_width_for_nobreak(Some(&geo), TextAlign::Left),
            "wrapped multi-line body must not WidthOnly-shrink"
        );
    }

    #[test]
    fn hug_display_title_still_needs_nobreak_grow() {
        // botanical-elegance header title: ink fills a ~319pt hug box.
        let w = 318_999i128;
        let geo = geo(w, vec![glyph(0, 0, 318_999, 0)]);
        assert!(
            should_autosize_width_for_nobreak(Some(&geo), TextAlign::Left),
            "shrink-wrapped title must WidthOnly-grow or host italic oversets"
        );
    }

    #[test]
    fn nearly_full_width_single_line_skips_nobreak_widthonly() {
        let w = 483_000i128;
        let geo = geo(w, vec![glyph(0, 0, 461_264, 0)]);
        assert!(
            !should_autosize_width_for_nobreak(Some(&geo), TextAlign::Left),
            "95% column line with ~22pt slack must not WidthOnly-shrink"
        );
    }

    #[test]
    fn tight_full_width_subject_line_still_needs_nobreak_grow() {
        let w = 487_000i128;
        let geo = geo(w, vec![glyph(0, 0, 446_318, 0)]);
        assert!(
            should_autosize_width_for_nobreak(Some(&geo), TextAlign::Left),
            "bold subject with ~41pt slack still needs NoBreak width grow"
        );
    }

    #[test]
    fn two_line_letter_body_with_wrap_slack_is_full_width() {
        // botanical-banner p2: 504pt column, longest lock line ~12pt short.
        let w = 504_000i128;
        let geo = geo(
            w,
            vec![glyph(0, 0, 477_663, 0), glyph(90, 0, 491_544, 12_600)],
        );
        assert!(
            has_full_width_lock_line(&geo),
            "12pt leftover on a 504pt column is wrap remainder, not tight ink"
        );
        assert!(
            !should_autosize_width_for_nobreak(Some(&geo), TextAlign::Left),
            "2-line letter body must not WidthOnly-grow past the box"
        );
        let text = "I will send a small swatch book by courier on Thursday, along with a note on lead times for the hand-stitched editions. Please let me know if you would like a second set for your London studio.";
        assert!(
            !should_pin_lock_breaks(Some(&geo), Pt(11_000), Some(text), TextAlign::Left, &[]),
            "column wrap must reflow in the lock frame, not pin NoBreak lines"
        );
    }

    #[test]
    fn tight_multiline_display_title_keeps_nobreak_widthonly() {
        let w = 180_000i128;
        let mut glyphs = Vec::new();
        for (i, ch) in "Editorial Grid.\nNarrative System.".chars().enumerate() {
            if ch == '\n' {
                continue;
            }
            let y = if i < 16 { 12_000 } else { 60_000 };
            glyphs.push(glyph(i as u32, (i as i128 % 16) * 10_000, 10_000, y));
        }
        let geo = geo(w, glyphs);
        assert!(
            should_autosize_width_for_nobreak(Some(&geo), TextAlign::Left),
            "tight 2-line display title still WidthOnly-grows"
        );
    }

    #[test]
    fn full_width_column_wrap_does_not_pin() {
        let text = "The quick brown fox jumps over the lazy dog and then wraps again.";
        let w = 200_000i128;
        let g = geo(
            w,
            vec![
                glyph(0, 0, 200_000, 0),
                glyph(20, 0, 80_000, 15_000),
                glyph(40, 0, 50_000, 30_000),
            ],
        );
        assert!(
            !should_pin_lock_breaks(Some(&g), Pt(12_000), Some(text), TextAlign::Left, &[]),
            "column wrap must not pin lock breaks"
        );
    }

    #[test]
    fn two_line_display_title_still_pins() {
        let text = "Editorial Grid. Narrative System.";
        let w = 200_000i128;
        let mut glyphs = Vec::new();
        for (i, _) in text.chars().enumerate() {
            let y = if i < 16 { 12_000 } else { 60_000 };
            glyphs.push(glyph(i as u32, (i as i128 % 16) * 8_000, 8_000, y));
        }
        let g = geo(w, glyphs);
        assert!(
            should_pin_lock_breaks(Some(&g), Pt(44_000), Some(text), TextAlign::Left, &[]),
            "tight 2-line display title still pins"
        );
    }

    #[test]
    fn midword_overflow_title_still_pins() {
        // Monumental single-word title: lock wraps inside the word because
        // the token is wider than the column (RECONSTRUCTION → N).
        let text = "RECONSTRUCTION";
        let w = 437_000i128;
        let mut glyphs = Vec::new();
        for i in 0..13 {
            glyphs.push(glyph(i as u32, i as i128 * 33_500, 33_500, 0));
        }
        glyphs.push(glyph(13, 0, 41_375, 52_200));
        let g = geo(w, glyphs);
        assert!(
            has_full_width_lock_line(&g),
            "first line fills the column"
        );
        assert!(
            has_midword_lock_wrap(&g, text, &[]),
            "wrap starts at N on a spaceless line"
        );
        assert!(
            should_pin_lock_breaks(Some(&g), Pt(58_000), Some(text), TextAlign::Left, &[]),
            "mid-word overflow must pin or InDesign oversets the word"
        );
    }

    #[test]
    fn superscript_glyph_row_does_not_pin_lock_breaks() {
        let text = "Alexander H. Sterling1,*, Elena Rostova2, and Marcus Vance3";
        let mods = vec![
            Modifier {
                range: [21, 24],
                mod_type: "superscript".into(),
                intent: "default".into(),
            },
            Modifier {
                range: [39, 40],
                mod_type: "superscript".into(),
                intent: "default".into(),
            },
            Modifier {
                range: [58, 59],
                mod_type: "superscript".into(),
                intent: "default".into(),
            },
        ];
        let g = geo(483_000, vec![
            glyph(21, 193_583, 8_000, -4_950),
            glyph(0, 71_997, 8_000, 0),
        ]);
        assert!(
            !should_pin_lock_breaks(Some(&g), Pt(11_000), Some(text), TextAlign::Right, &mods),
            "superscript rows must not insert a lock break at char 0"
        );
    }
}
