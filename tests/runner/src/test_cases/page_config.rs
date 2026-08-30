use super::super::node_builders::{container_node, image_node, text_node};
use k2f_core::{CanvasMode, Manifest, PageConfig, Pt};

pub fn page_zero_margins() -> Manifest {
    Manifest {
        title: "PageConfig: Zero Margins".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: PageConfig {
            width: Pt(595000),
            height: Pt(842000),
            margin: [Pt::ZERO; 4],
        },
        root: container_node(
            "root",
            "document",
            vec![
                text_node("t1", "No margins: top-left origin should be (0,0)."),
                image_node("img", Pt(200000), Pt(120000)),
                text_node("t2", "Second block after image."),
            ],
        ),
        running_blocks: vec![],
    }
}

pub fn page_zero_content_width() -> Manifest {
    // left + right == width => content width == 0
    Manifest {
        title: "PageConfig: Zero Content Width".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: PageConfig {
            width: Pt(200000),
            height: Pt(300000),
            margin: [Pt(20000), Pt(100000), Pt(20000), Pt(100000)],
        },
        // Use images only to avoid undefined text-wrap behavior at width=0.
        root: container_node(
            "root",
            "document",
            vec![
                image_node("img1", Pt(50000), Pt(40000)),
                image_node("img2", Pt(60000), Pt(50000)),
            ],
        ),
        running_blocks: vec![],
    }
}

pub fn page_near_zero_margins() -> Manifest {
    // Margins set to 1 unit (minimum non-zero) to test edge case behavior
    Manifest {
        title: "PageConfig: Near-Zero Margins (1 unit)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: PageConfig {
            width: Pt(595000),
            height: Pt(842000),
            margin: [Pt(1), Pt(1), Pt(1), Pt(1)],
        },
        root: container_node(
            "root",
            "document",
            vec![
                text_node(
                    "t1",
                    "Minimal margins (1 unit each): content area should be nearly full page.",
                ),
                image_node("img", Pt(200000), Pt(120000)),
                text_node("t2", "Second block after image."),
            ],
        ),
        running_blocks: vec![],
    }
}

pub fn page_near_zero_content_width() -> Manifest {
    // Content width == 1 unit (left + right == width - 1)
    Manifest {
        title: "PageConfig: Near-Zero Content Width (1 unit)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: PageConfig {
            width: Pt(200001),
            height: Pt(300000),
            margin: [Pt(20000), Pt(100000), Pt(20000), Pt(100000)],
        },
        // Use images only to avoid text-wrap issues at very narrow width
        root: container_node(
            "root",
            "document",
            vec![
                image_node("img1", Pt(50000), Pt(40000)),
                image_node("img2", Pt(60000), Pt(50000)),
            ],
        ),
        running_blocks: vec![],
    }
}

pub fn page_near_zero_content_height() -> Manifest {
    // Content height == 1 unit (top + bottom == height - 1)
    Manifest {
        title: "PageConfig: Near-Zero Content Height (1 unit)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: PageConfig {
            width: Pt(595000),
            height: Pt(40001),
            margin: [Pt(20000), Pt(72000), Pt(20000), Pt(72000)],
        },
        root: container_node(
            "root",
            "document",
            vec![
                text_node(
                    "t1",
                    "Minimal content height: items should paginate immediately.",
                ),
                image_node("img1", Pt(200000), Pt(50000)),
                text_node("t2", "This should appear on a later page."),
            ],
        ),
        running_blocks: vec![],
    }
}
