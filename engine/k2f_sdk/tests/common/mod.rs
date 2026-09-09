//! Integration-test helpers (not public SDK API).
use k2f_core::{GridTrack, ListMarkerType, NodeContent, SemanticNode, TableDataSource, TableSpec};
use k2f_sdk::Editor;
use std::path::PathBuf;

fn templates_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates")
}

pub fn open(name: &str) -> Editor {
    Editor::open_dir(&templates_root().join(name)).unwrap()
}

pub fn child_count(ed: &Editor) -> usize {
    let root: serde_json::Value = serde_json::from_str(&ed.get_node_json("root").unwrap()).unwrap();
    root["content"]["value"]["children"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0)
}

pub fn insert_node(ed: &mut Editor, parent_id: &str, node: &SemanticNode) {
    let json = serde_json::to_string(node).unwrap();
    let index = if parent_id == "root" {
        child_count(ed)
    } else {
        let parent: serde_json::Value =
            serde_json::from_str(&ed.get_node_json(parent_id).unwrap()).unwrap();
        parent["content"]["value"]["children"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0)
    };
    ed.insert_node(parent_id, index, &json).unwrap();
}

pub fn insert_text(ed: &mut Editor, parent_id: &str, id: &str, role: &str, text: &str) {
    insert_node(ed, parent_id, &text_node(id, role, text));
}

pub fn insert_heading(ed: &mut Editor, parent_id: &str, id: &str, level: u8, text: &str) {
    let role = match level {
        1 => "h1",
        2 => "h2",
        3 => "h3",
        _ => "h4",
    };
    let mut node = text_node(id, role, text);
    node.keep_with_next = true;
    insert_node(ed, parent_id, &node);
}

pub fn insert_warning(ed: &mut Editor, parent_id: &str, id: &str, text: &str) {
    let mut node = text_node(id, "warning", text);
    node.break_inside = k2f_core::BreakInside::Avoid;
    insert_node(ed, parent_id, &node);
}

pub fn insert_math(ed: &mut Editor, parent_id: &str, id: &str, tex: &str) {
    insert_node(
        ed,
        parent_id,
        &SemanticNode {
            id: id.to_string(),
            role: "math".to_string(),
            break_inside: k2f_core::BreakInside::Avoid,
            content: NodeContent::Math(tex.to_string()),
            ..Default::default()
        },
    );
}

pub fn insert_table(
    ed: &mut Editor,
    parent_id: &str,
    id: &str,
    columns: &[String],
    rows: &[Vec<String>],
) {
    insert_node(ed, parent_id, &table_node(id, columns, rows));
}

pub fn insert_section(ed: &mut Editor, id: &str) -> String {
    insert_node(
        ed,
        "root",
        &SemanticNode {
            id: id.to_string(),
            role: "section".to_string(),
            content: NodeContent::Container { children: vec![] },
            ..Default::default()
        },
    );
    id.to_string()
}

pub fn insert_columns(ed: &mut Editor, id: &str, count: u32, gap_pt: i64) -> String {
    insert_node(
        ed,
        "root",
        &SemanticNode {
            id: id.to_string(),
            role: "section".to_string(),
            content: NodeContent::Container { children: vec![] },
            layout: Some(k2f_core::LayoutHint::Columns { count, gap: gap_pt }),
            ..Default::default()
        },
    );
    id.to_string()
}

pub fn insert_list(ed: &mut Editor, parent_id: &str, id: &str, items: &[String]) {
    let children: Vec<SemanticNode> = items
        .iter()
        .enumerate()
        .map(|(i, item)| SemanticNode {
            id: format!("{id}.i{i}"),
            role: "list_item".to_string(),
            list_id: Some(id.to_string()),
            depth: Some(0),
            marker_type: Some(ListMarkerType::Bullet),
            content: NodeContent::Text(item.clone()),
            ..Default::default()
        })
        .collect();
    insert_node(
        ed,
        parent_id,
        &SemanticNode {
            id: id.to_string(),
            role: "section".to_string(),
            content: NodeContent::Container { children },
            ..Default::default()
        },
    );
}

fn text_node(id: &str, role: &str, text: &str) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: role.to_string(),
        content: NodeContent::Text(text.to_string()),
        ..Default::default()
    }
}

fn table_node(id: &str, columns: &[String], rows: &[Vec<String>]) -> SemanticNode {
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
