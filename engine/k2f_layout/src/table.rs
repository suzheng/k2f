use crate::alignment::align_offset_and_size;
use crate::fixed_size::subtract_if_bounded;
use crate::grid::{grid_axis_gaps, resolve_tracks, sum_prefix, sum_with_gaps};
use crate::resolved_style::{align_for_child, padding_for_role_variant};
use crate::{arrange_node, measure_node, LayoutContext, Point, Size, SizeConstraint};
use k2f_core::{
    table_row_slots, Align, GeometryNode, NodeContent, Pt, SemanticNode, TableDataSource, TableSpec,
};

fn table_axis_gaps(spec: &TableSpec) -> (Pt, Pt) {
    grid_axis_gaps(spec.gap, spec.row_gap, spec.column_gap)
}

fn inline_rows(spec: &TableSpec) -> Result<&Vec<Vec<SemanticNode>>, String> {
    match &spec.data {
        TableDataSource::Inline { rows } => Ok(rows),
        TableDataSource::Asset { source } => Err(format!(
            "Table uses asset-backed data source '{source}'. Compile must expand assets before layout."
        )),
    }
}

fn spanned_cell_width(
    node_id: &str,
    col_sizes: &[Pt],
    start_col: usize,
    span: usize,
    col_gap: Pt,
) -> Result<Pt, String> {
    let end = start_col
        .checked_add(span)
        .ok_or_else(|| format!("Table '{node_id}' cell span overflow at column {start_col}"))?;
    if span == 0 || end > col_sizes.len() {
        return Err(format!(
            "Table '{}' cell at column {} span {} exceeds column count {}",
            node_id,
            start_col,
            span,
            col_sizes.len()
        ));
    }
    Ok(sum_with_gaps(&col_sizes[start_col..end], col_gap))
}

pub(crate) fn measure_row_height(
    node_id: &str,
    row: &[SemanticNode],
    col_sizes: &[Pt],
    col_gap: Pt,
    max_h: Pt,
    ctx: &LayoutContext,
) -> Result<Pt, String> {
    let mut row_h = Pt::ZERO;
    for (cell, slot) in row.iter().zip(table_row_slots(row)) {
        let width = spanned_cell_width(node_id, col_sizes, slot.start_col, slot.span, col_gap)?;
        let cell_constraint = SizeConstraint::new(Size::ZERO, Size::new(width, max_h));
        let cell_size = measure_node(cell, cell_constraint, ctx)?;
        if cell_size.height > row_h {
            row_h = cell_size.height;
        }
    }
    Ok(row_h)
}

fn measure_all_row_heights(
    node_id: &str,
    rows: &[Vec<SemanticNode>],
    col_sizes: &[Pt],
    col_gap: Pt,
    max_h: Pt,
    ctx: &LayoutContext,
) -> Result<Vec<Pt>, String> {
    let mut out: Vec<Pt> = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(measure_row_height(
            node_id, row, col_sizes, col_gap, max_h, ctx,
        )?);
    }
    Ok(out)
}

pub fn measure_table(
    node: &SemanticNode,
    spec: &TableSpec,
    constraint: SizeConstraint,
    ctx: &LayoutContext,
) -> Result<Size, String> {
    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let inner_max = Size::new(
        subtract_if_bounded(constraint.max.width, padding.horizontal()),
        subtract_if_bounded(constraint.max.height, padding.vertical()),
    );

    let (row_gap_pt, col_gap_pt) = table_axis_gaps(spec);
    let col_sizes = resolve_tracks(&spec.column_widths, col_gap_pt, inner_max.width)?;

    let rows = inline_rows(spec)?;
    let row_heights = measure_all_row_heights(
        &node.id,
        rows,
        &col_sizes,
        col_gap_pt,
        inner_max.height,
        ctx,
    )?;

    let inner_w = sum_with_gaps(&col_sizes, col_gap_pt);
    let inner_h = sum_with_gaps(&row_heights, row_gap_pt);
    let measured = Size::new(inner_w + padding.horizontal(), inner_h + padding.vertical());
    Ok(constraint.constrain(measured))
}

pub fn arrange_table(
    node: &SemanticNode,
    spec: &TableSpec,
    position: Point,
    size: Size,
    ctx: &LayoutContext,
) -> Result<GeometryNode, String> {
    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let inner_size = Size::new(
        subtract_if_bounded(size.width, padding.horizontal()),
        subtract_if_bounded(size.height, padding.vertical()),
    );

    let (_, col_gap_pt) = table_axis_gaps(spec);
    let col_sizes = resolve_tracks(&spec.column_widths, col_gap_pt, inner_size.width)?;
    let rows = inline_rows(spec)?;

    // Re-measure row heights deterministically using the final negotiated inner size.
    let row_heights = measure_all_row_heights(
        &node.id,
        rows,
        &col_sizes,
        col_gap_pt,
        inner_size.height,
        ctx,
    )?;
    let row_indices: Vec<usize> = (0..rows.len()).collect();
    arrange_table_fragment(node, spec, position, size, ctx, &row_indices, &row_heights)
}

pub(crate) fn arrange_table_fragment(
    node: &SemanticNode,
    spec: &TableSpec,
    position: Point,
    size: Size,
    ctx: &LayoutContext,
    row_indices: &[usize],
    row_heights: &[Pt],
) -> Result<GeometryNode, String> {
    if row_indices.len() != row_heights.len() {
        return Err(format!(
            "Table '{}' fragment row_indices len {} does not match row_heights len {}",
            node.id,
            row_indices.len(),
            row_heights.len()
        ));
    }

    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let inner_pos = Point::new(position.x + padding.left, position.y + padding.top);
    let inner_size = Size::new(
        subtract_if_bounded(size.width, padding.horizontal()),
        subtract_if_bounded(size.height, padding.vertical()),
    );

    let (row_gap_pt, col_gap_pt) = table_axis_gaps(spec);
    let col_sizes = resolve_tracks(&spec.column_widths, col_gap_pt, inner_size.width)?;
    let rows = inline_rows(spec)?;

    let mut composed_children: Vec<GeometryNode> = Vec::new();
    for (frag_r, &src_r) in row_indices.iter().enumerate() {
        let row = rows.get(src_r).ok_or_else(|| {
            format!(
                "Table '{}' fragment references out-of-range row {}",
                node.id, src_r
            )
        })?;
        let cell_y = inner_pos.y + sum_prefix(row_heights, frag_r, row_gap_pt);
        for (cell, slot) in row.iter().zip(table_row_slots(row)) {
            let cell_x = inner_pos.x + sum_prefix(&col_sizes, slot.start_col, col_gap_pt);
            let cell_w =
                spanned_cell_width(&node.id, &col_sizes, slot.start_col, slot.span, col_gap_pt)?;
            let cell_size = Size::new(cell_w, row_heights[frag_r]);
            let cell_constraint = SizeConstraint::new(Size::ZERO, cell_size);
            let measured = measure_node(cell, cell_constraint, ctx)?;
            let (dx, child_w) = if matches!(cell.content, NodeContent::FormField(_)) {
                align_offset_and_size(
                    align_for_child(cell, Align::Stretch, ctx.theme),
                    cell_size.width,
                    measured.width,
                )
            } else {
                (Pt::ZERO, cell_size.width)
            };
            let align_y = align_for_child(cell, Align::Stretch, ctx.theme);
            let (dy, child_h) = align_offset_and_size(align_y, cell_size.height, measured.height);
            let child_geo = arrange_node(
                cell,
                Point::new(cell_x + dx, cell_y + dy),
                Size::new(child_w, child_h),
                ctx,
            )?;
            composed_children.push(child_geo);
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
