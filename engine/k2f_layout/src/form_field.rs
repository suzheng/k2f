//! Reserved-size form fields: measure from declared metrics, shape value inside, clip overflow.

use crate::fixed_size::subtract_if_bounded;
use crate::resolved_style::{padding_for_role_variant, resolve_text_style};
use crate::shape_run::push_shaped_run;
use crate::text_layout::{layout_text, line_height_for_style};
use crate::{LayoutContext, Point, Size, SizeConstraint};
use k2f_core::{FormFieldKind, FormFieldSpec, GeometryNode, Pt, SemanticNode, TextGlyphRun};
use k2f_text::byte_to_char_index;

const CHECKBOX_MARK: &str = "X";

pub(crate) fn measure_form_field(
    spec: &FormFieldSpec,
    node: &SemanticNode,
    constraint: SizeConstraint,
    ctx: &LayoutContext,
) -> Result<Size, String> {
    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let style = resolve_text_style(&node.role, node.variant.as_deref(), ctx.theme);
    let line_h = line_height_for_style(&style);
    let font_size = style.font_size;
    let max_w = constraint.max.width;
    let max_h = constraint.max.height;

    let width = match spec.kind {
        FormFieldKind::Checkbox => cap_axis(spec.width.unwrap_or(font_size), max_w),
        _ => match spec.width {
            Some(w) => cap_axis(w, max_w),
            None => {
                if unbounded(max_w) {
                    return Err(format!(
                        "FORM_FIELD_UNBOUNDED_WIDTH: node '{}': form field needs an explicit width when the parent width is unbounded",
                        node.id
                    ));
                }
                max_w
            }
        },
    };

    let height = match spec.kind {
        FormFieldKind::Checkbox => cap_axis(spec.height.unwrap_or(font_size), max_h),
        _ => {
            let computed = spec.height.unwrap_or_else(|| {
                let lines = spec.lines.unwrap_or(default_lines(spec.kind)) as i128;
                line_h * lines + padding.vertical()
            });
            cap_axis(computed, max_h)
        }
    };

    Ok(constraint.constrain(Size::new(width, height)))
}

pub(crate) fn arrange_form_field(
    spec: &FormFieldSpec,
    node: &SemanticNode,
    position: Point,
    size: Size,
    ctx: &LayoutContext,
) -> Result<GeometryNode, String> {
    let mut glyphs = Vec::new();
    let mut text_runs = Vec::new();

    match spec.kind {
        FormFieldKind::Checkbox => {
            if spec.value == k2f_core::CHECKBOX_CHECKED {
                place_checkbox_mark(node, size, ctx, &mut glyphs, &mut text_runs)?;
            }
        }
        FormFieldKind::Text | FormFieldKind::Multiline => {
            if !spec.value.is_empty() {
                place_field_value(spec, node, size, ctx, &mut glyphs, &mut text_runs)?;
            }
        }
    }

    let (glyphs, text_runs) = clip_to_box(glyphs, text_runs, size);

    Ok(GeometryNode {
        id: node.id.clone(),
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        glyphs,
        text_runs,
        fill_rects: vec![],
        children: vec![],
    })
}

fn place_checkbox_mark(
    node: &SemanticNode,
    size: Size,
    ctx: &LayoutContext,
    glyphs: &mut Vec<k2f_core::GlyphPosition>,
    text_runs: &mut Vec<TextGlyphRun>,
) -> Result<(), String> {
    let style = resolve_text_style(&node.role, node.variant.as_deref(), ctx.theme);
    let mark_w = crate::text_layout::measure_text_run_width(CHECKBOX_MARK, &style, ctx)?;
    let x = Pt(((size.width.0 - mark_w.0).max(0)) / 2);
    let y = Pt(((size.height.0 - style.font_size.0).max(0)) / 2);
    push_shaped_run(
        glyphs,
        text_runs,
        CHECKBOX_MARK,
        &style,
        x,
        y,
        ctx,
        0,
        false,
        None,
    )?;
    Ok(())
}

