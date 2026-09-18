mod common;

use k2f_core::SemanticNode;
use k2f_paint::OpenedDocument;
use k2f_pdf::{export_opened, PdfExportOptions, PdfScale};
use k2f_sdk::Editor;

fn opts() -> PdfExportOptions {
    PdfExportOptions::new(PdfScale::DEFAULT)
}

fn pack_body(with_underline: bool) -> OpenedDocument {
    let mut ed = Editor::open_dir(&common::repo_root().join("templates/blank")).unwrap();
    let mut node = SemanticNode {
        id: "root.u".into(),
        role: "body".into(),
        content: k2f_core::NodeContent::Text("Heading".into()),
        ..Default::default()
    };
    if with_underline {
        node.modifiers = vec![k2f_core::Modifier {
            range: [0, 7],
            mod_type: "underline".into(),
            intent: "default".into(),
        }];
    }
    ed.insert_node("root", 0, &serde_json::to_string(&node).unwrap())
        .unwrap();
    OpenedDocument::open(&ed.save_bytes().unwrap()).unwrap()
}

fn page0_content(pdf: &[u8]) -> String {
    let parsed = lopdf::Document::load_mem(pdf).unwrap();
    let id = *parsed.get_pages().values().next().unwrap();
    String::from_utf8_lossy(&parsed.get_page_content(id).unwrap()).into_owned()
}

#[test]
fn underline_modifier_writes_decoration_stroke() {
    let with = export_opened(&pack_body(true), opts()).unwrap();
    let without = export_opened(&pack_body(false), opts()).unwrap();
    let with_c = page0_content(&with);
    let without_c = page0_content(&without);
    assert!(
        with_c.contains("% k2f.d "),
        "underlined text must emit a decoration note, got {with_c}"
    );
    assert!(
        !without_c.contains("% k2f.d "),
        "plain text must not emit a decoration stroke"
    );
}
