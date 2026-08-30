mod common;

use k2f_core::{ColumnSpan, NodeContent, SemanticNode};
use k2f_sdk::{validate_agent_json, SCHEMA_INVALID};
use serde_json::json;

#[test]
fn begin_columns_passes_agent_schema() {
    let mut ed = common::open("report");
    common::insert_heading(&mut ed, "root", "paper.title", 1, "Title");
    let cols = common::insert_columns(&mut ed, "paper.body", 2, 12_000);
    common::insert_text(&mut ed, &cols, "paper.p1", "body", "Column body text.");
    ed.validate_package().unwrap();
}

#[test]
fn agent_schema_still_rejects_stack_layout() {
    let err = validate_agent_json(&json!({
        "id": "x",
        "role": "section",
        "layout": { "type": "stack", "gap": 0 },
        "content": { "type": "container", "value": { "children": [] } }
    }))
    .unwrap_err();
    assert_eq!(err.code, SCHEMA_INVALID);
}

#[test]
fn set_column_span_marks_node() {
    let mut ed = common::open("report");
    let cols = common::insert_columns(&mut ed, "paper.body", 2, 0);
    common::insert_node(
        &mut ed,
        &cols,
        &SemanticNode {
            id: "paper.fig".to_string(),
            role: "body".to_string(),
            column_span: ColumnSpan::All,
            content: NodeContent::Text("figure placeholder".to_string()),
            ..Default::default()
        },
    );
    let json: serde_json::Value =
        serde_json::from_str(&ed.get_node_json("paper.fig").unwrap()).unwrap();
    assert_eq!(json["column_span"], "all");
    ed.validate_package().unwrap();
}
