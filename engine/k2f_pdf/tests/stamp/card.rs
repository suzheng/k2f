use k2f_core::NodeContent;
use k2f_paint::OpenedDocument;
use k2f_pdf::{export_opened, PdfScale};
use k2f_sdk::Editor;

fn child_count(ed: &Editor) -> usize {
    let root: serde_json::Value = serde_json::from_str(&ed.get_node_json("root").unwrap()).unwrap();
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
    let index = if parent_id == "root" {
        child_count(ed)
    } else {
        0
    };
    ed.insert_node(parent_id, index, &json).unwrap();
}

fn export_card(variant: &str) -> Vec<u8> {
    let mut ed = Editor::open_template("invoice").unwrap();
    insert_text(&mut ed, "root", "card1", "Named card");
    ed.set_role("card1", "card", Some(variant)).unwrap();
    let bytes = ed.save_bytes().unwrap();
    let opened = OpenedDocument::open(&bytes).unwrap();
    export_opened(&opened, PdfScale::DEFAULT).unwrap()
}

#[test]
fn card_raised_exports_via_stamp() {
    let pdf = export_card("raised");
    assert!(pdf.starts_with(b"%PDF-"));
}

#[test]
fn card_glass_exports_via_stamp() {
    let pdf = export_card("glass");
    assert!(pdf.starts_with(b"%PDF-"));
}
