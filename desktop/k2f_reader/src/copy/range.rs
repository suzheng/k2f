//! Caret-range selection over the lock text layer, matching the web viewer.
//!
//! A drag is two carets in reading order, not a 2D rectangle. Horizontal
//! (same-y) drags select characters the way `user-select: text` does.

use k2f_paint::TextSpan;

/// Insertion point in a reading-order span list (`offset` is `0..=char_count`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Caret {
    pub order: usize,
    pub offset: usize,
}

/// Spans with text, top-then-left (same order as the web text layer).
pub fn reading_order(spans: &[TextSpan]) -> Vec<&TextSpan> {
    let mut hits: Vec<&TextSpan> = spans.iter().filter(|s| !s.text.is_empty()).collect();
    hits.sort_by(|a, b| a.y_pt.total_cmp(&b.y_pt).then(a.x_pt.total_cmp(&b.x_pt)));
    hits
}

pub fn span_contains(s: &TextSpan, x: f64, y: f64) -> bool {
    if !s.x_pt.is_finite()
        || !s.y_pt.is_finite()
        || !s.width_pt.is_finite()
        || !s.height_pt.is_finite()
        || s.width_pt <= 0.0
        || s.height_pt <= 0.0
        || !x.is_finite()
        || !y.is_finite()
    {
        return false;
    }
    x >= s.x_pt && x <= s.x_pt + s.width_pt && y >= s.y_pt && y <= s.y_pt + s.height_pt
}

pub fn y_overlaps(a: &TextSpan, b: &TextSpan) -> bool {
    let a1 = a.y_pt + a.height_pt;
    let b1 = b.y_pt + b.height_pt;
    a1 > b.y_pt && b1 > a.y_pt
}

/// Map a document-pt click to a caret. Misses still snap to the nearest line
/// so a drag that starts in the margin behaves like the web text layer.
pub fn point_to_caret(ordered: &[&TextSpan], x: f64, y: f64) -> Option<Caret> {
    if ordered.is_empty() || !x.is_finite() || !y.is_finite() {
        return None;
    }
    for (i, s) in ordered.iter().enumerate() {
        if span_contains(s, x, y) {
            return Some(Caret {
                order: i,
                offset: caret_offset_at_x(s, x),
            });
        }
    }
    if let Some(caret) = caret_on_line(ordered, x, y) {
        return Some(caret);
    }
    Some(caret_nearest_line(ordered, x, y))
}

/// Glyph-run slices between two document points. Collapsed → empty.
pub fn slices_at(spans: &[TextSpan], ax: f64, ay: f64, bx: f64, by: f64) -> Vec<TextSpan> {
    let ordered = reading_order(spans);
    let Some(a) = point_to_caret(&ordered, ax, ay) else {
        return Vec::new();
    };
    let Some(b) = point_to_caret(&ordered, bx, by) else {
        return Vec::new();
    };
    slice_between(&ordered, a, b)
}

pub fn slice_between(ordered: &[&TextSpan], a: Caret, b: Caret) -> Vec<TextSpan> {
    let (start, end) = if a <= b { (a, b) } else { (b, a) };
    if start == end {
        return Vec::new();
    }
    let mut out = Vec::new();
    for i in start.order..=end.order {
        let Some(s) = ordered.get(i) else {
            break;
        };
        let n = char_count(s);
        let from = if i == start.order {
            start.offset.min(n)
        } else {
            0
        };
        let to = if i == end.order { end.offset.min(n) } else { n };
        if from < to {
            out.push(slice_span(s, from, to));
        }
    }
    out
}

pub fn join_span_text(hits: &[&TextSpan]) -> String {
    let mut out = String::new();
    for (i, s) in hits.iter().enumerate() {
        if i > 0 && !y_overlaps(hits[i - 1], s) {
            out.push('\n');
        }
        out.push_str(&s.text);
    }
    out
}

fn char_count(s: &TextSpan) -> usize {
    s.text.chars().count()
}

fn caret_offset_at_x(s: &TextSpan, x: f64) -> usize {
    let n = char_count(s);
    if n == 0 || !(s.width_pt > 0.0) {
        return 0;
    }
    let t = ((x - s.x_pt) / s.width_pt).clamp(0.0, 1.0);
    ((t * n as f64).round() as usize).min(n)
}

