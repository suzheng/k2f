use k2f_core::NodeContent;
use k2f_paint::OpenedDocument;
use k2f_sdk::Editor;

fn child_count(ed: &Editor) -> usize {
    let root: serde_json::Value =
        serde_json::from_str(&ed.get_node_json("root").unwrap()).unwrap();
    root["content"]["value"]["children"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0)
}

fn insert_text(ed: &mut Editor, parent_id: &str, id: &str, text: &str) {
    let node = k2f_core::SemanticNode {
        id: id.to_string(),
        role: "body".to_string(),
        content: NodeContent::Text(text.to_string()),
        ..Default::default()
    };
    let json = serde_json::to_string(&node).unwrap();
    ed.insert_node(parent_id, child_count(ed), &json).unwrap();
}

pub fn glass_card_bytes() -> Vec<u8> {
    let mut ed = Editor::open_template("invoice").unwrap();
    insert_text(&mut ed, "root", "card1", "Glass card body");
    ed.set_role("card1", "card", Some("glass")).unwrap();
    ed.save_bytes().unwrap()
}

pub fn glass_card() -> OpenedDocument {
    OpenedDocument::open(&glass_card_bytes()).unwrap()
}
