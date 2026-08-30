use super::super::node_builders::{default_page_config, text_node};
use k2f_core::{CanvasMode, Manifest, Modifier};

pub fn modifier_overlap_matrix() -> Manifest {
    let text = "abcdef";
    Manifest {
        title: "Modifier Overlap Matrix".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: {
            use k2f_core::{NodeContent, SemanticNode};
            SemanticNode {
                id: "root".to_string(),
                role: "body".to_string(),
                variant: None,
                preserve_whitespace: None,
                list_id: None,
                depth: None,
                marker_type: None,
                content: NodeContent::Text(text.to_string()),
                modifiers: vec![
                    Modifier {
                        range: [0, text.len()],
                        mod_type: "underline".to_string(),
                        intent: "x".to_string(),
                    },
                    Modifier {
                        range: [0, text.len()],
                        mod_type: "emphasis".to_string(),
                        intent: "x".to_string(),
                    },
                ],
                layout: None,
                ..Default::default()
            }
        },
        running_blocks: vec![],
    }
}

pub fn modifier_precedence_matrix() -> Manifest {
    let text = "Modifier precedence check: underline vs emphasis.";
    let mut node = text_node("root", text);
    node.role = "body".to_string();
    node.modifiers = vec![
        Modifier {
            range: [0, text.len()],
            mod_type: "emphasis".to_string(),
            intent: "critical".to_string(),
        },
        Modifier {
            range: [0, text.len()],
            mod_type: "underline".to_string(),
            intent: "on".to_string(),
        },
    ];

    Manifest {
        title: "Modifier Precedence Matrix (Theme-Patched Font Sizes)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: node,
        running_blocks: vec![],
    }
}

pub fn modifier_role_override() -> Manifest {
    let text = "Role override via unknown modifier type.";
    let mut node = text_node("root", text);
    node.role = "body".to_string();
    node.modifiers = vec![Modifier {
        range: [0, text.len()],
        mod_type: "emphasis".to_string(),
        intent: "strong".to_string(),
    }];

    Manifest {
        title: "Modifier: Role Override Fallback".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: node,
        running_blocks: vec![],
    }
}

pub fn modifier_adjacent_segmentation() -> Manifest {
    // Test adjacent modifiers that create separate runs with correct boundaries.
    // Text: "HelloWorld" (10 chars)
    // Modifier 1: [0, 5] -> "Hello" (font_size: 18pt)
    // Modifier 2: [5, 10] -> "World" (font_size: 24pt)
    // These are adjacent (not overlapping), so they should create two separate runs.
    let text = "HelloWorld";
    let mut node = text_node("root", text);
    node.role = "body".to_string();
    node.modifiers = vec![
        Modifier {
            range: [0, 5],
            mod_type: "emphasis".to_string(),
            intent: "size_18".to_string(),
        },
        Modifier {
            range: [5, 10],
            mod_type: "emphasis".to_string(),
            intent: "size_24".to_string(),
        },
    ];

    Manifest {
        title: "Modifier: Adjacent Segmentation (Run Splitting)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: node,
        running_blocks: vec![],
    }
}
