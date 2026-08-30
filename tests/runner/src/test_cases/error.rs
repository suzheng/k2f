use super::super::node_builders::{container_node, default_page_config, grid_container, text_node};
use k2f_core::{
    CanvasMode, GridTrack, Manifest, Modifier, NodeContent, PageConfig, Pt, SemanticNode,
};

pub fn error_modifier_limit_exceeded() -> Manifest {
    let text = "hello";
    let mut node = text_node("root", text);
    let mut mods = Vec::new();
    for i in 0..51 {
        mods.push(Modifier {
            range: [0, 1],
            mod_type: "emphasis".to_string(),
            intent: format!("c{}", i),
        });
    }
    node.modifiers = mods;
    Manifest {
        title: "Error: Modifier Limit Exceeded".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: node,
        running_blocks: vec![],
    }
}

pub fn error_modifier_range_invalid() -> Manifest {
    let text = "hello";
    let mut node = text_node("root", text);
    node.modifiers = vec![Modifier {
        range: [0, 999],
        mod_type: "emphasis".to_string(),
        intent: "red".to_string(),
    }];
    Manifest {
        title: "Error: Modifier Range Invalid".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: node,
        running_blocks: vec![],
    }
}

pub fn error_modifier_range_not_on_char_boundary() -> Manifest {
    // "🙂" is multi-byte; [1,2] is inside the codepoint and is not a UTF-8 boundary.
    let text = "a🙂b";
    let mut node = text_node("root", text);
    node.modifiers = vec![Modifier {
        range: [1, 2],
        mod_type: "emphasis".to_string(),
        intent: "red".to_string(),
    }];
    Manifest {
        title: "Error: Modifier Range Not on Char Boundary".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: node,
        running_blocks: vec![],
    }
}

pub fn error_grid_too_many_children() -> Manifest {
    let grid = grid_container(
        "grid",
        "box",
        vec![GridTrack::Pt { pt: 100000 }],
        vec![GridTrack::Pt { pt: 100000 }],
        0,
        vec![text_node("a", "A"), text_node("b", "B")],
    );
    Manifest {
        title: "Error: Grid Too Many Children".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: grid,
        running_blocks: vec![],
    }
}

pub fn error_image_non_positive_size() -> Manifest {
    let node = SemanticNode {
        id: "img".to_string(),
        role: "body".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Image {
            src: "asset://bad".to_string(),
            width: Pt(0),
            height: Pt(1000),
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };
    Manifest {
        title: "Error: Image Non-Positive Size".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: node,
        running_blocks: vec![],
    }
}

pub fn error_table_reference_non_positive_size() -> Manifest {
    let node = SemanticNode {
        id: "tbl".to_string(),
        role: "table".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::TableReference {
            source: "asset://bad".to_string(),
            view_mode: "full".to_string(),
            width: Pt(1000),
            height: Pt(0),
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };
    Manifest {
        title: "Error: TableReference Non-Positive Size".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: node,
        running_blocks: vec![],
    }
}

pub fn error_grid_fr_rows_unbounded() -> Manifest {
    // Semantic validation allows this, but layout must reject resolving fr rows with infinite height.
    let grid = grid_container(
        "grid",
        "box",
        vec![GridTrack::Pt { pt: 200000 }],
        vec![GridTrack::Fr { fr: 1 }],
        0,
        vec![text_node("cell", "Cell")],
    );
    Manifest {
        title: "Error: Grid Fr Rows Unbounded".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        // Root-level container is paginated by its immediate children, so we must put the grid
        // as a *child* item to ensure the grid layout codepath runs.
        root: container_node("root", "document", vec![grid]),
        running_blocks: vec![],
    }
}

pub fn error_page_config_negative_margin() -> Manifest {
    Manifest {
        title: "Error: PageConfig Negative Margin".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: PageConfig {
            width: Pt(200000),
            height: Pt(200000),
            margin: [Pt(-1), Pt::ZERO, Pt::ZERO, Pt::ZERO],
        },
        root: text_node("root", "Bad margin"),
        running_blocks: vec![],
    }
}

pub fn error_page_config_negative_content_height() -> Manifest {
    Manifest {
        title: "Error: PageConfig Negative Content Height".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: PageConfig {
            width: Pt(200000),
            height: Pt(100000),
            margin: [Pt(60000), Pt::ZERO, Pt(60000), Pt::ZERO],
        },
        root: text_node("root", "Bad content height"),
        running_blocks: vec![],
    }
}
