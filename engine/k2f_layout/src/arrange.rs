use crate::alignment::{align_offset_and_size, justify_offset};
use crate::fixed_size::{cap_inner_size_by_fixed, fixed_size_hint, subtract_if_bounded};
use crate::grid::{resolve_tracks, sum_prefix};
use crate::list_item_measure::list_item_measure_spec;
use crate::resolved_style::resolve_list_style;
use crate::resolved_style::{padding_for_role_variant, resolve_self_align};
use crate::text_align::{
    count_justify_spaces, justify_space_extras, line_start_offset, JustifyBudget,
};
use crate::text_layout::layout_code_block;
use crate::text_layout::layout_text;
use crate::{measure_node, LayoutContext, Point, Size, SizeConstraint};
use k2f_core::{
    GeometryNode, LayoutHint, ListMarkerType, NodeContent, Pt, SemanticNode, StackDirection,
};
use k2f_text::{byte_to_char_index, TextShaper};

pub fn arrange_node(
    node: &SemanticNode,
    position: Point,
    // The size is passed down from the parent (after measurement pass)
    // In a real 2-pass system, we'd probably have a specific LayoutTree or look up the size
    // For this simple implementation, we might re-measure or assume correct size is passed.
    // However, typically Arrange takes the negotiated size.
    size: Size,
    ctx: &LayoutContext,
) -> Result<GeometryNode, String> {
    match &node.content {
        NodeContent::Text(_) => arrange_text(node, position, size, ctx),
        NodeContent::CodeBlock(code) => arrange_code_block(node, code, position, size, ctx),
        NodeContent::Math(tex) => arrange_math(node, tex, position, size, ctx),
        NodeContent::Image { .. } => Ok(arrange_leaf(node, position, size)),
        NodeContent::Container { children } => {
            arrange_container(node, children, position, size, ctx)
        }
        NodeContent::Table(spec) => crate::table::arrange_table(node, spec, position, size, ctx),
        NodeContent::TableReference { .. } => Ok(arrange_leaf(node, position, size)),
    }
}

fn arrange_math(
    node: &SemanticNode,
    tex: &str,
    position: Point,
    size: Size,
    ctx: &LayoutContext,
) -> Result<GeometryNode, String> {
    let placed = crate::math::arrange_math(tex, node, position, size, ctx)?;
    Ok(GeometryNode {
        id: node.id.clone(),
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        glyphs: placed.glyphs,
        text_runs: placed.text_runs,
        fill_rects: placed.fill_rects,
        children: vec![],
    })
}

