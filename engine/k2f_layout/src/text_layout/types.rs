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

/// Pushes a run into `out`, merging with the previous run when possible.
///
/// Two runs can be merged when they are adjacent in the original string (byte offsets touch)
/// and have an identical resolved `Style`.
pub(crate) fn push_or_merge_run(out: &mut Vec<TextRun>, run: TextRun) {
    if let Some(last) = out.last_mut() {
        if last.end == run.start
            && last.style == run.style
            && last.math_tex.is_none()
            && run.math_tex.is_none()
        {
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