fn place_field_value(
    spec: &FormFieldSpec,
    node: &SemanticNode,
    size: Size,
    ctx: &LayoutContext,
    glyphs: &mut Vec<k2f_core::GlyphPosition>,
    text_runs: &mut Vec<TextGlyphRun>,
) -> Result<(), String> {
    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let inner_w = subtract_if_bounded(size.width, padding.horizontal());
    let wrap = spec.kind == FormFieldKind::Multiline;
    let max_lines = spec.lines.unwrap_or(default_lines(spec.kind)) as usize;
    let wrap_w = if wrap { inner_w } else { Pt(i128::MAX) };
    let constraint = SizeConstraint::new(Size::ZERO, Size::new(wrap_w, Pt(i128::MAX)));
    let mut layout = layout_text(
        &spec.value,
        &node.role,
        node.variant.as_deref(),
        &[],
        constraint,
        ctx,
    )?;
    if layout.lines.len() > max_lines {
        layout.lines.truncate(max_lines);
    }

    let mut y_cursor = padding.top;
    for line in layout.lines {
        if y_cursor + line.height > size.height {
            break;
        }
        let mut x_cursor = padding.left;
        for run in line.runs {
            let adv = push_shaped_run(
                glyphs,
                text_runs,
                &run.text,
                &run.style,
                x_cursor,
                y_cursor,
                ctx,
                byte_to_char_index(&spec.value, run.start),
                true,
                None,
            )?;
            x_cursor += adv;
        }
        y_cursor += line.height;
    }
    Ok(())
}

fn clip_to_box(
    glyphs: Vec<k2f_core::GlyphPosition>,
    runs: Vec<TextGlyphRun>,
    size: Size,
) -> (Vec<k2f_core::GlyphPosition>, Vec<TextGlyphRun>) {
    let keep: Vec<bool> = glyphs
        .iter()
        .map(|g| {
            g.x_offset.0 >= 0
                && g.x_offset.0 + g.x_advance.0 <= size.width.0
                && g.y_offset.0 >= 0
                && g.y_offset.0 < size.height.0
        })
        .collect();
    if keep.iter().all(|k| *k) {
        return (glyphs, runs);
    }

    let mut new_glyphs = Vec::new();
    let mut old_to_new: Vec<Option<usize>> = vec![None; glyphs.len()];
    for (i, g) in glyphs.into_iter().enumerate() {
        if keep[i] {
            old_to_new[i] = Some(new_glyphs.len());
            new_glyphs.push(g);
        }
    }

    let mut new_runs = Vec::new();
    for run in runs {
        let [a, b] = run.glyph_range;
        let mut start = None;
        let mut end = None;
        for i in a..b {
            if let Some(ni) = old_to_new.get(i).copied().flatten() {
                if start.is_none() {
                    start = Some(ni);
                }
                end = Some(ni + 1);
            }
        }
        if let (Some(s), Some(e)) = (start, end) {
            new_runs.push(TextGlyphRun {
                glyph_range: [s, e],
                style: run.style,
            });
        }
    }
    (new_glyphs, new_runs)
}

fn default_lines(kind: FormFieldKind) -> u32 {
    match kind {
        FormFieldKind::Text => 1,
        FormFieldKind::Multiline => 3,
        FormFieldKind::Checkbox => 1,
    }
}

fn unbounded(v: Pt) -> bool {
    v.0 == i128::MAX
}

fn cap_axis(value: Pt, max: Pt) -> Pt {
    if unbounded(max) {
        value
    } else {
        Pt(value.0.min(max.0))
    }
}

/// Stretch would resize a reserved slot; form fields keep Start instead.
pub(crate) fn lock_stretch(node: &SemanticNode, align: k2f_core::Align) -> k2f_core::Align {
    if matches!(node.content, k2f_core::NodeContent::FormField(_))
        && align == k2f_core::Align::Stretch
    {
        k2f_core::Align::Start
    } else {
        align
    }
}
