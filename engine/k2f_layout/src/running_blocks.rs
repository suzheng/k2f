use crate::alignment::align_offset_and_size;
use crate::resolved_style::resolve_self_align;
use crate::{arrange_node, measure_node, LayoutContext, Point, Size, SizeConstraint};
use k2f_core::{
    Align, Manifest, NodeContent, Pt, RunningBlockPosition, SemanticNode, TableDataSource,
};

pub fn inject_running_blocks(
    manifest: &Manifest,
    pages: &mut [k2f_core::Page],
    ctx: &LayoutContext,
) -> Result<(), String> {
    if manifest.running_blocks.is_empty() {
        return Ok(());
    }
    if manifest.canvas_mode != k2f_core::CanvasMode::Paged {
        // Validation should have rejected this already, but keep layout robust.
        return Err("running blocks are only supported when canvas_mode is 'paged'".to_string());
    }

    let page_total = pages.len();
    if page_total == 0 {
        return Ok(());
    }

    let base_page_config = &manifest.page_config;
    let content_w = crate::pagination::content_width(base_page_config);
    let x_base = base_page_config.margin[3];

    for page in pages.iter_mut() {
        let page_current = page.index + 1;

        // Running blocks in header/footer should stack independently, but should be processed
        // in deterministic manifest order for paint ordering.
        let mut header_y = Pt::ZERO;
        let mut footer_y = base_page_config.height - base_page_config.margin[2];

        for rb in &manifest.running_blocks {
            let substituted = substitute_placeholders_in_node(&rb.node, page_current, page_total);

            let (cursor, band_end) = match rb.position {
                RunningBlockPosition::Header => (&mut header_y, base_page_config.margin[0]),
                RunningBlockPosition::Footer => (&mut footer_y, base_page_config.height),
            };

            if *cursor > band_end {
                return Err(format!(
                    "running block overflow: position {:?} has no remaining band height on page {}",
                    rb.position, page_current
                ));
            }

            let remaining_h = band_end - *cursor;
            let constraint = SizeConstraint::new(Size::ZERO, Size::new(content_w, remaining_h));
            let measured = measure_node(&substituted, constraint, ctx)?;

            let self_align =
                resolve_self_align(&substituted.role, substituted.variant.as_deref(), ctx.theme);
            let align_mode = self_align.unwrap_or(Align::Start);
            let (dx, child_w) = align_offset_and_size(align_mode, content_w, measured.width);

            // Place within the header/footer band; geometry is appended after main content so it paints as overlay.
            let pos = Point::new(x_base + dx, *cursor);
            let arranged_size = Size::new(child_w, measured.height);
            let geo = arrange_node(&substituted, pos, arranged_size, ctx)?;

            *cursor = *cursor + measured.height;
            if *cursor > band_end {
                return Err(format!(
                    "running block overflow: position {:?} exceeds band height on page {}",
                    rb.position, page_current
                ));
            }

            page.root.children.push(geo);
        }
    }

    Ok(())
}

fn substitute_placeholders_in_node(
    node: &SemanticNode,
    page_current: usize,
    page_total: usize,
) -> SemanticNode {
    let replace_text = |s: &str| {
        s.replace("{{page_current}}", &page_current.to_string())
            .replace("{{page_total}}", &page_total.to_string())
    };

    let content = match &node.content {
        NodeContent::Text(text) => NodeContent::Text(replace_text(text)),
        NodeContent::Container { children } => NodeContent::Container {
            children: children
                .iter()
                .map(|c| substitute_placeholders_in_node(c, page_current, page_total))
                .collect(),
        },
        NodeContent::Table(spec) => {
            let mut spec2 = spec.clone();
            if let TableDataSource::Inline { rows } = &spec.data {
                let mut new_rows: Vec<Vec<SemanticNode>> = Vec::with_capacity(rows.len());
                for row in rows {
                    let mut new_row: Vec<SemanticNode> = Vec::with_capacity(row.len());
                    for cell in row {
                        new_row.push(substitute_placeholders_in_node(
                            cell,
                            page_current,
                            page_total,
                        ));
                    }
                    new_rows.push(new_row);
                }
                spec2.data = TableDataSource::Inline { rows: new_rows };
            }
            NodeContent::Table(spec2)
        }
        other => other.clone(),
    };

    let mut out = node.clone();
    out.content = content;
    out
}
