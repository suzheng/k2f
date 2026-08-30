use crate::fixed_size::subtract_if_bounded;
use crate::fixed_size::{apply_fixed_min_outer, fixed_size_hint, inner_max_for_children};
use crate::grid::{resolve_tracks, sum_with_gaps};
use crate::list_item_measure::list_item_measure_spec;
use crate::resolved_style::padding_for_role_variant;
use crate::text_layout::layout_code_block;
use crate::text_layout::layout_text;
use crate::{LayoutContext, Size, SizeConstraint};
use k2f_core::{LayoutHint, NodeContent, Pt, SemanticNode, StackDirection};

pub fn measure_node(
    node: &SemanticNode,
    constraint: SizeConstraint,
    ctx: &LayoutContext,
) -> Result<Size, String> {
    match &node.content {
        NodeContent::Text(text) => measure_text(text, node, constraint, ctx),
        NodeContent::CodeBlock(code) => measure_code_block(code, node, constraint, ctx),
        NodeContent::Math(tex) => crate::math::measure_math(tex, node, ctx),
        NodeContent::Image { width, height, .. } => {
            measure_image(*width, *height, node, constraint, ctx)
        }
        NodeContent::Container { children } => measure_container(node, children, constraint, ctx),
        NodeContent::Table(spec) => crate::table::measure_table(node, spec, constraint, ctx),
        NodeContent::TableReference { width, height, .. } => {
            crate::leaf::measure_table_reference(*width, *height, constraint)
        }
    }
}

fn measure_image(
    width: Pt,
    height: Pt,
    node: &SemanticNode,
    constraint: SizeConstraint,
    ctx: &LayoutContext,
) -> Result<Size, String> {
    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let inner_max = Size::new(
        subtract_if_bounded(constraint.max.width, padding.horizontal()),
        subtract_if_bounded(constraint.max.height, padding.vertical()),
    );
    let inner_constraint = SizeConstraint::new(Size::ZERO, inner_max);
    let inner = crate::leaf::measure_image(width, height, inner_constraint)?;
    Ok(constraint.constrain(Size::new(
        inner.width + padding.horizontal(),
        inner.height + padding.vertical(),
    )))
}

fn measure_code_block(
    code: &k2f_core::CodeBlockValue,
    node: &SemanticNode,
    constraint: SizeConstraint,
    ctx: &LayoutContext,
) -> Result<Size, String> {
    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let inner_max = Size::new(
        subtract_if_bounded(constraint.max.width, padding.horizontal()),
        subtract_if_bounded(constraint.max.height, padding.vertical()),
    );
    let inner_constraint = SizeConstraint::new(Size::ZERO, inner_max);

    let canonical = code.to_canonical_text();
    let layout = layout_code_block(
        canonical.as_ref(),
        &node.role,
        node.variant.as_deref(),
        &node.modifiers,
        inner_constraint,
        ctx,
    )?;

    Ok(constraint.constrain(Size::new(
        layout.width + padding.horizontal(),
        layout.height + padding.vertical(),
    )))
}

fn measure_text(
    text: &str,
    node: &SemanticNode,
    constraint: SizeConstraint,
    ctx: &LayoutContext,
) -> Result<Size, String> {
    if node.role == "list_item" {
        return measure_list_item_text(text, node, constraint, ctx);
    }

    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let inner_max = Size::new(
        subtract_if_bounded(constraint.max.width, padding.horizontal()),
        subtract_if_bounded(constraint.max.height, padding.vertical()),
    );
    let inner_constraint = SizeConstraint::new(Size::ZERO, inner_max);
    let layout = if node.role == "code" {
        layout_code_block(
            text,
            &node.role,
            node.variant.as_deref(),
            &node.modifiers,
            inner_constraint,
            ctx,
        )?
    } else {
        layout_text(
            text,
            &node.role,
            node.variant.as_deref(),
            &node.modifiers,
            inner_constraint,
            ctx,
        )?
    };
    Ok(constraint.constrain(Size::new(
        layout.width + padding.horizontal(),
        layout.height + padding.vertical(),
    )))
}

fn measure_list_item_text(
    text: &str,
    node: &SemanticNode,
    constraint: SizeConstraint,
    ctx: &LayoutContext,
) -> Result<Size, String> {
    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let inner_max = Size::new(
        subtract_if_bounded(constraint.max.width, padding.horizontal()),
        subtract_if_bounded(constraint.max.height, padding.vertical()),
    );

    let spec = list_item_measure_spec(node, ctx)?;
    let leading = spec.leading_width();

    let wrap_w = subtract_if_bounded(inner_max.width, leading);
    let inner_constraint = SizeConstraint::new(Size::ZERO, Size::new(wrap_w, inner_max.height));

    let layout = layout_text(
        text,
        &node.role,
        node.variant.as_deref(),
        &node.modifiers,
        inner_constraint,
        ctx,
    )?;

    Ok(constraint.constrain(Size::new(
        layout.width + leading + padding.horizontal(),
        layout.height + padding.vertical(),
    )))
}

