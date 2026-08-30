use k2f_core::{
    Align, CellAlign, FixedSizeHint, GridTrack, JustifyContent, LayoutHint, NodeContent,
    PageConfig, Pt, SemanticNode, StackDirection,
};

pub fn default_page_config() -> PageConfig {
    PageConfig {
        width: Pt(595000),  // A4
        height: Pt(842000), // A4
        margin: [Pt(72000); 4],
    }
}

pub fn text_node(id: &str, text: &str) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "text".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Text(text.to_string()),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    }
}

pub fn container_node(id: &str, role: &str, children: Vec<SemanticNode>) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: role.to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container { children },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    }
}

pub fn container_node_with_variant(
    id: &str,
    role: &str,
    variant: Option<&str>,
    children: Vec<SemanticNode>,
) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: role.to_string(),
        variant: variant.map(|v| v.to_string()),
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container { children },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    }
}

pub fn text_node_role(id: &str, role: &str, text: &str) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: role.to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Text(text.to_string()),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    }
}

pub fn image_node(id: &str, width: Pt, height: Pt) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "body".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Image {
            src: format!("asset://{}", id),
            width,
            height,
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    }
}

pub fn table_ref_node(id: &str, width: Pt, height: Pt) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "table".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::TableReference {
            source: format!("asset://{}", id),
            view_mode: "full".to_string(),
            width,
            height,
        },
        modifiers: vec![],
        layout: None,
        ..Default::default()
    }
}

pub fn stack_container(
    id: &str,
    role: &str,
    direction: StackDirection,
    gap: i64,
    children: Vec<SemanticNode>,
) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: role.to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container { children },
        modifiers: vec![],
        layout: Some(LayoutHint::Stack {
            direction,
            gap,
            align_items: Align::default(),
            justify_content: JustifyContent::default(),
            size: FixedSizeHint::default(),
        }),
        ..Default::default()
    }
}

pub fn overlay_container(id: &str, role: &str, children: Vec<SemanticNode>) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: role.to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container { children },
        modifiers: vec![],
        layout: Some(LayoutHint::Overlay {
            size: FixedSizeHint::default(),
        }),
        ..Default::default()
    }
}

pub fn grid_container(
    id: &str,
    role: &str,
    columns: Vec<GridTrack>,
    rows: Vec<GridTrack>,
    gap: i64,
    children: Vec<SemanticNode>,
) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: role.to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container { children },
        modifiers: vec![],
        layout: Some(LayoutHint::Grid {
            columns,
            rows,
            gap,
            row_gap: None,
            column_gap: None,
            cell_align: Some(CellAlign::default()),
            size: Default::default(),
        }),
        ..Default::default()
    }
}
