use crate::style::Style;
use k2f_core::Pt;

/// A contiguous segment of text with a single resolved `Style`.
///
/// Offsets are byte offsets into the original UTF-8 string.
#[derive(Debug, Clone, PartialEq)]
pub struct TextRun {
    pub start: usize,
    pub end: usize,
    pub style: Style,
    pub text: String,
    /// TeX source when this run is an unsplittable inline math atom (U+FFFC).
    pub math_tex: Option<String>,
}

/// True when `push_or_merge_run` would concatenate `next` onto `prev`.
pub(crate) fn runs_merge(prev: &TextRun, next: &TextRun) -> bool {
    prev.end == next.start
        && prev.style == next.style
        && prev.math_tex.is_none()
        && next.math_tex.is_none()
}

/// Extra advance inserted between two wrap fragments that later merge into one shaped run.
///
/// Tracking is applied inside a shaped run (not after its last glyph). Measuring each
/// whitespace/word fragment separately would omit the join between fragments, so center/end
/// alignment and wrap decisions would use a width smaller than paint.
pub(crate) fn merge_join_tracking(prev: &TextRun, next: &TextRun) -> Pt {
    if runs_merge(prev, next) && !prev.text.is_empty() && !next.text.is_empty() {
        prev.style.letter_spacing
    } else {
        Pt::ZERO
    }
}

/// Pushes a run into `out`, merging with the previous run when possible.
///
/// Two runs can be merged when they are adjacent in the original string (byte offsets touch)
/// and have an identical resolved `Style`.
pub(crate) fn push_or_merge_run(out: &mut Vec<TextRun>, run: TextRun) {
    if let Some(last) = out.last_mut() {
        if runs_merge(last, &run) {
            last.end = run.end;
            last.text.push_str(&run.text);
            return;
        }
    }
    out.push(run);
}

/// One laid out line of text.
#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    pub runs: Vec<TextRun>,
    pub width: Pt,
    pub height: Pt,
}

/// The result of laying out a text node.
#[derive(Debug, Clone, PartialEq)]
pub struct TextLayout {
    pub lines: Vec<TextLine>,
    pub width: Pt,
    pub height: Pt,
}
