//! Display-math measure/arrange: TeX in → glyphs + fill_rects out.

use crate::resolved_style::{padding_for_role_variant, resolve_text_style};
use crate::style::resolve_font_family_key;
use crate::text_align::line_start_offset;
use crate::{LayoutContext, Point, Size};
use k2f_core::{FillRect, GlyphPosition, Pt, SemanticNode, TextGlyphRun, TextPaintStyle};
use k2f_math::{layout_tex, MathLayout, MathStyle};
use k2f_text::Font;

pub(crate) fn layout_math_tex(
    tex: &str,
    node: &SemanticNode,
    ctx: &LayoutContext,
) -> Result<(MathLayout, MathPaintCtx), String> {
    let style = resolve_text_style(&node.role, node.variant.as_deref(), ctx.theme);
    let font_name = resolve_font_family_key(&style.font_family, ctx.theme);
    let font = ctx.fonts.get_font(&font_name).ok_or_else(|| {
        format!(
            "FONT_MISSING: math font '{font_name}' (from '{}') is not embedded",
            style.font_family
        )
    })?;
    let layout =
        layout_tex(tex, font, style.font_size, MathStyle::Display).map_err(|e| e.to_string())?;
    Ok((
        layout,
        MathPaintCtx {
            font_name,
            font_size: style.font_size,
            color: crate::render_plan::resolve_color_ref_for_plan(&style.color, ctx.theme),
            text_align: style.text_align,
        },
    ))
}

pub(crate) struct MathPaintCtx {
    pub font_name: String,
    pub font_size: Pt,
    pub color: String,
    pub text_align: crate::style::TextAlign,
}

pub(crate) fn measure_math(
    tex: &str,
    node: &SemanticNode,
    ctx: &LayoutContext,
) -> Result<Size, String> {
    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let (layout, _) = layout_math_tex(tex, node, ctx)?;
    // Horizontal overflow is allowed: do not clamp to the content-box max.
    Ok(Size::new(
        layout.width + padding.horizontal(),
        layout.height + padding.vertical(),
    ))
}

pub(crate) struct ArrangedMath {
    pub width: Pt,
    pub glyphs: Vec<GlyphPosition>,
    pub text_runs: Vec<TextGlyphRun>,
    pub fill_rects: Vec<FillRect>,
}

pub(crate) fn arrange_math(
    tex: &str,
    node: &SemanticNode,
    position: Point,
    size: Size,
    ctx: &LayoutContext,
) -> Result<ArrangedMath, String> {
    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let (layout, paint) = layout_math_tex(tex, node, ctx)?;
    let inner_w = crate::fixed_size::subtract_if_bounded(size.width, padding.horizontal());
    let origin_x = padding.left + line_start_offset(paint.text_align, inner_w, layout.width);
    let origin_y = padding.top;

    let font = ctx.fonts.get_font(&paint.font_name).ok_or_else(|| {
        format!(
            "FONT_MISSING: math font '{}' is not embedded",
            paint.font_name
        )
    })?;
    Ok(place_math_layout(
        &layout, font, &paint, position, origin_x, origin_y,
    ))
}

pub(crate) fn layout_inline_tex(
    tex: &str,
    font_size: Pt,
    ctx: &LayoutContext,
) -> Result<MathLayout, String> {
    let (_name, font) = math_font(ctx)?;
    k2f_math::layout_tex(tex, font, font_size, k2f_math::MathStyle::Text).map_err(|e| e.to_string())
}

pub(crate) fn place_inline_math(
    tex: &str,
    style: &crate::style::Style,
    origin_x: Pt,
    origin_y: Pt,
    position: Point,
    ctx: &LayoutContext,
) -> Result<ArrangedMath, String> {
    let (font_name, font) = math_font(ctx)?;
    let layout = k2f_math::layout_tex(tex, font, style.font_size, k2f_math::MathStyle::Text)
        .map_err(|e| e.to_string())?;
    let paint = MathPaintCtx {
        font_name,
        font_size: style.font_size,
        color: crate::render_plan::resolve_color_ref_for_plan(&style.color, ctx.theme),
        text_align: crate::style::TextAlign::Start,
    };
    Ok(place_math_layout(
        &layout, font, &paint, position, origin_x, origin_y,
    ))
}

fn math_font<'a>(ctx: &'a LayoutContext) -> Result<(String, &'a Font), String> {
    let family = ctx
        .theme
        .roles
        .get("math")
        .map(|r| r.font_family.as_str())
        .unwrap_or("NotoSansMath-Regular");
    let key = crate::style::resolve_font_family_key(family, ctx.theme);
    let font = ctx
        .fonts
        .get_font(&key)
        .or_else(|| ctx.fonts.get_font("default"))
        .ok_or_else(|| format!("FONT_MISSING: math font '{key}' is not embedded"))?;
    let name = if ctx.fonts.get_font(&key).is_some() {
        key
    } else {
        "default".to_string()
    };
    Ok((name, font))
}

fn place_math_layout(
    layout: &MathLayout,
    font: &Font,
    paint: &MathPaintCtx,
    position: Point,
    origin_x: Pt,
    origin_y: Pt,
) -> ArrangedMath {
    let max_size = layout
        .glyphs
        .iter()
        .map(|g| g.font_size)
        .max_by_key(|s| s.0)
        .unwrap_or(paint.font_size);
    let max_asc = font.typographic_ascender(max_size);

    let mut glyphs = Vec::with_capacity(layout.glyphs.len());
    for g in &layout.glyphs {
        glyphs.push(GlyphPosition {
            glyph_id: g.glyph_id,
            cluster: GlyphPosition::CLUSTER_NOT_SOURCE,
            x_offset: origin_x + g.x,
            y_offset: origin_y + g.y - max_asc,
            x_advance: Pt::ZERO,
            y_advance: Pt::ZERO,
        });
    }

    let mut text_runs = Vec::new();
    let mut run_start = 0usize;
    while run_start < glyphs.len() {
        let size = layout.glyphs[run_start].font_size;
        let mut run_end = run_start + 1;
        while run_end < glyphs.len() && layout.glyphs[run_end].font_size == size {
            run_end += 1;
        }
        text_runs.push(TextGlyphRun {
            glyph_range: [run_start, run_end],
            style: TextPaintStyle {
                font_family: paint.font_name.clone(),
                font_size: size,
                color: paint.color.clone(),
                bold: false,
                italic: false,
                strikethrough: false,
                underline: false,
            },
        });
        run_start = run_end;
    }

    let fill_rects = layout
        .fill_rects
        .iter()
        .map(|r| FillRect {
            x: position.x + origin_x + r.x,
            y: position.y + origin_y + r.y,
            width: r.width,
            height: r.height,
        })
        .collect();

    ArrangedMath {
        width: layout.width,
        glyphs,
        text_runs,
        fill_rects,
    }
}
