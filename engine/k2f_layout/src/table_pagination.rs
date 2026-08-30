use crate::alignment::align_offset_and_size;
use crate::fixed_size::subtract_if_bounded;
use crate::grid::{resolve_tracks, sum_with_gaps};
use crate::resolved_style::padding_for_role_variant;
use crate::{LayoutContext, Paginator, Point, Size};
use k2f_core::{Align, NodeContent, Pt, SemanticNode, TableDataSource, TableSpec};

/// Root-flow pagination for strict tables in paged mode:
/// split on row boundaries and optionally repeat header rows at page breaks.
pub(crate) fn paginate_table_root_flow_paged(
    node: &SemanticNode,
    spec: &TableSpec,
    paginator: &mut Paginator,
    content_width: Pt,
    align_mode: Align,
    ctx: &LayoutContext,
) -> Result<(), String> {
    if paginator.mode != k2f_core::CanvasMode::Paged {
        return Err("paginate_table_root_flow_paged called in non-paged mode".to_string());
    }
    if !matches!(node.content, NodeContent::Table(_)) {
        return Err("paginate_table_root_flow_paged called for non-table node".to_string());
    }

    let padding = padding_for_role_variant(&node.role, node.variant.as_deref(), ctx.theme)?;
    let gap_pt = Pt(spec.gap as i128);

    let rows = match &spec.data {
        TableDataSource::Inline { rows } => rows,
        TableDataSource::Asset { source } => {
            return Err(format!(
                "Table '{}' uses asset-backed data source '{source}'. Compile must expand assets before pagination/layout.",
                node.id
            ));
        }
    };

    // Determine the table's natural width (without measuring all rows) and apply the root-flow
    // alignment/stretch rules to produce a stable fragment width.
    let inner_max_w = subtract_if_bounded(content_width, padding.horizontal());
    let col_sizes_for_width = resolve_tracks(&spec.column_widths, gap_pt, inner_max_w)?;
    let natural_inner_w = sum_with_gaps(&col_sizes_for_width, gap_pt);
    let mut natural_outer_w = natural_inner_w + padding.horizontal();
    if natural_outer_w > content_width {
        natural_outer_w = content_width;
    }

    let (dx, frag_w) = align_offset_and_size(align_mode, content_width, natural_outer_w);
    let frag_inner_w = subtract_if_bounded(frag_w, padding.horizontal());
    let col_sizes = resolve_tracks(&spec.column_widths, gap_pt, frag_inner_w)?;

    // Empty tables still paint deterministically as an empty (padded) box.
    if rows.is_empty() {
        let h = padding.vertical();
        let pos = paginator.allocate_space(h);
        let frag_pos = Point::new(pos.x + dx, pos.y);
        let geo = crate::table::arrange_table_fragment(
            node,
            spec,
            frag_pos,
            Size::new(frag_w, h),
            ctx,
            &[],
            &[],
        )?;
        paginator.add_item(geo);
        return Ok(());
    }

    let header_rows = spec.header_rows.min(rows.len());
    let mut row_height_cache: Vec<Option<Pt>> = vec![None; rows.len()];
    let mut row_height_for = |idx: usize| -> Result<Pt, String> {
        if let Some(h) = row_height_cache[idx] {
            return Ok(h);
        }
        let max_h = Pt(i128::MAX);
        let h = crate::table::measure_row_height(&node.id, &rows[idx], &col_sizes, max_h, ctx)?;
        row_height_cache[idx] = Some(h);
        Ok(h)
    };

    let mut first_fragment = true;
    let mut next_row: usize = 0;

    while next_row < rows.len() {
        let bottom_limit = paginator.page_config.height - paginator.page_config.margin[2];
        let remaining = bottom_limit - paginator.current_y;
        let at_top = paginator.current_y == paginator.page_config.margin[0];

        let repeat_headers = !first_fragment && header_rows > 0 && next_row >= header_rows;

        let mut frag_row_indices: Vec<usize> = Vec::new();
        let mut frag_row_heights: Vec<Pt> = Vec::new();
        let mut inner_used = Pt::ZERO;

        let mut try_add_row = |row_idx: usize,
                               frag_row_indices: &mut Vec<usize>,
                               frag_row_heights: &mut Vec<Pt>,
                               inner_used: &mut Pt|
         -> Result<bool, String> {
            let h = row_height_for(row_idx)?;
            let add = if frag_row_heights.is_empty() {
                h
            } else {
                gap_pt + h
            };
            let needed = padding.vertical() + (*inner_used + add);
            if needed <= remaining {
                frag_row_indices.push(row_idx);
                frag_row_heights.push(h);
                *inner_used += add;
                Ok(true)
            } else {
                Ok(false)
            }
        };

        // If we need repeated headers, they must fit before any body row is considered.
        if repeat_headers {
            let mut header_fit = true;
            for hidx in 0..header_rows {
                if !try_add_row(
                    hidx,
                    &mut frag_row_indices,
                    &mut frag_row_heights,
                    &mut inner_used,
                )? {
                    header_fit = false;
                    break;
                }
            }

            if !header_fit {
                if !at_top {
                    paginator.page_break();
                    continue;
                }
                return Err(format!(
                    "Table '{}' header rows do not fit on an empty page (cannot paginate)",
                    node.id
                ));
            }
        }

        // Add as many body rows as fit.
        let mut body_cursor = next_row;
        while body_cursor < rows.len() {
            if try_add_row(
                body_cursor,
                &mut frag_row_indices,
                &mut frag_row_heights,
                &mut inner_used,
            )? {
                body_cursor += 1;
            } else {
                break;
            }
        }

        // If we made no forward progress on body rows, we need a page break or an error.
        if body_cursor == next_row {
            if !at_top {
                paginator.page_break();
                continue;
            }

            // On an empty page we still can't fit the next row (or the next row + repeated headers).
            return Err(format!(
                "Table '{}' row {} does not fit on a page (cannot paginate on row boundaries)",
                node.id, next_row
            ));
        }

        let fragment_h = padding.vertical() + inner_used;
        let pos = paginator.allocate_space(fragment_h);
        let frag_pos = Point::new(pos.x + dx, pos.y);
        let geo = crate::table::arrange_table_fragment(
            node,
            spec,
            frag_pos,
            Size::new(frag_w, fragment_h),
            ctx,
            &frag_row_indices,
            &frag_row_heights,
        )?;
        paginator.add_item(geo);

        first_fragment = false;
        next_row = body_cursor;
    }

    Ok(())
}
