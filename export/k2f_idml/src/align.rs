use crate::ir::TextAlign;
use k2f_core::{GeometryNode, GlyphPosition, Pt};

const MIN_SLACK: i128 = 2_000;

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

/// Pin lock wrap points whenever glyph lines exceed semantic paragraphs.
/// A tight-height check is not used: extra host wrap overprints in IDML.
pub(crate) fn should_pin_lock_breaks(
    geo: Option<&GeometryNode>,
    _font_size: k2f_core::Pt,
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
    lines.len() >= 2 && lines.len() > source_paragraphs(text)
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
    let mut min_right = i128::MAX;
    let mut min_slack = i128::MAX;
    for line in &lines {
        let (slack, _, right) = line_gaps(line, box_w);
        min_right = min_right.min(right);
        min_slack = min_slack.min(slack);
    }
    min_right < slack_limit || min_slack < slack_limit
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

fn infer_from_gaps(left: i128, right: i128, box_w: i128) -> TextAlign {
    let slack = left.saturating_add(right);
    if slack < MIN_SLACK {
        return TextAlign::Left;
    }
    let delta = left - right;
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
        assert_eq!(infer_text_align(&geo(w, vec![]), "A"), TextAlign::Left);
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
}