fn line_indices(ordered: &[&TextSpan], probe: &TextSpan) -> Vec<usize> {
    let mut line: Vec<usize> = ordered
        .iter()
        .enumerate()
        .filter(|(_, s)| y_overlaps(probe, s))
        .map(|(i, _)| i)
        .collect();
    line.sort_by(|&a, &b| ordered[a].x_pt.total_cmp(&ordered[b].x_pt));
    line
}

fn caret_on_line(ordered: &[&TextSpan], x: f64, y: f64) -> Option<Caret> {
    let mut line: Vec<usize> = ordered
        .iter()
        .enumerate()
        .filter(|(_, s)| y >= s.y_pt && y <= s.y_pt + s.height_pt)
        .map(|(i, _)| i)
        .collect();
    if line.is_empty() {
        return None;
    }
    line.sort_by(|&a, &b| ordered[a].x_pt.total_cmp(&ordered[b].x_pt));
    Some(caret_along_line(ordered, &line, x))
}

fn caret_nearest_line(ordered: &[&TextSpan], x: f64, y: f64) -> Caret {
    let mut best_i = 0usize;
    let mut best_d = f64::INFINITY;
    for (i, s) in ordered.iter().enumerate() {
        let y0 = s.y_pt;
        let y1 = s.y_pt + s.height_pt;
        let d = if y < y0 {
            y0 - y
        } else if y > y1 {
            y - y1
        } else {
            0.0
        };
        if d < best_d {
            best_d = d;
            best_i = i;
        }
    }
    let line = line_indices(ordered, ordered[best_i]);
    let probe = ordered[best_i];
    if y < probe.y_pt {
        return Caret {
            order: line[0],
            offset: 0,
        };
    }
    if y > probe.y_pt + probe.height_pt {
        let last = *line.last().unwrap_or(&best_i);
        return Caret {
            order: last,
            offset: char_count(ordered[last]),
        };
    }
    caret_along_line(ordered, &line, x)
}

fn caret_along_line(ordered: &[&TextSpan], line: &[usize], x: f64) -> Caret {
    let first = line[0];
    let last = *line.last().unwrap_or(&first);
    if x <= ordered[first].x_pt {
        return Caret {
            order: first,
            offset: 0,
        };
    }
    if x >= ordered[last].x_pt + ordered[last].width_pt {
        return Caret {
            order: last,
            offset: char_count(ordered[last]),
        };
    }
    for w in line.windows(2) {
        let left = w[0];
        let right = w[1];
        let left_r = ordered[left].x_pt + ordered[left].width_pt;
        let right_l = ordered[right].x_pt;
        if x >= left_r && x <= right_l {
            let mid = (left_r + right_l) * 0.5;
            return if x < mid {
                Caret {
                    order: left,
                    offset: char_count(ordered[left]),
                }
            } else {
                Caret {
                    order: right,
                    offset: 0,
                }
            };
        }
    }
    nearest_on_line(ordered, line, x)
}

fn nearest_on_line(ordered: &[&TextSpan], line: &[usize], x: f64) -> Caret {
    let mut best = line[0];
    let mut best_d = f64::INFINITY;
    for &i in line {
        let s = ordered[i];
        let d = if x < s.x_pt {
            s.x_pt - x
        } else if x > s.x_pt + s.width_pt {
            x - (s.x_pt + s.width_pt)
        } else {
            0.0
        };
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    Caret {
        order: best,
        offset: caret_offset_at_x(ordered[best], x),
    }
}

fn slice_span(s: &TextSpan, from: usize, to: usize) -> TextSpan {
    let n = char_count(s);
    let from = from.min(n);
    let to = to.min(n).max(from);
    let text: String = s.text.chars().skip(from).take(to - from).collect();
    let t0 = if n == 0 { 0.0 } else { from as f64 / n as f64 };
    let t1 = if n == 0 { 0.0 } else { to as f64 / n as f64 };
    TextSpan {
        node_id: s.node_id.clone(),
        char_start: s.char_start + from,
        char_end: s.char_start + to,
        text,
        x_pt: s.x_pt + s.width_pt * t0,
        y_pt: s.y_pt,
        width_pt: s.width_pt * (t1 - t0),
        height_pt: s.height_pt,
    }
}
