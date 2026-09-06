//! Text-layer selection → `text/plain` (+ optional K2F JSON).
//!
//! Interactive copy is a caret range (same as the web text layer). A 2D
//! drag-rect still intersects whole spans for lock-level tests. Coordinates
//! are document points, matching `OpenedDocument::text_layer`. Same-line
//! hits concatenate; stacked lines are separated by `\n`.

mod payload;
mod range;
mod rect;

pub use payload::{CopyFormat, CopyPayload, SelectedNode, K2F_NODES_MIME};
pub use range::{
    join_span_text, point_to_caret, reading_order, slice_between, slices_at, span_contains, Caret,
};
pub use rect::RectPt;

use k2f_paint::TextSpan;

/// Intersecting spans in reading order (top, then left). Empty text is skipped.
pub fn selected_spans(spans: &[TextSpan], sel: RectPt) -> Vec<&TextSpan> {
    reading_order(spans)
        .into_iter()
        .filter(|s| sel.intersects_span(s))
        .collect()
}

/// Concatenate intersecting spans in reading order (top, then left).
pub fn plain_text_from_spans(spans: &[TextSpan], sel: RectPt) -> String {
    join_span_text(&selected_spans(spans, sel))
}

/// Concatenate a caret-range slice (character-level, like the web viewer).
pub fn plain_text_from_points(spans: &[TextSpan], ax: f64, ay: f64, bx: f64, by: f64) -> String {
    let sliced = slices_at(spans, ax, ay, bx, by);
    let hits: Vec<&TextSpan> = sliced.iter().collect();
    join_span_text(&hits)
}
