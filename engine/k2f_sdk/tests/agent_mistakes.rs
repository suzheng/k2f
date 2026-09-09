mod common;

use k2f_core::{NodeContent, Pt, SemanticNode};
use k2f_sdk::{
    validate_agent_json, AgentError, Editor, DUPLICATE_ID, INVALID_ARGUMENT, INVALID_ID,
    SCHEMA_INVALID, TABLE_ROW_MISMATCH, UNKNOWN_ROLE,
};
use serde_json::json;

fn doc() -> Editor {
    common::open("invoice")
}

#[test]
fn duplicate_id_is_rejected() {
    let mut d = doc();
    common::insert_text(&mut d, "root", "n", "body", "a");
    let node = SemanticNode {
        id: "n".to_string(),
        role: "body".to_string(),
        content: NodeContent::Text("b".to_string()),
        ..Default::default()
    };
    let json = serde_json::to_string(&node).unwrap();
    let err = d.insert_node("root", 1, &json).unwrap_err();
    assert_eq!(err.code, DUPLICATE_ID);
}

#[test]
fn unknown_role_is_rejected() {
    let mut d = doc();
    common::insert_text(&mut d, "root", "n", "body", "a");
    let err = d.set_role("n", "magic_box", None).unwrap_err();
    assert_eq!(err.code, UNKNOWN_ROLE);
}

#[test]
fn hyphenated_id_is_rejected() {
    let mut d = doc();
    let node = SemanticNode {
        id: "clause-4".to_string(),
        role: "body".to_string(),
        content: NodeContent::Text("x".to_string()),
        ..Default::default()
    };
    let json = serde_json::to_string(&node).unwrap();
    let err = d.insert_node("root", 0, &json).unwrap_err();
    assert_eq!(err.code, INVALID_ID);
}

#[test]
fn uneven_table_is_rejected() {
    let mut d = doc();
    common::insert_table(
        &mut d,
        "root",
        "t",
        &["a".into(), "b".into()],
        &[vec!["only".into()]],
    );
    let err = d.validate_package().unwrap_err();
    assert_eq!(err.code, TABLE_ROW_MISMATCH);
}

#[test]
fn image_without_positive_mm_is_rejected() {
    let mut d = doc();
    let node = SemanticNode {
        id: "logo".to_string(),
        role: "body".to_string(),
        content: NodeContent::Image {
            src: "assets/images/logo.jpg".to_string(),
            width: Pt(0),
            height: Pt(1000),
        },
        ..Default::default()
    };
    let json = serde_json::to_string(&node).unwrap();
    let err = d.insert_node("root", 0, &json).unwrap_err();
    assert_eq!(err.code, SCHEMA_INVALID);
}

#[test]
fn agent_json_rejects_color_on_node() {
    let err = validate_agent_json(&json!({
        "id": "n",
        "role": "body",
        "color": "#ff0000",
        "content": { "type": "text", "value": "x" }
    }))
    .unwrap_err();
    assert_eq!(err.code, SCHEMA_INVALID);
}

#[test]
fn agent_json_rejects_layout_and_fractional_pt_field() {
    let layout = validate_agent_json(&json!({
        "id": "n",
        "role": "body",
        "layout": { "type": "stack", "gap": 13.5 },
        "content": { "type": "text", "value": "x" }
    }))
    .unwrap_err();
    assert_eq!(layout.code, SCHEMA_INVALID);

    let pt = validate_agent_json(&json!({
        "id": "n",
        "role": "body",
        "font_size": 13.5,
        "content": { "type": "text", "value": "x" }
    }))
    .unwrap_err();
    assert_eq!(pt.code, SCHEMA_INVALID);
}

#[test]
fn agent_json_rejects_code_block() {
    let err = validate_agent_json(&json!({
        "id": "n",
        "role": "body",
        "content": { "type": "code_block", "value": { "code": "x", "language": "rs" } }
    }))
    .unwrap_err();
    assert_eq!(err.code, SCHEMA_INVALID);
}

#[test]
fn agent_json_rejects_color_on_modifier() {
    let err = validate_agent_json(&json!({
        "id": "n",
        "role": "body",
        "modifiers": [{
            "range": [0, 1],
            "type": "emphasis",
            "intent": "critical",
            "color": "#ff0000"
        }],
        "content": { "type": "text", "value": "x" }
    }))
    .unwrap_err();
    assert_eq!(err.code, SCHEMA_INVALID);
}

#[test]
fn save_error_is_machine_readable() {
    let mut ed = common::open("legal");
    let node = SemanticNode {
        id: "bad id".to_string(),
        role: "body".to_string(),
        content: NodeContent::Text("x".to_string()),
        ..Default::default()
    };
    let json = serde_json::to_string(&node).unwrap();
    let err: AgentError = ed.insert_node("root", 0, &json).unwrap_err();
    let s = err.to_string();
    assert!(s.starts_with("INVALID_ID:"), "{s}");
}

#[test]
fn unknown_template_is_rejected() {
    let Err(err) = Editor::open_template("not-a-theme") else {
        panic!("expected unknown template error");
    };
    assert_eq!(err.code, INVALID_ARGUMENT);
    assert!(err.to_string().contains("unknown template"));
}

#[test]
fn clinical_alias_is_not_accepted() {
    let Err(err) = Editor::open_template("clinical") else {
        panic!("expected unknown template error");
    };
    assert_eq!(err.code, INVALID_ARGUMENT);
}

#[test]
fn invoice_allows_lists_and_math() {
    let mut d = doc();
    common::insert_list(&mut d, "root", "invoice.notes", &["Net 30".into()]);
    common::insert_math(&mut d, "root", "invoice.eq", "E=mc^2");
    let bytes = d.save_bytes().unwrap();
    let pkg = k2f_package::unpack_bytes(&bytes).unwrap();
    assert_eq!(
        k2f_package::verify_package(&pkg).unwrap(),
        k2f_package::VerifyStatus::Valid
    );
}
