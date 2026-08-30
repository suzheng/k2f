use k2f_core::{CanvasMode, ListMarkerType, Manifest, SemanticNode};

use super::super::node_builders::{container_node, default_page_config};

fn list_item_node(
    id: &str,
    list_id: &str,
    depth: u32,
    marker_type: Option<ListMarkerType>,
    text: &str,
    variant: Option<&str>,
) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "list_item".to_string(),
        variant: variant.map(|v| v.to_string()),
        preserve_whitespace: None,
        list_id: Some(list_id.to_string()),
        depth: Some(depth),
        marker_type,
        content: k2f_core::NodeContent::Text(text.to_string()),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    }
}

pub fn semantic_lists_bullets_basic() -> Manifest {
    let children = vec![
        list_item_node(
            "li.1",
            "list_a",
            0,
            Some(ListMarkerType::Bullet),
            "First item",
            Some("compact"),
        ),
        list_item_node(
            "li.2",
            "list_a",
            0,
            Some(ListMarkerType::Bullet),
            "Second item with a longer sentence that should wrap onto multiple lines to exercise the fixed marker box + hanging indent behavior.",
            Some("compact"),
        ),
        list_item_node(
            "li.3",
            "list_a",
            0,
            Some(ListMarkerType::Bullet),
            "Third item",
            Some("compact"),
        ),
    ];

    Manifest {
        title: "Semantic Lists: Bullets Basic".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", children),
        running_blocks: vec![],
    }
}

pub fn semantic_lists_numbered_nested() -> Manifest {
    let children = vec![
        list_item_node(
            "steps.1",
            "steps",
            0,
            Some(ListMarkerType::Number),
            "Install Rust",
            None,
        ),
        list_item_node(
            "steps.2",
            "steps",
            1,
            Some(ListMarkerType::Number),
            "Install wasm-pack",
            None,
        ),
        list_item_node(
            "steps.3",
            "steps",
            1,
            Some(ListMarkerType::Number),
            "Build the project",
            None,
        ),
        list_item_node(
            "steps.4",
            "steps",
            0,
            Some(ListMarkerType::Number),
            "Run tests",
            None,
        ),
    ];

    Manifest {
        title: "Semantic Lists: Numbered + Nested".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", children),
        running_blocks: vec![],
    }
}

pub fn semantic_lists_pagination_continues_numbers() -> Manifest {
    // Many items in a numbered list to ensure pagination occurs and numbering is derived
    // from the semantic stream (not page fragments).
    let mut children: Vec<SemanticNode> = Vec::new();
    for i in 1..=40 {
        children.push(list_item_node(
            &format!("p.li.{i}"),
            "paged_numbers",
            0,
            Some(ListMarkerType::Number),
            "This is a long list item designed to consume vertical space and force page breaks. The numbering should remain consistent across pages.",
            None,
        ));
    }

    Manifest {
        title: "Semantic Lists: Pagination Continues Numbers".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", children),
        running_blocks: vec![],
    }
}

pub fn semantic_lists_digit_boundary_no_reflow() -> Manifest {
    // Items around the 9 -> 10 boundary. The marker label width changes in characters,
    // but wrap width must remain stable due to fixed marker box sizing.
    let mut children: Vec<SemanticNode> = Vec::new();
    for i in 1..=12 {
        children.push(list_item_node(
            &format!("b.li.{i}"),
            "boundary",
            0,
            Some(ListMarkerType::Number),
            "A moderately long sentence that should wrap the same way for items 9 and 10, regardless of the marker string width.",
            None,
        ));
    }

    Manifest {
        title: "Semantic Lists: Digit Boundary No Reflow".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", children),
        running_blocks: vec![],
    }
}