fn arrange_code_block(
    node: &SemanticNode,
    code: &k2f_core::CodeBlockValue,
    position: Point,
    size: Size,
    ctx: &LayoutContext,
) -> Result<GeometryNode, String> {
    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let inner_w = subtract_if_bounded(size.width, padding.horizontal());

    let text_align =
        crate::resolved_style::resolve_text_style(&node.role, node.variant.as_deref(), ctx.theme)
            .text_align;

    let canonical = code.to_canonical_text();
    let constraint = SizeConstraint::new(Size::ZERO, Size::new(inner_w, Pt(i128::MAX)));
    let layout = layout_code_block(
        canonical.as_ref(),
        &node.role,
        node.variant.as_deref(),
        &node.modifiers,
        constraint,
        ctx,
    )?;

    let mut glyphs = vec![];
    let mut text_runs: Vec<k2f_core::TextGlyphRun> = vec![];

    // Code blocks are top-aligned within their padded inner box.
    let mut y_cursor = padding.top;
    for line in layout.lines {
        let mut x_cursor = padding.left + line_start_offset(text_align, inner_w, line.width);
        for run in line.runs {
            let adv = push_shaped_run(
                &mut glyphs,
                &mut text_runs,
                &run.text,
                &run.style,
                x_cursor,
                y_cursor,
                ctx,
                byte_to_char_index(canonical.as_ref(), run.start),
                true,
                None,
            )?;
            x_cursor += adv;
        }
        y_cursor += line.height;
    }

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

fn arrange_text(
    node: &SemanticNode,
    position: Point,
    size: Size,
    ctx: &LayoutContext,
) -> Result<GeometryNode, String> {
    if node.role == "list_item" {
        return arrange_list_item_text(node, position, size, ctx);
    }

    let mut glyphs = vec![];
    let mut text_runs: Vec<k2f_core::TextGlyphRun> = vec![];
    let mut fill_rects = vec![];
    if let NodeContent::Text(text) = &node.content {
        // Apply role/variant padding even for leaf text nodes.
        // This enables table cells and other decorated text blocks to have inner padding.
        let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
        let inner_w = subtract_if_bounded(size.width, padding.horizontal());
        let inner_h = subtract_if_bounded(size.height, padding.vertical());

        let text_align = crate::resolved_style::resolve_text_style(
            &node.role,
            node.variant.as_deref(),
            ctx.theme,
        )
        .text_align;
        let first_line_indent = crate::resolved_style::resolve_text_style(
            &node.role,
            node.variant.as_deref(),
            ctx.theme,
        )
        .first_line_indent;

        // Use size.width as wrap constraint for deterministic glyph layout.
        let constraint = SizeConstraint::new(Size::ZERO, Size::new(inner_w, Pt(i128::MAX)));
        let layout = if node.role == "code" {
            layout_code_block(
                text,
                &node.role,
                node.variant.as_deref(),
                &node.modifiers,
                constraint,
                ctx,
            )?
        } else {
            layout_text(
                text,
                &node.role,
                node.variant.as_deref(),
                &node.modifiers,
                constraint,
                ctx,
            )?
        };

        // Vertically center the laid-out text within the padded inner box (best-effort).
        let mut y_cursor = padding.top;
        if inner_h.0 != i128::MAX && layout.height.0 < inner_h.0 {
            let free = inner_h.0 - layout.height.0;
            y_cursor += Pt(free / 2);
        }
        let line_count = layout.lines.len();
        for (line_idx, line) in layout.lines.into_iter().enumerate() {
            let is_last = line_idx + 1 == line_count;
            let is_first = line_idx == 0;
            let line_inner_w = if is_first && first_line_indent.0 > 0 {
                (inner_w - first_line_indent).max(Pt::ZERO)
            } else {
                inner_w
            };
            let space_count =
                count_justify_spaces(line.runs.iter().filter_map(|r| {
                    if r.math_tex.is_some() {
                        None
                    } else {
                        Some(r.text.as_str())
                    }
                }));
            let (extra_per, rem) =
                justify_space_extras(text_align, line_inner_w, line.width, space_count, is_last);
            let mut budget = JustifyBudget { extra_per, rem };
            let mut x_cursor = padding.left
                + if is_first {
                    first_line_indent
                } else {
                    Pt::ZERO
                }
                + line_start_offset(text_align, line_inner_w, line.width);
            for run in line.runs {
                if let Some(tex) = &run.math_tex {
                    let placed = crate::math::place_inline_math(
                        tex, &run.style, x_cursor, y_cursor, position, ctx,
                    )?;
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
                        byte_to_char_index(text, run.start),
                        true,
                        Some(&mut budget),
                    )?;
                    x_cursor += adv;
                }
            }
            y_cursor += line.height;
        }
    }

    Ok(GeometryNode {
        id: node.id.clone(),
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        glyphs,
        text_runs,
        fill_rects,
        children: vec![],
    })
}

