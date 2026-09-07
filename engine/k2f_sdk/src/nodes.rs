use k2f_core::{
    BreakInside, GridTrack, ListMarkerType, NodeContent, Pt, SemanticNode, TableDataSource,
    TableSpec,
};

pub fn text_node(id: &str, role: &str, text: &str) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: role.to_string(),
        content: NodeContent::Text(text.to_string()),
        ..Default::default()
    }
}

pub fn heading_node(id: &str, text: &str, level: u8) -> SemanticNode {
    let mut n = text_node(id, heading_role(level), text);
    n.keep_with_next = true;
    n
}

pub fn heading_role(level: u8) -> &'static str {
    match level {
        1 => "h1",
        2 => "h2",
        3 => "h3",
        _ => "h4",
    }
}

pub fn warning_node(id: &str, text: &str) -> SemanticNode {
    let mut n = text_node(id, "warning", text);
    n.break_inside = BreakInside::Avoid;
    n
}

pub fn math_node(id: &str, tex: &str) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "math".to_string(),
        break_inside: BreakInside::Avoid,
        content: NodeContent::Math(tex.to_string()),
        ..Default::default()
    }
}

pub fn container_node(id: &str, role: &str) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: role.to_string(),
        content: NodeContent::Container { children: vec![] },
        ..Default::default()
    }
}

pub fn image_node(id: &str, src: &str, width: Pt, height: Pt) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "body".to_string(),
        break_inside: BreakInside::Avoid,
        content: NodeContent::Image {
            src: src.to_string(),
            width,
            height,
        },
        ..Default::default()
    }
}

pub fn list_item_node(
    id: &str,
    list_id: &str,
    text: &str,
    depth: u32,
    marker_type: ListMarkerType,
) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "list_item".to_string(),
        list_id: Some(list_id.to_string()),
        depth: Some(depth),
        marker_type: Some(marker_type),
        content: NodeContent::Text(text.to_string()),
        ..Default::default()
    }
}

pub fn table_node(id: &str, columns: &[String], rows: &[Vec<String>]) -> SemanticNode {
    let cols = columns.len();
    let widths = vec![GridTrack::Fr { fr: 1 }; cols];
    let mut all_rows = Vec::with_capacity(rows.len() + 1);
    all_rows.push(header_row(id, columns));
    for (ri, row) in rows.iter().enumerate() {
        let alt = ri % 2 == 1;
        all_rows.push(body_row(id, ri, row, alt));
    }
    SemanticNode {
        id: id.to_string(),
        role: "table".to_string(),
        content: NodeContent::Table(TableSpec {
            column_widths: widths,
            header_rows: 1,
            gap: 4000,
            row_gap: None,
            column_gap: None,
            data: TableDataSource::Inline { rows: all_rows },
        }),
        ..Default::default()
    }
}

fn header_row(table_id: &str, columns: &[String]) -> Vec<SemanticNode> {
    columns
        .iter()
        .enumerate()
        .map(|(ci, label)| text_node(&format!("{table_id}.h.c{ci}"), "table_header_cell", label))
        .collect()
}

fn body_row(table_id: &str, ri: usize, row: &[String], alt: bool) -> Vec<SemanticNode> {
    row.iter()
        .enumerate()
        .map(|(ci, value)| {
            let mut n = text_node(&format!("{table_id}.r{ri}.c{ci}"), "table_row_cell", value);
            if alt {
                n.variant = Some("alt".to_string());
            }
            n
        })
        .collect()
}
