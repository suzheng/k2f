use k2f_core::{CanvasMode, Manifest, Pt};

use super::super::node_builders::{
    container_node, container_node_with_variant, default_page_config, image_node,
    overlay_container, text_node_role,
};

pub fn overlay_glass_over_image() -> Manifest {
    let mut img = image_node("bg_image", Pt(420_000), Pt(220_000));
    img.role = "body".to_string();

    let glass_card = container_node_with_variant(
        "glass_card",
        "card",
        Some("glass"),
        vec![text_node_role(
            "glass_card_text",
            "body",
            "Glass card should paint on top of the image with stable ordering.",
        )],
    );

    let scene = overlay_container("scene", "document", vec![img, glass_card]);

    Manifest {
        title: "Overlay: Glass Card Over Image".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![scene]),
        running_blocks: vec![],
    }
}
