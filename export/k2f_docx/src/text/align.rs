use crate::ir::TextAlign;
use k2f_core::{GeometryNode, GlyphPosition};

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

pub(crate) fn source_lines(geo: &GeometryNode) -> Vec<Vec<&GlyphPosition>> {
    let mut glyphs = source_glyphs(geo);
    if glyphs.is_empty() {
        return Vec::new();
    }
    glyphs.sort_by_key(|g| (g.y_offset.0, g.x_offset.0));

    let mut lines: Vec<Vec<&GlyphPosition>> = Vec::new();
    for g in glyphs {
        if let Some(line) = lines.iter_mut().find(|line| {
            line.iter()
                .any(|existing| (existing.y_offset.0 - g.y_offset.0).abs() <= 7_500)
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

/// Office wrap on a glyph-tight lock box reflows the last word onto a clipped
/// second line when host bold/metrics are wider than rustybuzz. Keep wrap only
/// when the lock already wrapped or left leftover width in the frame.
pub(crate) fn should_wrap_lock(geo: Option<&GeometryNode>) -> bool {
    let Some(geo) = geo else {
        return true;
    };
    let lines = source_lines(geo);
    if lines.len() >= 2 {
        return true;
    };
    let Some(line) = lines.first() else {
        return true;
    };
    let (slack, _, _) = line_gaps(line, geo.width.0);
    slack >= MIN_SLACK
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

fn infer_from_gaps(left: i128, right: i128, box_w: i128) -> TextAlign {
    let slack = left.saturating_add(right);
    if slack < MIN_SLACK {
        return TextAlign::Left;
    }
    let delta = left - right;
    if delta.abs() * 5 <= slack && left * 10 > box_w {
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
    fn wrap_only_when_lock_has_slack_or_multiple_lines() {
        let tight = geo(40_000, vec![glyph(0, 0, 40_000, 0)]);
        assert!(!should_wrap_lock(Some(&tight)));
        let slack = geo(100_000, vec![glyph(0, 0, 40_000, 0)]);
        assert!(should_wrap_lock(Some(&slack)));
        let wrapped = geo(
            40_000,
            vec![glyph(0, 0, 40_000, 0), glyph(1, 0, 20_000, 14_000)],
        );
        assert!(should_wrap_lock(Some(&wrapped)));
        assert!(should_wrap_lock(None));
    }
}
