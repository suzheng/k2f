use super::super::node_builders::{
    container_node, default_page_config, image_node, text_node, text_node_role,
};
use k2f_core::{CanvasMode, Manifest, Pt, RunningBlockNode, RunningBlockPosition};

pub fn pagination_exact_fit_images() -> Manifest {
    let page = default_page_config();
    let content_h = page.height - page.margin[0] - page.margin[2];
    // Two images that exactly fill the content height (no gaps).
    let h1 = Pt(content_h.0 / 2);
    let h2 = Pt(content_h.0 - h1.0);

    Manifest {
        title: "Pagination: Exact Fit (Images)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: page,
        root: container_node(
            "root",
            "document",
            vec![
                image_node("img_1", Pt(400000), h1),
                image_node("img_2", Pt(400000), h2),
            ],
        ),
        running_blocks: vec![],
    }
}

pub fn pagination_just_over_images() -> Manifest {
    let page = default_page_config();
    let content_h = page.height - page.margin[0] - page.margin[2];
    // First fits, second pushes over by 1 unit, forcing a new page.
    let h1 = Pt(content_h.0 / 2);
    let h2 = Pt(content_h.0 - h1.0 + 1);

    Manifest {
        title: "Pagination: Just Over (Images)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: page,
        root: container_node(
            "root",
            "document",
            vec![
                image_node("img_1", Pt(400000), h1),
                image_node("img_2", Pt(400000), h2),
            ],
        ),
        running_blocks: vec![],
    }
}

pub fn pagination_oversize_first_item() -> Manifest {
    let page = default_page_config();
    let content_h = page.height - page.margin[0] - page.margin[2];
    // A single item larger than the content area; this is a deterministic overflow edge case.
    let oversized = Pt(content_h.0 + 200000);

    Manifest {
        title: "Pagination: Oversize First Item".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: page,
        root: container_node(
            "root",
            "document",
            vec![
                image_node("img_oversize", Pt(400000), oversized),
                text_node(
                    "after",
                    "After oversize item (should land on a later page deterministically).",
                ),
            ],
        ),
        running_blocks: vec![],
    }
}

pub fn pagination_large_page_count() -> Manifest {
    let page = default_page_config();
    let content_h = page.height - page.margin[0] - page.margin[2];
    // Create enough items to generate approximately 20-25 pages for stress testing pagination determinism.
    // Each image takes up roughly half the content height, so we get ~2 items per page.
    // Target: ~50 items = ~25 pages
    let item_height = Pt(content_h.0 / 2);
    let mut children = Vec::new();

    for i in 0..50 {
        children.push(image_node(&format!("img_{}", i), Pt(400000), item_height));
    }

    Manifest {
        title: "Pagination: Large Page Count Stress Test".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: page,
        root: container_node("root", "document", children),
        running_blocks: vec![],
    }
}

pub fn running_footer_page_numbers() -> Manifest {
    let mut manifest = pagination_just_over_images();
    manifest.title = "Running footer page numbers".to_string();
    manifest.running_blocks = vec![RunningBlockNode {
        position: RunningBlockPosition::Footer,
        node: text_node_role(
            "running.footer.page_numbers",
            "footer",
            "Page {{page_current}} of {{page_total}}",
        ),
    }];
    manifest
}
