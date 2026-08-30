use crate::layout_context::SizeConstraint;
use crate::LayoutContext;
use k2f_core::{Modifier, Pt};

use super::font_runs;
use super::metrics::line_height_for_style;
use super::run_split::split_runs;
use super::types::{TextLayout, TextLine};
use super::wrap::wrap_runs;

/// Text layout: run splitting + deterministic wrapping.
///
/// - Uses the role base style from `Theme`.
/// - Applies `modifiers`: splits text into deterministic styled runs.
/// - Applies deterministic wrapping: hard newlines and word-boundary wrapping.
pub fn layout_text(
    text: &str,
    role: &str,
    variant: Option<&str>,
    modifiers: &[Modifier],
    constraint: SizeConstraint,
    ctx: &LayoutContext,
) -> Result<TextLayout, String> {
    // Split runs by modifiers (range-based styling), then by embedded-font coverage.
    let styled = split_runs(text, role, variant, modifiers, ctx.theme)?;
    let runs = font_runs::split_font_runs(styled, ctx)?;

    let base_style = crate::resolved_style::resolve_text_style(role, variant, ctx.theme);
    let base_line_height = line_height_for_style(&base_style);
    let first_line_indent = base_style.first_line_indent;

    // Wrap into lines (hard newlines always apply; width constraint applies when finite).
    let max_width = constraint.max.width;
    let lines: Vec<TextLine> = wrap_runs(
        &runs,
        max_width,
        base_line_height,
        first_line_indent,
        ctx,
    )?;

    // Compute layout width/height from wrapped lines.
    let mut width = Pt::ZERO;
    let mut height = Pt::ZERO;
    for (idx, line) in lines.iter().enumerate() {
        let line_width = if idx == 0 {
            line.width + first_line_indent
        } else {
            line.width
        };
        if line_width > width {
            width = line_width;
        }
        height += line.height;
    }

    Ok(TextLayout {
        lines,
        width,
        height,
    })
}
