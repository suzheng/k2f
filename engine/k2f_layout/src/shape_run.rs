use crate::text_align::JustifyBudget;
use crate::LayoutContext;
use k2f_core::Pt;
use k2f_text::{split_by_coverage, TextShaper};

pub(crate) fn push_shaped_run(
    glyphs: &mut Vec<k2f_core::GlyphPosition>,
    text_runs: &mut Vec<k2f_core::TextGlyphRun>,
    text: &str,
    style: &crate::style::Style,
    x_cursor: Pt,
    y_cursor: Pt,
    ctx: &LayoutContext,
    char_origin: u32,
    from_source: bool,
    mut justify: Option<&mut JustifyBudget>,
) -> Result<Pt, String> {
    if text.is_empty() {
        return Ok(Pt::ZERO);
    }

    let primary = style.face_key(ctx.theme);
    let segments = split_by_coverage(text, &primary, ctx.fonts)?;
    let mut x_base = x_cursor;
    let mut char_off = 0u32;
    let mut total_advance = Pt::ZERO;

    for segment in segments {
        if segment.text.is_empty() {
            continue;
        }
        let mut seg_style = style.clone();
        seg_style.font_family = segment.font_key;
        let font_name = seg_style.face_key(ctx.theme);
        let font = ctx.fonts.get_font(&font_name).ok_or_else(|| {
            format!(
                "Font '{}' not loaded (resolved from '{}')",
                font_name, seg_style.font_family
            )
        })?;

        let mut run_glyphs = TextShaper::shape_text(&segment.text, font, style.font_size)?;
        crate::text_layout::apply_tracking(&mut run_glyphs, style.letter_spacing);
        let chars: Vec<char> = segment.text.chars().collect();
        let glyph_start = glyphs.len();
        let mut run_advance = Pt::ZERO;
        let mut shift = Pt::ZERO;
        for g in &mut run_glyphs {
            let ch = chars.get(g.cluster as usize).copied();
            let extra = if ch == Some(' ') {
                justify
                    .as_mut()
                    .map(|b| b.take_space_extra())
                    .unwrap_or(Pt::ZERO)
            } else {
                Pt::ZERO
            };
            run_advance += g.x_advance + extra;
            g.x_advance = g.x_advance + extra;
            g.x_offset = g.x_offset + x_base + shift;
            // baseline_shift > 0 raises (y grows downward).
            g.y_offset = g.y_offset + y_cursor - style.baseline_shift;
            shift += extra;
            g.cluster = if from_source {
                char_origin + char_off + g.cluster
            } else {
                k2f_core::GlyphPosition::CLUSTER_NOT_SOURCE
            };
        }
        glyphs.extend(run_glyphs);
        let glyph_end = glyphs.len();

        text_runs.push(k2f_core::TextGlyphRun {
            glyph_range: [glyph_start, glyph_end],
            style: k2f_core::TextPaintStyle {
                font_family: font_name,
                font_size: style.font_size,
                color: crate::render_plan::resolve_color_ref_for_plan(&style.color, ctx.theme),
                bold: style.bold,
                italic: style.italic,
                strikethrough: style.strikethrough,
                underline: style.underline,
            },
        });

        x_base += run_advance;
        total_advance += run_advance;
        char_off += chars.len() as u32;
    }

    Ok(total_advance)
}
