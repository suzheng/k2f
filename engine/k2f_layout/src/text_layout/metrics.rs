use crate::style::Style;
use crate::LayoutContext;
use k2f_core::Pt;
use k2f_text::TextShaper;

pub fn line_height_for_style(style: &Style) -> Pt {
    let base = style.font_size * style.line_height_mult / 1000;
    let shift_abs = if style.baseline_shift.0 < 0 {
        Pt(-style.baseline_shift.0)
    } else {
        style.baseline_shift
    };
    base + shift_abs
}

pub fn measure_text_run_width(
    text: &str,
    style: &Style,
    ctx: &LayoutContext,
) -> Result<Pt, String> {
    let font_key = crate::style::resolve_font_family_key(&style.font_family, ctx.theme);
    let font = ctx.fonts.get_font(&font_key).ok_or_else(|| {
        format!(
            "Font '{}' not loaded (resolved from '{}')",
            font_key, style.font_family
        )
    })?;

    let mut glyphs = TextShaper::shape_text(text, font, style.font_size)?;
    super::tracking::apply_tracking(&mut glyphs, style.letter_spacing);
    let mut width = Pt::ZERO;
    for g in glyphs {
        width += g.x_advance;
    }
    Ok(width)
}