fn arrange_list_item_text(
    node: &SemanticNode,
    position: Point,
    size: Size,
    ctx: &LayoutContext,
) -> Result<GeometryNode, String> {
    let NodeContent::Text(text) = &node.content else {
        return Err("arrange_list_item_text called for non-text node".to_string());
    };

    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let inner_w = subtract_if_bounded(size.width, padding.horizontal());
    let inner_h = subtract_if_bounded(size.height, padding.vertical());

    let base_style =
        crate::resolved_style::resolve_text_style(&node.role, node.variant.as_deref(), ctx.theme);
    let text_align = base_style.text_align;

    let list_style = resolve_list_style(&node.role, node.variant.as_deref(), ctx.theme)
        .ok_or_else(|| {
            format!(
            "Missing list_style for role '{}' (required to arrange list items deterministically)",
            node.role
        )
        })?;

    let marker_align = list_style.marker_align.unwrap_or_else(|| {
        match node.marker_type.unwrap_or(ListMarkerType::Bullet) {
            ListMarkerType::Number => crate::style::TextAlign::End,
            ListMarkerType::Bullet => crate::style::TextAlign::Center,
        }
    });

    let spec = list_item_measure_spec(node, ctx)?;
    let leading = spec.leading_width();
    let wrap_w = subtract_if_bounded(inner_w, leading);

    let constraint = SizeConstraint::new(Size::ZERO, Size::new(wrap_w, Pt(i128::MAX)));
    let layout = layout_text(
        text,
        &node.role,
        node.variant.as_deref(),
        &node.modifiers,
        constraint,
        ctx,
    )?;

    let mut glyphs = vec![];
    let mut text_runs: Vec<k2f_core::TextGlyphRun> = vec![];
    let mut fill_rects = vec![];

    // Best-effort vertical centering inside the padded inner box, matching non-list text behavior.
    let mut y_cursor = padding.top;
    if inner_h.0 != i128::MAX && layout.height.0 < inner_h.0 {
        let free = inner_h.0 - layout.height.0;
        y_cursor += Pt(free / 2);
    }

    // Emit marker glyphs aligned to the first line.
    let marker_label = ctx
        .list_markers
        .get(&node.id)
        .ok_or_else(|| format!("Missing derived marker label for list_item '{}'", node.id))?;

    let marker_box_x = padding.left + spec.indent;
    let marker_x = marker_box_x
        + marker_label_offset(
            marker_align,
            spec.marker_box_width,
            marker_label,
            &base_style,
            ctx,
        )?;
    push_shaped_run(
        &mut glyphs,
        &mut text_runs,
        marker_label,
        &base_style,
        marker_x,
        y_cursor,
        ctx,
        0,
        false,
        None,
    )?;

    // Emit body glyphs with hanging indent; marker label never affects wrap width.
    let line_count = layout.lines.len();
    for (line_idx, line) in layout.lines.into_iter().enumerate() {
        let is_last = line_idx + 1 == line_count;
        let space_count = count_justify_spaces(line.runs.iter().filter_map(|r| {
            if r.math_tex.is_some() {
                None
            } else {
                Some(r.text.as_str())
            }
        }));
        let (extra_per, rem) =
            justify_space_extras(text_align, wrap_w, line.width, space_count, is_last);
        let mut budget = JustifyBudget { extra_per, rem };
        let mut x_cursor = padding.left
            + spec.indent
            + spec.marker_box_width
            + spec.marker_gap
            + line_start_offset(text_align, wrap_w, line.width);
        for run in line.runs {
            if let Some(tex) = &run.math_tex {
                let placed = crate::math::place_inline_math(
                    tex, &run.style, x_cursor, y_cursor, position, ctx,
                )?;
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
                    byte_to_char_index(text, run.start),
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
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        glyphs,
        text_runs,
        fill_rects,
        children: vec![],
    })
}

pub(crate) fn append_placed_math(
    glyphs: &mut Vec<k2f_core::GlyphPosition>,
    text_runs: &mut Vec<k2f_core::TextGlyphRun>,
    fill_rects: &mut Vec<k2f_core::FillRect>,
    placed: crate::math::ArrangedMath,
) {
    let glyph_start = glyphs.len();
    glyphs.extend(placed.glyphs);
    for mut run in placed.text_runs {
        run.glyph_range[0] += glyph_start;
        run.glyph_range[1] += glyph_start;
        text_runs.push(run);
    }
    fill_rects.extend(placed.fill_rects);
}

fn marker_label_offset(
    align: crate::style::TextAlign,
    marker_box_width: Pt,
    label: &str,
    style: &crate::style::Style,
    ctx: &LayoutContext,
) -> Result<Pt, String> {
    let font_name = crate::style::resolve_font_family_key(&style.font_family, ctx.theme);
    let font = ctx.fonts.get_font(&font_name).ok_or_else(|| {
        format!(
            "Font '{}' not loaded (resolved from '{}')",
            font_name, style.font_family
        )
    })?;
    let mut glyphs = TextShaper::shape_text(label, font, style.font_size)?;
    crate::text_layout::apply_tracking(&mut glyphs, style.letter_spacing);
    let mut width = Pt::ZERO;
    for g in &glyphs {
        width += g.x_advance;
    }
    Ok(line_start_offset(align, marker_box_width, width))
}

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
    mut justify: Option<&mut crate::text_align::JustifyBudget>,
) -> Result<Pt, String> {
    if text.is_empty() {
        return Ok(Pt::ZERO);
    }

    let font_name = crate::style::resolve_font_family_key(&style.font_family, ctx.theme);
    let font = ctx.fonts.get_font(&font_name).ok_or_else(|| {
        format!(
            "Font '{}' not loaded (resolved from '{}')",
            font_name, style.font_family
        )
    })?;

    let mut run_glyphs = TextShaper::shape_text(text, font, style.font_size)?;
    crate::text_layout::apply_tracking(&mut run_glyphs, style.letter_spacing);
    let chars: Vec<char> = text.chars().collect();
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
        g.x_offset = g.x_offset + x_cursor + shift;
        // baseline_shift > 0 raises (y grows downward).
        g.y_offset = g.y_offset + y_cursor - style.baseline_shift;
        shift += extra;
        g.cluster = if from_source {
            char_origin + g.cluster
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

    Ok(run_advance)
}

fn arrange_leaf(node: &SemanticNode, position: Point, size: Size) -> GeometryNode {
    GeometryNode {
        id: node.id.clone(),
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        glyphs: vec![],
        text_runs: vec![],
        fill_rects: vec![],
        children: vec![],
    }
}

fn arrange_container(
    node: &SemanticNode,
    children: &[SemanticNode],
    position: Point,
    size: Size,
    ctx: &LayoutContext,
) -> Result<GeometryNode, String> {
    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let fixed = fixed_size_hint(&node.layout);
    let inner_pos = Point::new(position.x + padding.left, position.y + padding.top);
    let inner_size = Size::new(
        subtract_if_bounded(size.width, padding.horizontal()),
        subtract_if_bounded(size.height, padding.vertical()),
    );
    let inner_size_for_children = cap_inner_size_by_fixed(inner_size, padding, fixed);

    // Overlay container: children share the same origin and are stacked visually in source order
    // (first = back, last = front). Geometry order preserves this deterministically.
    if matches!(node.layout, Some(LayoutHint::Overlay { .. })) {
        let child_constraint = SizeConstraint::new(Size::ZERO, inner_size_for_children);
        let mut composed_children = Vec::with_capacity(children.len());
        for child in children {
            let child_size = measure_node(child, child_constraint, ctx)?;
            let child_geo = arrange_node(child, inner_pos, child_size, ctx)?;
            composed_children.push(child_geo);
        }

        return Ok(GeometryNode {
            id: node.id.clone(),
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
            glyphs: vec![],
            text_runs: vec![],
            fill_rects: vec![],
            children: composed_children,
        });
    }

    // Grid container
    if let Some(LayoutHint::Grid {
        columns,
        rows,
        gap,
        row_gap,
        column_gap,
        cell_align,
        ..
    }) = &node.layout
    {
        let (row_gap_pt, col_gap_pt) = crate::grid::grid_axis_gaps(*gap, *row_gap, *column_gap);
        let col_sizes = resolve_tracks(columns, col_gap_pt, inner_size_for_children.width)?;
        let row_sizes = resolve_tracks(rows, row_gap_pt, inner_size_for_children.height)?;

        let mut composed_children = Vec::new();
        for (idx, child) in children.iter().enumerate() {
            let c = idx % col_sizes.len();
            let r = idx / col_sizes.len();
            if r >= row_sizes.len() {
                break;
            }
            let cell_x = inner_pos.x + sum_prefix(&col_sizes, c, col_gap_pt);
            let cell_y = inner_pos.y + sum_prefix(&row_sizes, r, row_gap_pt);
            let cell_size = Size::new(col_sizes[c], row_sizes[r]);

            let child_constraint = SizeConstraint::new(Size::ZERO, cell_size);
            let measured_child = measure_node(child, child_constraint, ctx)?;

            let cell_align = cell_align.unwrap_or_default();
            let (dx, child_w) =
                align_offset_and_size(cell_align.x, cell_size.width, measured_child.width);
            let (dy, child_h) =
                align_offset_and_size(cell_align.y, cell_size.height, measured_child.height);

            let child_pos = Point::new(cell_x + dx, cell_y + dy);
            let child_size = Size::new(child_w, child_h);
            let child_geo = arrange_node(child, child_pos, child_size, ctx)?;
            composed_children.push(child_geo);
        }

        return Ok(GeometryNode {
            id: node.id.clone(),
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
            glyphs: vec![],
            text_runs: vec![],
            fill_rects: vec![],
            children: composed_children,
        });
    }

    // Columns (non-paginated path): pack children left-to-right into equal columns.
    if let Some(LayoutHint::Columns { count, gap }) = &node.layout {
        let gap_pt = Pt(*gap as i128);
        let tracks: Vec<k2f_core::GridTrack> = (0..*count as usize)
            .map(|_| k2f_core::GridTrack::Fr { fr: 1 })
            .collect();
        let widths = resolve_tracks(&tracks, gap_pt, inner_size_for_children.width)?;
        let n = widths.len().max(1);
        let col_h = inner_size_for_children.height;
        let mut composed_children = Vec::with_capacity(children.len());
        let mut col = 0usize;
        let mut y_in_col = Pt::ZERO;
        for child in children {
            let col_w = widths[col.min(n - 1)];
            let child_constraint = SizeConstraint::new(Size::ZERO, Size::new(col_w, Pt(i128::MAX)));
            let child_size = measure_node(child, child_constraint, ctx)?;
            if y_in_col + child_size.height > col_h && y_in_col != Pt::ZERO {
                col += 1;
                y_in_col = Pt::ZERO;
                if col >= n {
                    // Overflow past last column: still place in last column (deterministic clip risk).
                    col = n - 1;
                }
            }
            let mut x = inner_pos.x;
            for i in 0..col {
                x += widths[i] + gap_pt;
            }
            let child_pos = Point::new(x, inner_pos.y + y_in_col);
            let child_geo = arrange_node(child, child_pos, child_size, ctx)?;
            composed_children.push(child_geo);
            y_in_col += child_size.height;
        }
        return Ok(GeometryNode {
            id: node.id.clone(),
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
            glyphs: vec![],
            text_runs: vec![],
            fill_rects: vec![],
            children: composed_children,
        });
    }

    // Stack (default vertical)
    let (direction, gap, align_items, justify_content) = match &node.layout {
        Some(LayoutHint::Stack {
            direction,
            gap,
            align_items,
            justify_content,
            ..
        }) => (*direction, Pt(*gap as i128), *align_items, *justify_content),
        _ => (
            StackDirection::Vertical,
            Pt::ZERO,
            k2f_core::Align::default(),
            k2f_core::JustifyContent::default(),
        ),
    };

    let mut composed_children = Vec::new();

    match direction {
        StackDirection::Vertical => {
            // For vertical stacks: constrain width, leave height unconstrained per child.
            // If a fixed outer height is present, we still use it to bound children deterministically.
            let child_max_h = if fixed.height.is_some() {
                inner_size_for_children.height
            } else {
                Pt(i128::MAX)
            };
            let child_constraint = SizeConstraint::new(
                Size::ZERO,
                Size::new(inner_size_for_children.width, child_max_h),
            );

            // Measure pass to compute main-axis distribution.
            let mut measured_sizes: Vec<Size> = Vec::with_capacity(children.len());
            let mut used_main = Pt::ZERO;
            for (i, child) in children.iter().enumerate() {
                let s = measure_node(child, child_constraint, ctx)?;
                used_main += s.height;
                if gap != Pt::ZERO && i + 1 < children.len() {
                    used_main += gap;
                }
                measured_sizes.push(s);
            }

            let start_offset =
                justify_offset(justify_content, inner_size_for_children.height, used_main);
            let mut current_y = inner_pos.y + start_offset;

            for (i, child) in children.iter().enumerate() {
                let measured = measured_sizes[i];
                let self_align =
                    resolve_self_align(&child.role, child.variant.as_deref(), ctx.theme);
                let align_mode = self_align.unwrap_or(align_items);
                let (dx, child_w) = align_offset_and_size(
                    align_mode,
                    inner_size_for_children.width,
                    measured.width,
                );
                let child_pos = Point::new(inner_pos.x + dx, current_y);
                let child_size = Size::new(child_w, measured.height);
                let child_geo = arrange_node(child, child_pos, child_size, ctx)?;

                current_y += measured.height;
                if gap != Pt::ZERO && i + 1 < children.len() {
                    current_y += gap;
                }
                composed_children.push(child_geo);
            }
        }
        StackDirection::Horizontal => {
            // For horizontal stacks: constrain height, leave width unconstrained per child.
            // If a fixed outer width is present, we still use it to bound children deterministically.
            let child_max_w = if fixed.width.is_some() {
                inner_size_for_children.width
            } else {
                Pt(i128::MAX)
            };
            let child_constraint = SizeConstraint::new(
                Size::ZERO,
                Size::new(child_max_w, inner_size_for_children.height),
            );

            // Measure pass to compute main-axis distribution.
            let mut measured_sizes: Vec<Size> = Vec::with_capacity(children.len());
            let mut used_main = Pt::ZERO;
            for (i, child) in children.iter().enumerate() {
                let s = measure_node(child, child_constraint, ctx)?;
                used_main += s.width;
                if gap != Pt::ZERO && i + 1 < children.len() {
                    used_main += gap;
                }
                measured_sizes.push(s);
            }

            let start_offset =
                justify_offset(justify_content, inner_size_for_children.width, used_main);
            let mut current_x = inner_pos.x + start_offset;

            for (i, child) in children.iter().enumerate() {
                let measured = measured_sizes[i];
                let self_align =
                    resolve_self_align(&child.role, child.variant.as_deref(), ctx.theme);
                let align_mode = self_align.unwrap_or(align_items);
                let (dy, child_h) = align_offset_and_size(
                    align_mode,
                    inner_size_for_children.height,
                    measured.height,
                );
                let child_pos = Point::new(current_x, inner_pos.y + dy);
                let child_size = Size::new(measured.width, child_h);
                let child_geo = arrange_node(child, child_pos, child_size, ctx)?;

                current_x += measured.width;
                if gap != Pt::ZERO && i + 1 < children.len() {
                    current_x += gap;
                }
                composed_children.push(child_geo);
            }
        }
    }

    Ok(GeometryNode {
        id: node.id.clone(),
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        glyphs: vec![],
        text_runs: vec![],
        fill_rects: vec![],
        children: composed_children,
    })
}
