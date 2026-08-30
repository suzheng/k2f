use k2f_core::{CanvasMode, Manifest, StackDirection};

use super::super::node_builders::{
    container_node, container_node_with_variant, default_page_config, stack_container,
    text_node_role,
};

pub fn role_variant_primitives() -> Manifest {
    let card = container_node_with_variant(
        "card_1",
        "card",
        Some("glass"),
        vec![text_node_role(
            "card_text",
            "body",
            "Card with role-driven variants and named primitives.",
        )],
    );

    Manifest {
        title: "Role + Variant + Primitives".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![card]),
        running_blocks: vec![],
    }
}

pub fn card_variants() -> Manifest {
    let card_default = container_node(
        "card_default",
        "card",
        vec![text_node_role(
            "card_default_text",
            "body",
            "Default card (base role decoration).",
        )],
    );
    let card_warning = container_node_with_variant(
        "card_warning",
        "card",
        Some("warning"),
        vec![text_node_role(
            "card_warning_text",
            "body",
            "Warning card variant.",
        )],
    );
    let card_glass = container_node_with_variant(
        "card_glass",
        "card",
        Some("glass"),
        vec![text_node_role(
            "card_glass_text",
            "body",
            "Glass card variant (backdrop blur + border).",
        )],
    );

    let stack = stack_container(
        "cards",
        "document",
        StackDirection::Vertical,
        18_000, // 18pt gap
        vec![card_default, card_warning, card_glass],
    );

    Manifest {
        title: "Card Variants".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![stack]),
        running_blocks: vec![],
    }
}

pub fn elevation_test() -> Manifest {
    let card_low = container_node(
        "elevation_low",
        "card",
        vec![text_node_role(
            "elevation_low_text",
            "body",
            "Low elevation shadow.",
        )],
    );
    let card_high = container_node_with_variant(
        "elevation_high",
        "card",
        Some("elevation_high"),
        vec![text_node_role(
            "elevation_high_text",
            "body",
            "High elevation shadow.",
        )],
    );

    let stack = stack_container(
        "elevations",
        "document",
        StackDirection::Vertical,
        18_000,
        vec![card_low, card_high],
    );

    Manifest {
        title: "Elevation Shadows".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![stack]),
        running_blocks: vec![],
    }
}
