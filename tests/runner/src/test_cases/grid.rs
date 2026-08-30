use super::super::node_builders::{
    container_node, default_page_config, grid_container, image_node, text_node,
};
use k2f_core::{CanvasMode, GridTrack, Manifest};

pub fn grid_track_matrix_basic() -> Manifest {
    let grid = grid_container(
        "grid",
        "box",
        vec![
            GridTrack::Pt { pt: 140000 },
            GridTrack::Fr { fr: 1 },
            GridTrack::Fr { fr: 2 },
        ],
        vec![GridTrack::Pt { pt: 80000 }, GridTrack::Pt { pt: 90000 }],
        8000,
        vec![
            text_node("c1", "Pt col"),
            text_node("c2", "Fr=1 col"),
            text_node("c3", "Fr=2 col"),
            image_node("c4_img", k2f_core::Pt(60000), k2f_core::Pt(60000)),
            text_node("c5", "Row2-Col2"),
            text_node("c6", "Row2-Col3"),
        ],
    );

    Manifest {
        title: "Grid: Basic Track Matrix (Pt + Fr Columns, Pt Rows)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![grid]),
        running_blocks: vec![],
    }
}

pub fn grid_fixed_overflow() -> Manifest {
    // Fixed columns that exceed available width. This asserts deterministic behavior when
    // fixed tracks don't fit (engine currently does not shrink fixed tracks).
    let grid = grid_container(
        "grid",
        "box",
        vec![GridTrack::Pt { pt: 400000 }, GridTrack::Pt { pt: 250000 }],
        vec![GridTrack::Pt { pt: 120000 }],
        12000,
        vec![
            text_node("left", "Fixed 400pt-ish column"),
            text_node(
                "right",
                "Fixed 250pt-ish column (may overflow content width)",
            ),
        ],
    );

    Manifest {
        title: "Grid: Fixed Track Overflow".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![grid]),
        running_blocks: vec![],
    }
}

pub fn grid_nested_fr_rows() -> Manifest {
    // Nested grid uses fr rows, but only inside a finite-height parent cell.
    let nested = grid_container(
        "nested_grid",
        "box",
        vec![GridTrack::Fr { fr: 1 }, GridTrack::Fr { fr: 1 }],
        vec![GridTrack::Fr { fr: 1 }, GridTrack::Fr { fr: 2 }],
        6000,
        vec![
            text_node("n11", "n(1,1)"),
            text_node("n12", "n(1,2)"),
            text_node("n21", "n(2,1)"),
            text_node("n22", "n(2,2)"),
        ],
    );

    let parent = grid_container(
        "parent_grid",
        "box",
        vec![GridTrack::Pt { pt: 220000 }, GridTrack::Pt { pt: 220000 }],
        vec![GridTrack::Pt { pt: 180000 }, GridTrack::Pt { pt: 140000 }],
        12000,
        vec![
            nested,
            text_node("p12", "Sibling cell"),
            image_node("p21_img", k2f_core::Pt(100000), k2f_core::Pt(90000)),
            text_node("p22", "Bottom-right"),
        ],
    );

    Manifest {
        title: "Grid: Nested Fr Rows (Finite Cell)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![parent]),
        running_blocks: vec![],
    }
}

pub fn grid_fr_remainder_distribution() -> Manifest {
    // 3 equal fr tracks in a finite width. When the remaining width isn't divisible by 3,
    // the grid resolver deterministically assigns leftover units to earlier fr tracks.
    let grid = grid_container(
        "grid",
        "box",
        vec![
            GridTrack::Fr { fr: 1 },
            GridTrack::Fr { fr: 1 },
            GridTrack::Fr { fr: 1 },
        ],
        vec![GridTrack::Pt { pt: 90000 }],
        0,
        vec![
            text_node("c1", "FR1"),
            text_node("c2", "FR2"),
            text_node("c3", "FR3"),
        ],
    );

    Manifest {
        title: "Grid: Fr Remainder Distribution".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![grid]),
        running_blocks: vec![],
    }
}

pub fn grid_partially_filled() -> Manifest {
    // Grid with 3 columns and 3 rows (9 cells total), but only 5 children.
    // Tests that empty cells are handled correctly - grid maintains full size but
    // only filled cells contain child geometry nodes.
    let grid = grid_container(
        "grid",
        "box",
        vec![
            GridTrack::Pt { pt: 120000 },
            GridTrack::Pt { pt: 120000 },
            GridTrack::Pt { pt: 120000 },
        ],
        vec![
            GridTrack::Pt { pt: 80000 },
            GridTrack::Pt { pt: 80000 },
            GridTrack::Pt { pt: 80000 },
        ],
        10000,
        vec![
            text_node("c1", "Cell (1,1)"),
            text_node("c2", "Cell (1,2)"),
            text_node("c3", "Cell (1,3)"),
            text_node("c4", "Cell (2,1)"),
            text_node("c5", "Cell (2,2)"),
            // Cells (2,3), (3,1), (3,2), (3,3) are empty
        ],
    );

    Manifest {
        title: "Grid: Partially Filled (Fewer Children Than Cells)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![grid]),
        running_blocks: vec![],
    }
}
