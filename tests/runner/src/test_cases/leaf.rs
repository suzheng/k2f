use super::super::node_builders::{
    container_node, default_page_config, grid_container, image_node, stack_container,
    table_ref_node, text_node,
};
use k2f_core::{CanvasMode, GridTrack, Manifest, Pt, StackDirection};

pub fn leaf_image_table_reference() -> Manifest {
    Manifest {
        title: "Leaf: Image + TableReference".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node(
            "root",
            "document",
            vec![
                text_node("label_img", "Image (fits width)"),
                image_node("img_fit", Pt(200000), Pt(80000)),
                text_node("label_tbl", "TableReference (explicit size)"),
                table_ref_node("tbl_ref", Pt(300000), Pt(90000)),
                text_node(
                    "label_scale",
                    "Image (too wide -> deterministic scale-down)",
                ),
                image_node("img_too_wide", Pt(700000), Pt(140000)),
            ],
        ),
        running_blocks: vec![],
    }
}

pub fn grid_cell_constraints_scale_image() -> Manifest {
    // We can't constrain leaf nodes in both axes directly (grid children fill the cell),
    // so we do height-constrained scaling via a horizontal stack inside a finite-height grid row.
    let inner = stack_container(
        "inner",
        "box",
        StackDirection::Horizontal,
        12000,
        vec![
            // Too tall for the row => scales down based on max height.
            image_node("img_tall", Pt(200000), Pt(200000)),
            image_node("img_ok", Pt(60000), Pt(60000)),
        ],
    );

    let grid = grid_container(
        "grid",
        "box",
        vec![GridTrack::Pt { pt: 420000 }],
        vec![GridTrack::Pt { pt: 80000 }],
        0,
        vec![inner],
    );

    Manifest {
        title: "Leaf: Image Scale-Down by Height (Bounded Row)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![grid]),
        running_blocks: vec![],
    }
}

pub fn table_reference_constrained_in_cell() -> Manifest {
    // TableReference uses deterministic clamping to the constraint (no proportional scaling).
    // We exercise height clamping by placing it in a horizontal stack inside a bounded-height row.
    let inner = stack_container(
        "inner",
        "box",
        StackDirection::Horizontal,
        12000,
        vec![
            table_ref_node("tbl_big", Pt(280000), Pt(200000)),
            table_ref_node("tbl_ok", Pt(80000), Pt(40000)),
        ],
    );

    let grid = grid_container(
        "grid",
        "box",
        vec![GridTrack::Pt { pt: 420000 }],
        vec![GridTrack::Pt { pt: 70000 }],
        0,
        vec![inner],
    );

    Manifest {
        title: "Leaf: TableReference Constrained by Height (Bounded Row)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![grid]),
        running_blocks: vec![],
    }
}

pub fn image_scaling_by_width_paged() -> Manifest {
    // Test image scaling by width constraint in paged mode.
    // Default page config: width=595pt, margins=72pt each side -> content_width=451pt
    // We create an image wider than 451pt to test width-constrained scaling.
    // Image: 600pt wide x 200pt tall (aspect ratio 3:1)
    // Expected: Image should scale down to fit 451pt width, maintaining aspect ratio.
    // Scaled size: width=451pt, height=150.33pt (approximately)
    Manifest {
        title: "Leaf: Image Scaling by Width (Paged Content Width)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node(
            "root",
            "document",
            vec![
                text_node("label1", "Image wider than page content width (600pt x 200pt natural size)."),
                text_node("label2", "Expected: Scales down to fit 451pt content width, maintaining 3:1 aspect ratio."),
                image_node("img_wide", Pt(600000), Pt(200000)),
                text_node("label3", "Image that fits within content width (300pt x 100pt)."),
                image_node("img_fits", Pt(300000), Pt(100000)),
            ],
        ),
        running_blocks: vec![],
    }
}

pub fn image_scaling_by_height_bounded_container() -> Manifest {
    // Test image scaling by height constraint in a bounded vertical container.
    // Create a vertical stack with fixed height that constrains child images.
    // The container height is 150pt, and we place an image that's 200pt tall.
    // Expected: Image scales down to fit 150pt height, maintaining aspect ratio.
    let bounded_container = stack_container(
        "bounded",
        "box",
        StackDirection::Vertical,
        10000,
        vec![
            text_node(
                "label1",
                "Bounded container (150pt height) with tall image (200pt x 300pt).",
            ),
            text_node(
                "label2",
                "Expected: Image scales down to fit container height, maintaining aspect ratio.",
            ),
            image_node("img_tall", Pt(300000), Pt(200000)),
        ],
    );

    let grid = grid_container(
        "grid",
        "box",
        vec![GridTrack::Pt { pt: 400000 }],
        vec![GridTrack::Pt { pt: 150000 }],
        0,
        vec![bounded_container],
    );

    Manifest {
        title: "Leaf: Image Scaling by Height (Bounded Container)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![grid]),
        running_blocks: vec![],
    }
}

pub fn table_reference_clamping_bounded_height() -> Manifest {
    // Test table reference clamping under bounded height constraint.
    // TableReference uses clamping (not scaling), so it should be clipped to the constraint.
    // Create a bounded-height row (80pt) and place a table reference that's 120pt tall.
    // Expected: Table reference height is clamped to 80pt, width remains unchanged.
    let inner = stack_container(
        "inner",
        "box",
        StackDirection::Horizontal,
        12000,
        vec![
            text_node(
                "label1",
                "Bounded row (80pt height) with tall table reference (120pt x 200pt).",
            ),
            text_node(
                "label2",
                "Expected: Table reference height clamped to 80pt (no scaling, just clamping).",
            ),
            table_ref_node("tbl_tall", Pt(200000), Pt(120000)),
            table_ref_node("tbl_fits", Pt(100000), Pt(50000)),
        ],
    );

    let grid = grid_container(
        "grid",
        "box",
        vec![GridTrack::Pt { pt: 450000 }],
        vec![GridTrack::Pt { pt: 80000 }],
        0,
        vec![inner],
    );

    Manifest {
        title: "Leaf: TableReference Clamping Under Bounded Height".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![grid]),
        running_blocks: vec![],
    }
}
