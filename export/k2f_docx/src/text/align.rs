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
fn line_cluster_tol(geo: &GeometryNode) -> i128 {
    let fs = geo
        .text_runs
        .first()
        .map(|r| r.style.font_size.0.abs())
        .filter(|n| *n > 0)
        .unwrap_or(12_000);
    (fs / 2).clamp(MIN_LINE_CLUSTER_TOL, MAX_LINE_CLUSTER_TOL)
}

/// Office wrap on a glyph-tight lock box reflows the last word onto a clipped
/// second line when host bold/metrics are wider than rustybuzz. Keep wrap only
/// when the lock already wrapped *within* a paragraph, or leftover width sits
/// in a frame tall enough for another line. One-line-tall boxes (pills,
/// title/date rows) must not wrap. Explicit `\n` lines in a frame that only
/// fits those lines also stay `wrap=none` so a host-wider last line does not
/// create a clipped extra row (footer event lines, two-line labels).
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
        return fits_n_lines(geo, &lines[0], font_size, lines.len() + 1);
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

/// `wrap=none` on a wide center/justify frame makes hosts ignore `w:jc` /
/// `algn` and paint at `lIns` (lock-left of a page-width box). Turn wrap on
/// when leftover width is large enough that a host-wider line still fits.
/// Glyph-tight pills that already fill ≥85% stay `wrap=none` so they do not
/// clip a second row.
pub(crate) fn host_wrap(geo: Option<&GeometryNode>, align: TextAlign, wrap: bool) -> bool {
    if wrap {
        return true;
    }
    if !matches!(align, TextAlign::Center | TextAlign::Justify) {
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
    lines.iter().all(|line| {
        let (_, left, right) = line_gaps(line, box_w);
        left >= MIN_SLACK && right >= MIN_SLACK && !line_fills_padded_width(left, right, box_w)
    })
}

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
    }
}
