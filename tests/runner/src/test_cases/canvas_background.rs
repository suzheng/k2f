use k2f_core::{CanvasMode, Manifest};

use super::super::node_builders::{
    container_node_with_variant, default_page_config, text_node_role,
};

pub fn canvas_background_gradient() -> Manifest {
    Manifest {
        title: "Canvas Background Gradient".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        // The root selects a role+variant whose box decoration background references
        // a named theme surface primitive (global background).
        root: container_node_with_variant(
            "root",
            "document",
            Some("notes"),
            vec![text_node_role(
                "body_1",
                "body",
                "This page should paint a global background before content.",
            )],
        ),
        running_blocks: vec![],
    }
}
