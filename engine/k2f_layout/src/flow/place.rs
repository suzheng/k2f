use crate::alignment::align_offset_and_size;
use crate::resolved_style::resolve_self_align;
use crate::{LayoutContext, Paginator, Point, Size, SizeConstraint};
use k2f_core::{
    Align, BreakInside, CanvasMode, LayoutHint, NodeContent, Pt, SemanticNode, StackDirection,
};

use super::columns;
use super::text_frag;

pub(crate) fn paginate_flow_items(
    items: &[SemanticNode],
    paginator: &mut Paginator,
    content_width: Pt,
    gap: Pt,
    align_items: Align,
    canvas_mode: CanvasMode,
    ctx: &LayoutContext,
) -> Result<(), String> {
    let n = items.len();
    let mut i = 0;
    while i < n {
        let group_end = keep_group_end(items, i);
        if canvas_mode == CanvasMode::Paged {
            ensure_group_starts_together(items, i, group_end, paginator, content_width, gap, ctx)?;
        }
        while i <= group_end {
            place_item(
                &items[i],
                paginator,
                content_width,
                align_items,
                canvas_mode,
                ctx,
            )?;
            if gap != Pt::ZERO && i + 1 < n {
                apply_gap(paginator, gap, canvas_mode);
            }
            i += 1;
        }
    }
    Ok(())
}

/// Break to the next packing region: next column when in a band, else next page.
pub(crate) fn region_break(paginator: &mut Paginator) -> Result<(), String> {
    if paginator.in_column_band() {
        paginator.column_break()?;
    } else {
        paginator.page_break();
    }
    Ok(())
}

fn keep_group_end(items: &[SemanticNode], start: usize) -> usize {
    let mut end = start;
    while end + 1 < items.len() && items[end].keep_with_next {
        end += 1;
    }
    end
}

fn ensure_group_starts_together(
    items: &[SemanticNode],
    start: usize,
    end: usize,
    paginator: &mut Paginator,
    content_width: Pt,
    gap: Pt,
    ctx: &LayoutContext,
) -> Result<(), String> {
    if paginator.at_content_top() {
        return Ok(());
    }
    let mut needed = Pt::ZERO;
    for (k, item) in items[start..=end].iter().enumerate() {
        if k > 0 {
            needed += gap;
        }
        needed += min_start_height_for(item, content_width, ctx)?;
    }
    if needed > paginator.remaining_height() {
        region_break(paginator)?;
    }
    Ok(())
}

pub(crate) fn min_start_height_for(
    node: &SemanticNode,
    content_width: Pt,
    ctx: &LayoutContext,
) -> Result<Pt, String> {
    if can_split_text(node) {
        let prep = text_frag::prepare_text(node, content_width, ctx)?;
        return Ok(text_frag::first_line_height(&prep));
    }
    if can_split_stack(node, ctx)? {
        if let NodeContent::Container { children } = &node.content {
            if let Some(first) = children.first() {
                return min_start_height_for(first, content_width, ctx);
            }
        }
    }
    text_frag::measure_full_height(node, content_width, ctx)
}

fn can_split_text(node: &SemanticNode) -> bool {
    node.break_inside != BreakInside::Avoid
        && matches!(node.content, NodeContent::Text(_))
        && node.role != "list_item"
}

fn can_split_stack(node: &SemanticNode, ctx: &LayoutContext) -> Result<bool, String> {
    if node.break_inside == BreakInside::Avoid {
        return Ok(false);
    }
    if !matches!(node.content, NodeContent::Container { .. }) {
        return Ok(false);
    }
    match &node.layout {
        Some(LayoutHint::Overlay { .. })
        | Some(LayoutHint::Grid { .. })
        | Some(LayoutHint::Columns { .. }) => return Ok(false),
        Some(LayoutHint::Stack {
            direction: StackDirection::Horizontal,
            ..
        }) => return Ok(false),
        _ => {}
    }
    // Padding/chrome is not fragmented. A padded card taller than a page must `avoid` or fail.
    let padding = crate::resolved_style::padding_for_role_variant(
        &node.role,
        node.variant.as_deref(),
        ctx.theme,
    )?;
    Ok(padding == crate::resolved_style::EdgeInsets::ZERO)
}

fn apply_gap(paginator: &mut Paginator, gap: Pt, canvas_mode: CanvasMode) {
    if canvas_mode == CanvasMode::Infinite {
        paginator.current_y += gap;
        return;
    }
    let limit = match &paginator.column_band {
        Some(b) => b.band_bottom,
        None => paginator.page_config.height - paginator.page_config.margin[2],
    };
    if paginator.current_y + gap <= limit {
        paginator.current_y += gap;
        if let Some(b) = paginator.column_band.as_mut() {
            if paginator.current_y > b.used_max_y {
                b.used_max_y = paginator.current_y;
            }
        }
    }
}

pub(crate) fn place_item(
    node: &SemanticNode,
    paginator: &mut Paginator,
    content_width: Pt,
    align_items: Align,
    canvas_mode: CanvasMode,
    ctx: &LayoutContext,
) -> Result<(), String> {
    let self_align = resolve_self_align(&node.role, node.variant.as_deref(), ctx.theme);
    let align_mode = self_align.unwrap_or(align_items);

    if canvas_mode == CanvasMode::Paged {
        if matches!(node.layout, Some(LayoutHint::Columns { .. })) {
            return columns::place_columns(node, paginator, content_width, align_mode, ctx);
        }
        if let NodeContent::Table(spec) = &node.content {
            crate::table_pagination::paginate_table_root_flow_paged(
                node,
                spec,
                paginator,
                content_width,
                align_mode,
                ctx,
            )?;
            return Ok(());
        }
    }

    if canvas_mode == CanvasMode::Paged && can_split_text(node) {
        return place_splittable_text(node, paginator, content_width, align_mode, ctx);
    }

    if canvas_mode == CanvasMode::Paged && can_split_stack(node, ctx)? {
        return place_splittable_stack(node, paginator, content_width, align_mode, ctx);
    }

    place_unsplittable(node, paginator, content_width, align_mode, canvas_mode, ctx)
}

