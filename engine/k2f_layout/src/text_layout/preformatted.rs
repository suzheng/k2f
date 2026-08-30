use crate::layout_context::SizeConstraint;
use crate::LayoutContext;
use k2f_core::{Modifier, Pt};

use super::metrics::line_height_for_style;
use super::run_split::split_runs;
use super::types::{push_or_merge_run, TextLayout, TextLine, TextRun};
use super::wrap_tokenize::{split_run_for_wrapping, FragKind};

/// Preformatted text layout for semantic code blocks.
///
/// Contract:
/// - Hard newlines only (no soft wrapping).
/// - Preserve leading and trailing whitespace on each line (no trimming).
/// - Preserve blank lines (consecutive newlines).
pub fn layout_code_block(
    text: &str,
    role: &str,
    variant: Option<&str>,
    modifiers: &[Modifier],
    _constraint: SizeConstraint,
    ctx: &LayoutContext,
) -> Result<TextLayout, String> {
    // Split runs by modifiers (range-based styling).
    let runs: Vec<TextRun> = split_runs(text, role, variant, modifiers, ctx.theme)?;

    let base_style = crate::resolved_style::resolve_text_style(role, variant, ctx.theme);
    let base_line_height = line_height_for_style(&base_style);

    // Preformatted wrapping: hard newlines only, no trimming, no width-based wrapping.
    let lines: Vec<TextLine> = wrap_runs_preformatted(&runs, base_line_height, ctx)?;

    // Compute layout width/height from lines.
    let mut width = Pt::ZERO;
    let mut height = Pt::ZERO;
    for line in &lines {
        if line.width > width {
            width = line.width;
        }
        height += line.height;
    }

    Ok(TextLayout {
        lines,
        width,
        height,
    })
}

fn wrap_runs_preformatted(
    runs: &[TextRun],
    base_line_height: Pt,
    ctx: &LayoutContext,
) -> Result<Vec<TextLine>, String> {
    let mut out_lines: Vec<TextLine> = Vec::new();

    let mut line_runs: Vec<TextRun> = Vec::new();
    let mut line_width = Pt::ZERO;
    let mut line_height = base_line_height;

    for run in runs {
        let frags = split_run_for_wrapping(run);
        for frag in frags {
            match frag.kind {
                FragKind::Newline => {
                    flush_line_preformatted(
                        &mut out_lines,
                        &mut line_runs,
                        &mut line_width,
                        &mut line_height,
                        base_line_height,
                    );
                }
                FragKind::Whitespace | FragKind::Text => {
                    let r = frag.run.expect("run present for text/whitespace");
                    let w = super::metrics::measure_text_run_width(&r.text, &r.style, ctx)?;
                    let h = line_height_for_style(&r.style).max(base_line_height);
                    line_width += w;
                    if h > line_height {
                        line_height = h;
                    }
                    push_or_merge_run(&mut line_runs, r);
                }
                FragKind::Atom => {
                    let r = frag.run.expect("run present for atom");
                    let tex = r.math_tex.as_deref().unwrap_or("");
                    let math = crate::math::layout_inline_tex(tex, r.style.font_size, ctx)?;
                    let h = math.height.max(base_line_height);
                    line_width += math.width;
                    if h > line_height {
                        line_height = h;
                    }
                    push_or_merge_run(&mut line_runs, r);
                }
            }
        }
    }

    // Flush final line (even if empty, to preserve trailing newline behavior deterministically).
    flush_line_preformatted(
        &mut out_lines,
        &mut line_runs,
        &mut line_width,
        &mut line_height,
        base_line_height,
    );

    Ok(out_lines)
}

fn flush_line_preformatted(
    out_lines: &mut Vec<TextLine>,
    line_runs: &mut Vec<TextRun>,
    line_width: &mut Pt,
    line_height: &mut Pt,
    base_line_height: Pt,
) {
    let width = *line_width;
    let height = if line_runs.is_empty() {
        base_line_height
    } else {
        (*line_height).max(base_line_height)
    };

    // Note: no leading/trailing whitespace trimming; runs are verbatim.
    out_lines.push(TextLine {
        runs: std::mem::take(line_runs),
        width,
        height,
    });

    *line_width = Pt::ZERO;
    *line_height = base_line_height;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LayoutContext, Size, SizeConstraint, Theme};

    #[test]
    fn code_block_preserves_leading_and_trailing_whitespace_per_line() {
        let fonts = crate::test_utils::test_fonts();
        let theme = Theme::default();
        let ctx = LayoutContext::new(&fonts, &theme);

        let text = "    let x = 1;   \nnext";
        let modifiers: Vec<Modifier> = vec![];
        let constraint = SizeConstraint::new(Size::ZERO, Size::new(Pt(10_000), Pt(i128::MAX)));

        let layout =
            layout_code_block(text, "code_block", None, &modifiers, constraint, &ctx).unwrap();
        assert_eq!(layout.lines.len(), 2);

        let line0_text: String = layout.lines[0]
            .runs
            .iter()
            .map(|r| r.text.as_str())
            .collect();
        assert!(line0_text.starts_with("    "));
        assert!(line0_text.ends_with("   "));
    }

    #[test]
    fn code_block_preserves_blank_lines() {
        let fonts = crate::test_utils::test_fonts();
        let theme = Theme::default();
        let ctx = LayoutContext::new(&fonts, &theme);

        let text = "a\n\nb";
        let modifiers: Vec<Modifier> = vec![];
        let constraint = SizeConstraint::infinite();

        let layout =
            layout_code_block(text, "code_block", None, &modifiers, constraint, &ctx).unwrap();
        assert_eq!(layout.lines.len(), 3);
        assert_eq!(layout.lines[1].runs.len(), 0);
        assert_eq!(layout.lines[1].width, Pt::ZERO);
        assert!(layout.lines[1].height > Pt::ZERO);
    }

    #[test]
    fn code_block_never_soft_wraps_long_lines() {
        let fonts = crate::test_utils::test_fonts();
        let theme = Theme::default();
        let ctx = LayoutContext::new(&fonts, &theme);

        let text = "this_is_a_single_very_long_line_without_newlines";
        let modifiers: Vec<Modifier> = vec![];
        let constraint = SizeConstraint::new(Size::ZERO, Size::new(Pt(1_000), Pt(i128::MAX)));

        let layout =
            layout_code_block(text, "code_block", None, &modifiers, constraint, &ctx).unwrap();
        assert_eq!(layout.lines.len(), 1);
    }
}
