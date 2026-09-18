use crate::style::Style;
use crate::LayoutContext;
use k2f_core::Pt;
use k2f_text::{split_by_coverage, TextShaper};

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
    if text.is_empty() {
        return Ok(Pt::ZERO);
    }
    let primary = style.face_key(ctx.theme);
    let segments = split_by_coverage(text, &primary, ctx.fonts)?;
    let mut width = Pt::ZERO;
    for segment in segments {
        let mut seg_style = style.clone();
        seg_style.font_family = segment.font_key;
        let font_key = seg_style.face_key(ctx.theme);
        let font = ctx.fonts.get_font(&font_key).ok_or_else(|| {
            format!(
                "Font '{}' not loaded (resolved from '{}')",
                font_key, seg_style.font_family
            )
        })?;
        let mut glyphs = TextShaper::shape_text(&segment.text, font, style.font_size)?;
        super::tracking::apply_tracking(&mut glyphs, style.letter_spacing);
        for g in glyphs {
            width += g.x_advance;
        }
    }
    Ok(width)
}