fn place_unsplittable(
    node: &SemanticNode,
    paginator: &mut Paginator,
    content_width: Pt,
    align_mode: Align,
    canvas_mode: CanvasMode,
    ctx: &LayoutContext,
) -> Result<(), String> {
    let constraint = SizeConstraint::new(Size::ZERO, Size::new(content_width, Pt(i128::MAX)));
    let measured = crate::measure_node(node, constraint, ctx)?;
    let (dx, child_w) = align_offset_and_size(align_mode, content_width, measured.width);

    if canvas_mode == CanvasMode::Paged {
        if measured.height > paginator.remaining_height() && !paginator.at_content_top() {
            region_break(paginator)?;
        }
        if measured.height > paginator.page_content_height() {
            return Err(format!(
                "UNSPLITTABLE_OVERFLOW: node '{}' height {} exceeds page content height {}",
                node.id,
                measured.height.0,
                paginator.page_content_height().0
            ));
        }
    }

    let pos = paginator.allocate_space(measured.height);
    let child_pos = Point::new(pos.x + dx, pos.y);
    let geo = text_frag::arrange_whole(node, child_pos, child_w, measured.height, ctx)?;
    paginator.add_item(geo);
    Ok(())
}

fn place_splittable_text(
    node: &SemanticNode,
    paginator: &mut Paginator,
    content_width: Pt,
    align_mode: Align,
    ctx: &LayoutContext,
) -> Result<(), String> {
    let prep = text_frag::prepare_text(node, content_width, ctx)?;
    let full_h = text_frag::fragment_height(&prep, 0, prep.layout.lines.len());
    let (dx, child_w) =
        align_offset_and_size(align_mode, content_width, prep.outer_w.min(content_width));
    let child_w = if child_w.0 == 0 {
        content_width
    } else {
        child_w
    };

    // In a column band, never take the "move whole block" shortcuts — always pack by
    // line fragments so a paragraph can split across columns without duplicating glyphs.
    let in_band = paginator.in_column_band();

    if full_h <= paginator.remaining_height() {
        let pos = paginator.allocate_space(full_h);
        let geo = text_frag::arrange_fragment(
            node,
            &prep,
            0,
            prep.layout.lines.len(),
            Point::new(pos.x + dx, pos.y),
            child_w,
            ctx,
        )?;
        paginator.add_item(geo);
        return Ok(());
    }

    if !in_band && !paginator.at_content_top() && full_h <= paginator.page_content_height() {
        region_break(paginator)?;
        let pos = paginator.allocate_space(full_h);
        let geo = text_frag::arrange_fragment(
            node,
            &prep,
            0,
            prep.layout.lines.len(),
            Point::new(pos.x + dx, pos.y),
            child_w,
            ctx,
        )?;
        paginator.add_item(geo);
        return Ok(());
    }

    let n = prep.layout.lines.len();
    if n == 0 {
        return place_unsplittable(
            node,
            paginator,
            content_width,
            align_mode,
            CanvasMode::Paged,
            ctx,
        );
    }

    let mut start = 0;
    while start < n {
        if text_frag::fragment_height(&prep, start, start + 1) > paginator.remaining_height()
            && !paginator.at_content_top()
        {
            region_break(paginator)?;
        }
        if text_frag::fragment_height(&prep, start, start + 1) > paginator.page_content_height() {
            return Err(format!(
                "UNSPLITTABLE_OVERFLOW: text '{}' line {} taller than a page",
                node.id, start
            ));
        }
        let mut end = start + 1;
        while end < n
            && text_frag::fragment_height(&prep, start, end + 1) <= paginator.remaining_height()
        {
            end += 1;
        }
        let h = text_frag::fragment_height(&prep, start, end);
        let pos = paginator.allocate_space(h);
        let geo = text_frag::arrange_fragment(
            node,
            &prep,
            start,
            end,
            Point::new(pos.x + dx, pos.y),
            child_w,
            ctx,
        )?;
        paginator.add_item(geo);
        start = end;
    }
    Ok(())
}

fn place_splittable_stack(
    node: &SemanticNode,
    paginator: &mut Paginator,
    content_width: Pt,
    align_mode: Align,
    ctx: &LayoutContext,
) -> Result<(), String> {
    let full_h = text_frag::measure_full_height(node, content_width, ctx)?;
    if full_h <= paginator.page_content_height() {
        return place_unsplittable(
            node,
            paginator,
            content_width,
            align_mode,
            CanvasMode::Paged,
            ctx,
        );
    }

    let NodeContent::Container { children } = &node.content else {
        return place_unsplittable(
            node,
            paginator,
            content_width,
            align_mode,
            CanvasMode::Paged,
            ctx,
        );
    };
    let (gap, child_align) = match &node.layout {
        Some(LayoutHint::Stack {
            gap, align_items, ..
        }) => (Pt(*gap as i128), *align_items),
        _ => (Pt::ZERO, Align::Stretch),
    };

    paginate_flow_items(
        children,
        paginator,
        content_width,
        gap,
        child_align,
        CanvasMode::Paged,
        ctx,
    )
}
