//! Drag-rect selection over `TextSpan` → `text/plain` (+ optional K2F JSON).
//!
//! Coordinates are document points (not window pixels), matching
//! `OpenedDocument::text_layer`. Same-line hits concatenate; stacked lines
//! are separated by `\n` so a page drag stays readable.

mod payload;
mod rect;

pub use payload::{CopyFormat, CopyPayload, SelectedNode, K2F_NODES_MIME};
pub use rect::RectPt;

use k2f_paint::TextSpan;

/// Intersecting spans in reading order (top, then left). Empty text is skipped.
pub fn selected_spans(spans: &[TextSpan], sel: RectPt) -> Vec<&TextSpan> {
    let mut hits: Vec<&TextSpan> = spans
        .iter()
        .filter(|s| !s.text.is_empty() && sel.intersects_span(s))
        .collect();
    hits.sort_by(|a, b| a.y_pt.total_cmp(&b.y_pt).then(a.x_pt.total_cmp(&b.x_pt)));
    hits
}

fn y_overlaps(a: &TextSpan, b: &TextSpan) -> bool {
    let a1 = a.y_pt + a.height_pt;
    let b1 = b.y_pt + b.height_pt;
    a1 > b.y_pt && b1 > a.y_pt
}

/// Concatenate intersecting spans in reading order (top, then left).
pub fn plain_text_from_spans(spans: &[TextSpan], sel: RectPt) -> String {
    let hits = selected_spans(spans, sel);
    let mut out = String::new();
    for (i, s) in hits.iter().enumerate() {
        if i > 0 && !y_overlaps(hits[i - 1], s) {
            out.push('\n');
        }
        out.push_str(&s.text);
    }
    out
}
