use crate::text_layout::{layout_text, TextLayout};
use crate::{arrange_node, LayoutContext, Point, Size, SizeConstraint};
use k2f_core::{GeometryNode, NodeContent, Pt, SemanticNode};
use k2f_text::byte_to_char_index;

use super::super::arrange::{append_placed_math, push_shaped_run};
use super::super::fixed_size::subtract_if_bounded;
use super::super::resolved_style::padding_for_role_variant;
use super::super::text_align::{
    count_justify_spaces, justify_space_extras, line_start_offset, JustifyBudget,
};

pub(crate) struct TextPages {
    pub layout: TextLayout,
    pub padding_top: Pt,
    pub padding_bottom: Pt,
    pub padding_left: Pt,
    pub inner_w: Pt,
    pub outer_w: Pt,
    pub first_line_indent: Pt,
}

pub(crate) fn prepare_text(
    node: &SemanticNode,
    content_width: Pt,
    ctx: &LayoutContext,
) -> Result<TextPages, String> {
    let NodeContent::Text(text) = &node.content else {
        return Err(format!("prepare_text on non-text '{}'", node.id));
    };
    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let inner_w = subtract_if_bounded(content_width, padding.horizontal());
    let constraint = SizeConstraint::new(Size::ZERO, Size::new(inner_w, Pt(i128::MAX)));
    let layout = layout_text(
        text,
        &node.role,
        node.variant.as_deref(),
        &node.modifiers,
        constraint,
        ctx,
    )?;
    let first_line_indent =
        crate::resolved_style::resolve_text_style(&node.role, node.variant.as_deref(), ctx.theme)
            .first_line_indent;
    Ok(TextPages {
        layout,
        padding_top: padding.top,
        padding_bottom: padding.bottom,
        padding_left: padding.left,
        inner_w,
        outer_w: content_width,
        first_line_indent,
    })
}

pub(crate) fn fragment_height(prep: &TextPages, start: usize, end: usize) -> Pt {
    let mut h = Pt::ZERO;
    for line in &prep.layout.lines[start..end] {
        h += line.height;
    }
    if start == 0 {
        h += prep.padding_top;
    }
    if end == prep.layout.lines.len() {
        h += prep.padding_bottom;
    }
    h
}

pub(crate) fn first_line_height(prep: &TextPages) -> Pt {
    if prep.layout.lines.is_empty() {
        prep.padding_top + prep.padding_bottom
    } else {
        fragment_height(prep, 0, 1)
    }
}

pub(crate) fn arrange_fragment(
    node: &SemanticNode,
    prep: &TextPages,
    start: usize,
    end: usize,
    pos: Point,
    width: Pt,
    ctx: &LayoutContext,
) -> Result<GeometryNode, String> {
    let height = fragment_height(prep, start, end);
    let text_align =
        crate::resolved_style::resolve_text_style(&node.role, node.variant.as_deref(), ctx.theme)
            .text_align;
    let mut glyphs = vec![];
    let mut text_runs = vec![];
    let mut fill_rects = vec![];
    let mut y_cursor = if start == 0 {
        prep.padding_top
    } else {
        Pt::ZERO
    };
    let source = match &node.content {
        NodeContent::Text(t) => t.as_str(),
        _ => "",
    };
    let total_lines = prep.layout.lines.len();
    for (frag_i, line) in prep.layout.lines[start..end].iter().enumerate() {
        let line_idx = start + frag_i;
        let is_last = line_idx + 1 == total_lines;
        let is_first_in_node = line_idx == 0;
        let line_inner_w = if is_first_in_node && prep.first_line_indent.0 > 0 {
            (prep.inner_w - prep.first_line_indent).max(Pt::ZERO)
        } else {
            prep.inner_w
        };
        let space_count = count_justify_spaces(line.runs.iter().filter_map(|r| {
            if r.math_tex.is_some() {
                None
            } else {
                Some(r.text.as_str())
            }
        }));
        let (extra_per, rem) =
            justify_space_extras(text_align, line_inner_w, line.width, space_count, is_last);
        let mut budget = JustifyBudget { extra_per, rem };
        let mut x_cursor = prep.padding_left
            + if is_first_in_node {
                prep.first_line_indent
            } else {
                Pt::ZERO
            }
            + line_start_offset(text_align, line_inner_w, line.width);
        for run in &line.runs {
            if let Some(tex) = &run.math_tex {
                let placed =
                    crate::math::place_inline_math(tex, &run.style, x_cursor, y_cursor, pos, ctx)?;
                x_cursor += placed.width;
                append_placed_math(&mut glyphs, &mut text_runs, &mut fill_rects, placed);
            } else {
                let adv = push_shaped_run(
                    &mut glyphs,
                    &mut text_runs,
                    &run.text,
                    &run.style,
                    x_cursor,
                    y_cursor,
                    ctx,
                    byte_to_char_index(source, run.start),
                    true,
                    Some(&mut budget),
                )?;
                x_cursor += adv;
            }
        }
        y_cursor += line.height;
    }
    Ok(GeometryNode {
        id: node.id.clone(),
        x: pos.x,
        y: pos.y,
        width,
        height,
        glyphs,
        text_runs,
        fill_rects,
        children: vec![],
    })
}

pub(crate) fn measure_full_height(
    node: &SemanticNode,
    content_width: Pt,
    ctx: &LayoutContext,
) -> Result<Pt, String> {
    let constraint = SizeConstraint::new(Size::ZERO, Size::new(content_width, Pt(i128::MAX)));
    Ok(crate::measure_node(node, constraint, ctx)?.height)
}

pub(crate) fn arrange_whole(
    node: &SemanticNode,
    pos: Point,
    width: Pt,
    height: Pt,
    ctx: &LayoutContext,
) -> Result<GeometryNode, String> {
    arrange_node(node, pos, Size::new(width, height), ctx)
}
