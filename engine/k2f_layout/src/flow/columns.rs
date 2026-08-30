use crate::grid::resolve_tracks;
use crate::resolved_style::{padding_for_role_variant, EdgeInsets};
use crate::{LayoutContext, Paginator, Pt};
use k2f_core::{Align, CanvasMode, GridTrack, LayoutHint, NodeContent, SemanticNode};

use super::place::{paginate_flow_items, place_item, region_break};

enum Segment<'a> {
    Flow(&'a [SemanticNode]),
    Span(&'a SemanticNode),
}

/// Place a `LayoutHint::Columns` container into the paginator.
pub(crate) fn place_columns(
    node: &SemanticNode,
    paginator: &mut Paginator,
    content_width: Pt,
    align_items: Align,
    ctx: &LayoutContext,
) -> Result<(), String> {
    let (count, gap_i64) = match &node.layout {
        Some(LayoutHint::Columns { count, gap }) => (*count as usize, *gap),
        _ => {
            return Err(format!(
                "place_columns called without columns layout on '{}'",
                node.id
            ))
        }
    };
    if !(2..=4).contains(&count) {
        return Err(format!(
            "columns count must be 2..=4 on '{}' (got {count})",
            node.id
        ));
    }
    let gap_pt = Pt(gap_i64 as i128);
    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;

    let NodeContent::Container { children } = &node.content else {
        return Err(format!(
            "columns layout requires container content on '{}'",
            node.id
        ));
    };

    let tracks: Vec<GridTrack> = (0..count).map(|_| GridTrack::Fr { fr: 1 }).collect();
    let widths = resolve_tracks(&tracks, gap_pt, content_width)?;
    let measure_w = widths.iter().copied().min().unwrap_or(content_width);

    // Padding/chrome is not fragmented: the whole columns block must fit one page.
    if padding != EdgeInsets::ZERO {
        return place_columns_atomic(
            node,
            children,
            paginator,
            &widths,
            gap_pt,
            measure_w,
            padding,
            align_items,
            ctx,
        );
    }

    for seg in split_segments(children) {
        match seg {
            Segment::Flow(items) => {
                if items.is_empty() {
                    continue;
                }
                place_flow_segment(
                    items,
                    paginator,
                    &widths,
                    gap_pt,
                    measure_w,
                    align_items,
                    ctx,
                )?;
            }
            Segment::Span(item) => {
                if paginator.in_column_band() {
                    paginator.exit_column_band();
                }
                place_item(
                    item,
                    paginator,
                    content_width,
                    align_items,
                    CanvasMode::Paged,
                    ctx,
                )?;
            }
        }
    }

    if paginator.in_column_band() {
        paginator.exit_column_band();
    }
    Ok(())
}

fn place_columns_atomic(
    node: &SemanticNode,
    children: &[SemanticNode],
    paginator: &mut Paginator,
    widths: &[Pt],
    col_gap: Pt,
    measure_w: Pt,
    padding: EdgeInsets,
    align_items: Align,
    ctx: &LayoutContext,
) -> Result<(), String> {
    let n_cols = widths.len() as i128;
    let total_h = measure_flow_height(children, measure_w, ctx)?;
    let packed_inner = ceil_div_pt(total_h, n_cols);
    let packed = packed_inner + padding.vertical();
    if packed > paginator.remaining_height() && !paginator.at_content_top() {
        region_break(paginator)?;
    }
    if packed > paginator.page_content_height() {
        return Err(format!(
            "UNSPLITTABLE_OVERFLOW: padded columns '{}' height {} exceeds page content height {}",
            node.id,
            packed.0,
            paginator.page_content_height().0
        ));
    }
    let top = paginator.current_y;
    let inner_top = top + padding.top;
    let inner_bottom = inner_top + packed_inner;
    paginator.enter_column_band(
        paginator.content_left(),
        widths.to_vec(),
        col_gap,
        inner_top,
        inner_bottom,
    )?;
    paginate_flow_items(
        children,
        paginator,
        measure_w,
        Pt::ZERO,
        align_items,
        CanvasMode::Paged,
        ctx,
    )?;
    if paginator.in_column_band() {
        paginator.exit_column_band();
    }
    paginator.current_y = top + packed;
    Ok(())
}

fn split_segments(children: &[SemanticNode]) -> Vec<Segment<'_>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < children.len() {
        if children[i].column_span.is_all() {
            out.push(Segment::Span(&children[i]));
            i += 1;
            continue;
        }
        let start = i;
        while i < children.len() && !children[i].column_span.is_all() {
            i += 1;
        }
        out.push(Segment::Flow(&children[start..i]));
    }
    out
}

fn place_flow_segment(
    items: &[SemanticNode],
    paginator: &mut Paginator,
    widths: &[Pt],
    col_gap: Pt,
    measure_w: Pt,
    align_items: Align,
    ctx: &LayoutContext,
) -> Result<(), String> {
    if paginator.in_column_band() {
        paginator.exit_column_band();
    }

    let n_cols = widths.len() as i128;
    let total_h = measure_flow_height(items, measure_w, ctx)?;
    let min_block = max_min_start_height(items, measure_w, ctx)?;

    let mut remaining_page = paginator.remaining_page_height();
    if min_block > remaining_page && !paginator.at_content_top() {
        region_break(paginator)?;
        remaining_page = paginator.remaining_page_height();
    }

    let mut band_h = ceil_div_pt(total_h, n_cols);
    if band_h < min_block {
        band_h = min_block;
    }
    // Line-granularity: ceil(H/N) can be shorter than N whole lines require.
    // Bump so each column can hold ceil(line_slots / N) minimum blocks.
    if min_block.0 > 0 {
        let slots = (total_h.0 + min_block.0 - 1) / min_block.0;
        let per_col = (slots + n_cols - 1) / n_cols;
        let needed = Pt(per_col * min_block.0);
        if needed > band_h {
            band_h = needed;
        }
    }
    if band_h > remaining_page {
        band_h = remaining_page;
    }
    if band_h.0 <= 0 {
        region_break(paginator)?;
        remaining_page = paginator.remaining_page_height();
        band_h = ceil_div_pt(total_h, n_cols)
            .max(min_block)
            .min(remaining_page);
    }

    let band_top = paginator.current_y;
    let band_bottom = band_top + band_h;
    paginator.enter_column_band(
        paginator.content_left(),
        widths.to_vec(),
        col_gap,
        band_top,
        band_bottom,
    )?;

    paginate_flow_items(
        items,
        paginator,
        measure_w,
        Pt::ZERO,
        align_items,
        CanvasMode::Paged,
        ctx,
    )?;

    if paginator.in_column_band() {
        paginator.exit_column_band();
    }
    Ok(())
}

fn measure_flow_height(
    items: &[SemanticNode],
    width: Pt,
    ctx: &LayoutContext,
) -> Result<Pt, String> {
    let mut h = Pt::ZERO;
    for item in items {
        h += super::text_frag::measure_full_height(item, width, ctx)?;
    }
    Ok(h)
}

fn max_min_start_height(
    items: &[SemanticNode],
    width: Pt,
    ctx: &LayoutContext,
) -> Result<Pt, String> {
    let mut m = Pt::ZERO;
    for item in items {
        let h = super::place::min_start_height_for(item, width, ctx)?;
        if h > m {
            m = h;
        }
    }
    Ok(m)
}

fn ceil_div_pt(h: Pt, n: i128) -> Pt {
    if n <= 0 {
        return h;
    }
    if h.0 <= 0 {
        return Pt::ZERO;
    }
    Pt((h.0 + n - 1) / n)
}