fn measure_container(
    node: &SemanticNode,
    children: &[SemanticNode],
    constraint: SizeConstraint,
    ctx: &LayoutContext,
) -> Result<Size, String> {
    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let fixed = fixed_size_hint(&node.layout);
    let inner_max = inner_max_for_children(constraint, padding, fixed);
    let inner_max_w = inner_max.width;
    let inner_max_h = inner_max.height;

    // Default strategy: Vertical Stack
    // Width = Max Child Width
    // Height = Sum Child Heights

    // Overlay container: children are measured independently and placed at the same origin.
    // Container size is the max child width/height (plus padding).
    if matches!(node.layout, Some(LayoutHint::Overlay { .. })) {
        let mut width = Pt::ZERO;
        let mut height = Pt::ZERO;

        let child_constraint = SizeConstraint::new(Size::ZERO, Size::new(inner_max_w, inner_max_h));

        for child in children {
            let child_size = measure_node(child, child_constraint, ctx)?;
            if child_size.width > width {
                width = child_size.width;
            }
            if child_size.height > height {
                height = child_size.height;
            }
        }

        let measured = Size::new(width + padding.horizontal(), height + padding.vertical());
        return Ok(apply_fixed_min_outer(measured, fixed, constraint));
    }

    // Grid container (explicit tracks only)
    if let Some(LayoutHint::Grid {
        columns,
        rows,
        gap,
        row_gap,
        column_gap,
        ..
    }) = &node.layout
    {
        let available_w = inner_max_w;
        let available_h = inner_max_h;
        let (row_gap_pt, col_gap_pt) = crate::grid::grid_axis_gaps(*gap, *row_gap, *column_gap);
        let col_sizes = resolve_tracks(columns, col_gap_pt, available_w)?;
        let row_sizes = resolve_tracks(rows, row_gap_pt, available_h)?;

        let total_w = sum_with_gaps(&col_sizes, col_gap_pt);
        let total_h = sum_with_gaps(&row_sizes, row_gap_pt);

        // Measure children inside their cells (for determinism and future overflow checks)
        for (idx, child) in children.iter().enumerate() {
            let c = idx % col_sizes.len();
            let r = idx / col_sizes.len();
            if r >= row_sizes.len() {
                break;
            }
            let cell = Size::new(col_sizes[c], row_sizes[r]);
            let child_constraint = SizeConstraint::new(Size::ZERO, cell);
            let _ = measure_node(child, child_constraint, ctx)?;
        }

        let measured = Size::new(
            total_w + padding.horizontal(),
            total_h + padding.vertical(),
        );
        return Ok(apply_fixed_min_outer(measured, fixed, constraint));
    }

    // Continuous multi-column flow: full width, height ≈ ceil(content_h / N).
    if let Some(LayoutHint::Columns { count, gap }) = &node.layout {
        let n = (*count).max(1) as i128;
        let gap_pt = Pt(*gap as i128);
        let tracks: Vec<k2f_core::GridTrack> = (0..*count as usize)
            .map(|_| k2f_core::GridTrack::Fr { fr: 1 })
            .collect();
        let widths = resolve_tracks(&tracks, gap_pt, inner_max_w)?;
        let measure_w = widths.iter().copied().min().unwrap_or(inner_max_w);
        let child_constraint = SizeConstraint::new(Size::ZERO, Size::new(measure_w, Pt(i128::MAX)));
        let mut content_h = Pt::ZERO;
        for child in children {
            content_h += measure_node(child, child_constraint, ctx)?.height;
        }
        let packed_h = if content_h.0 <= 0 {
            Pt::ZERO
        } else {
            Pt((content_h.0 + n - 1) / n)
        };
        let measured = Size::new(
            inner_max_w + padding.horizontal(),
            packed_h + padding.vertical(),
        );
        return Ok(apply_fixed_min_outer(measured, fixed, constraint));
    }

    // Stack container (vertical or horizontal)
    let (direction, gap) = match &node.layout {
        Some(LayoutHint::Stack { direction, gap, .. }) => (*direction, Pt(*gap as i128)),
        _ => (StackDirection::Vertical, Pt::ZERO),
    };

    match direction {
        StackDirection::Vertical => {
            let mut width = Pt::ZERO;
            let mut height = Pt::ZERO;

            // For vertical stacks: constrain width, leave height unconstrained per child.
            // If a fixed outer height is present, we still use it to bound children deterministically.
            let child_max_h = if fixed.height.is_some() {
                inner_max_h
            } else {
                Pt(i128::MAX)
            };
            let child_constraint =
                SizeConstraint::new(Size::ZERO, Size::new(inner_max_w, child_max_h));

            for (i, child) in children.iter().enumerate() {
                let child_size = measure_node(child, child_constraint, ctx)?;
                if child_size.width > width {
                    width = child_size.width;
                }
                height += child_size.height;
                if gap != Pt::ZERO && i + 1 < children.len() {
                    height += gap;
                }
            }

            let measured = Size::new(width + padding.horizontal(), height + padding.vertical());
            Ok(apply_fixed_min_outer(measured, fixed, constraint))
        }
        StackDirection::Horizontal => {
            let mut width = Pt::ZERO;
            let mut height = Pt::ZERO;

            // For horizontal stacks: constrain height, leave width unconstrained per child.
            // If a fixed outer width is present, we still use it to bound children deterministically.
            let child_max_w = if fixed.width.is_some() {
                inner_max_w
            } else {
                Pt(i128::MAX)
            };
            let child_constraint =
                SizeConstraint::new(Size::ZERO, Size::new(child_max_w, inner_max_h));

            for (i, child) in children.iter().enumerate() {
                let child_size = measure_node(child, child_constraint, ctx)?;
                width += child_size.width;
                if child_size.height > height {
                    height = child_size.height;
                }
                if gap != Pt::ZERO && i + 1 < children.len() {
                    width += gap;
                }
            }

            let measured = Size::new(width + padding.horizontal(), height + padding.vertical());
            Ok(apply_fixed_min_outer(measured, fixed, constraint))
        }
    }
}
