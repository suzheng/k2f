mod alignment;
pub mod arrange;
mod compile;
mod fixed_size;
mod flow;
mod fonts;
pub mod grid;
pub mod layout_context;
mod leaf;
mod list_item_measure;
mod list_markers;
pub mod list_style;
mod math;
pub mod measure;
pub mod pagination;
mod render_plan;
pub mod resolved_style;
mod running_blocks;
pub mod style;
mod table;
mod table_pagination;
mod text_align;
pub mod text_layout;
pub mod theme;
pub mod visual_primitives;

#[cfg(test)]
mod alignment_tests;

pub use arrange::arrange_node;
pub use compile::{compile_chunk_with_fonts, compile_manifest};
pub use k2f_core::AssetsMap;
pub use layout_context::{LayoutContext, Point, Size, SizeConstraint};
pub use list_style::*;
pub use measure::measure_node;
pub use pagination::Paginator;
pub use resolved_style::*;
pub use style::*;
pub use text_layout::{layout_text, TextLayout};
pub use theme::*;
pub use visual_primitives::*;

use k2f_core::{CanvasMode, LayoutResult, Manifest, Pt};

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn compile_chunk(
        content_json: &str,
        theme_json: &str,
        font_data: &[u8],
    ) -> Result<String, String> {
        crate::compile::compile_chunk(content_json, theme_json, font_data, None)
    }

    pub fn compile_chunk_with_assets(
        content_json: &str,
        theme_json: &str,
        font_data: &[u8],
        assets: &k2f_core::AssetsMap,
    ) -> Result<String, String> {
        crate::compile::compile_chunk(content_json, theme_json, font_data, Some(assets))
    }

    pub fn layout(manifest: &Manifest, ctx: &LayoutContext) -> Result<LayoutResult, String> {
        // Derive list marker labels once up front so pagination and arrangement can access them
        // without mutating the semantic tree.
        let mut derived_ctx = LayoutContext::new(ctx.fonts, ctx.theme);
        derived_ctx.list_markers =
            crate::list_markers::derive_list_marker_map(&manifest.root, ctx.theme)?;
        let ctx = &derived_ctx;

        // PageConfig invariants are required for correct margin/pagination behavior.
        crate::pagination::validate_page_config(&manifest.page_config)?;

        // 2. Measure the root node (Pass 1)
        // We measure the logical root.
        // In this architecture, the Manifest has a single root.
        // If it's a Container, we care about its children for pagination.
        // If it's a leaf, it just goes on the first page.

        // Vertical stacks paginate at child and line boundaries. Nested padded boxes
        // stay atomic (no chrome fragmentation). `break_inside: avoid` never splits.

        use k2f_core::NodeContent;
        let root_padding = padding_for_role_variant(
            &manifest.root.role,
            manifest.root.variant.as_deref(),
            ctx.theme,
        )?;

        // Root padding should constrain top-level pagination/layout the same way a "document"
        // container would in a non-paginated tree. We model that by inflating the page margins.
        let mut page_config = manifest.page_config.clone();
        page_config.margin[0] = page_config.margin[0] + root_padding.top;
        page_config.margin[1] = page_config.margin[1] + root_padding.right;
        page_config.margin[2] = page_config.margin[2] + root_padding.bottom;
        page_config.margin[3] = page_config.margin[3] + root_padding.left;

        // If we adjusted margins, ensure we don't violate invariants.
        crate::pagination::validate_page_config(&page_config)?;

        // Replace paginator with padded margins (keeps pagination logic centralized).
        let mut paginator = Paginator::new(page_config.clone(), manifest.canvas_mode);
        let content_width = crate::pagination::content_width(&page_config);

        if let NodeContent::Container { children } = &manifest.root.content {
            // Respect the root container's stack gap + alignment when laying out top-level items.
            //
            // Current pagination model paginates root's immediate children; we still do that,
            // but we apply the same stack semantics (gap + cross-axis alignment) that
            // `arrange_container` would apply if the root itself were arranged.
            let (direction, gap_i64, align_items) = match &manifest.root.layout {
                Some(k2f_core::LayoutHint::Stack {
                    direction,
                    gap,
                    align_items,
                    ..
                }) => (*direction, *gap, *align_items),
                _ => (
                    k2f_core::StackDirection::Vertical,
                    0_i64,
                    k2f_core::Align::Stretch,
                ),
            };

            // Only vertical stack behaves like document flow today.
            // If the root declares horizontal, we fall back to the previous behavior.
            if direction == k2f_core::StackDirection::Horizontal {
                for child in children {
                    let constraint =
                        SizeConstraint::new(Size::ZERO, Size::new(content_width, Pt(i128::MAX)));
                    let size = measure_node(child, constraint, ctx)?;
                    let pos = paginator.allocate_space(size.height);
                    let geo = arrange_node(child, pos, size, ctx)?;
                    paginator.add_item(geo);
                }
            } else {
                crate::flow::paginate_flow_items(
                    children,
                    &mut paginator,
                    content_width,
                    Pt(gap_i64 as i128),
                    align_items,
                    manifest.canvas_mode,
                    ctx,
                )?;
            }
        } else {
            crate::flow::paginate_flow_items(
                std::slice::from_ref(&manifest.root),
                &mut paginator,
                content_width,
                Pt::ZERO,
                k2f_core::Align::Start,
                manifest.canvas_mode,
                ctx,
            )?;
        }

        let mut pages = paginator.pages;
        if manifest.canvas_mode == CanvasMode::Paged && !manifest.running_blocks.is_empty() {
            crate::running_blocks::inject_running_blocks(manifest, &mut pages, ctx)?;
        }

        Ok(LayoutResult { pages })
    }
}

#[cfg(test)]
mod arrange_tests;
#[cfg(test)]
mod code_role_tests;
#[cfg(test)]
mod columns_tests;
#[cfg(test)]
mod grid_tests;
#[cfg(test)]
mod math_layout_tests;
#[cfg(test)]
mod measure_tests;
#[cfg(test)]
mod pagination_tests;
#[cfg(test)]
mod table_pagination_tests;
#[cfg(test)]
mod table_tests;
#[cfg(test)]
pub mod test_utils;
