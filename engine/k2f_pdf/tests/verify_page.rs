mod common;

use k2f_core::NodeContent;
use k2f_package::{inspect_package, pack_bytes, unpack_bytes, IntegrityStatus};
use k2f_pdf::{export_opened, PdfExportOptions, PdfScale};
use k2f_paint::{Banner, OpenedDocument};
use k2f_sdk::{generate_key, sign, Editor};

fn insert_text(ed: &mut Editor, id: &str, text: &str) {
    let node = k2f_core::SemanticNode {
        id: id.to_string(),
        role: "body".to_string(),
        content: NodeContent::Text(text.to_string()),
        ..Default::default()
    };
    let json = serde_json::to_string(&node).unwrap();
    let root: serde_json::Value =
        serde_json::from_str(&ed.get_node_json("root").unwrap()).unwrap();
    let index = root["content"]["value"]["children"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    ed.insert_node("root", index, &json).unwrap();
}

#[test]
fn signed_pdf_verify_page_lists_hashes_and_status() {
    let mut ed = Editor::open_template("invoice").unwrap();
    ed.set_generated_by("agent.trust-test");
    insert_text(&mut ed, "invoice.total", "100.00");
    let mut pkg = unpack_bytes(&ed.save_bytes().unwrap()).unwrap();
    pkg.manifest.generated_by = Some("agent.trust-test".into());
    let bytes = pack_bytes(&pkg).unwrap();
    let key = generate_key().unwrap();
    let signed = sign(
        &bytes,
        &key.secret_hex,
        Some("Demo Legal"),
        Some(1_704_067_200),
    )
    .unwrap();
    assert_eq!(
        inspect_package(&unpack_bytes(&signed).unwrap())
            .unwrap()
            .status,
        IntegrityStatus::Signed
    );

    let opened = OpenedDocument::open(&signed).unwrap();
    let pdf = export_opened(&opened, PdfExportOptions::new(PdfScale::DEFAULT).with_trust_pack()).unwrap();
    let parsed = lopdf::Document::load_mem(&pdf).unwrap();
    let lock_pages = opened.lock().unwrap().geometry.pages.len();
    assert_eq!(parsed.get_pages().len(), lock_pages + 1);

    let last_id = *parsed.get_pages().values().last().unwrap();
    let content = parsed.get_page_content(last_id).unwrap();
    let text = String::from_utf8_lossy(&content);
    assert!(text.contains("% k2f.verify content_hash:"), "{text}");
    assert!(text.contains("% k2f.verify status: SIGNED"), "{text}");
    assert!(text.contains("% k2f.verify fingerprint:"), "{text}");
}

#[test]
fn default_export_matches_lock_page_count_without_trust_pack() {
    let mut ed = Editor::open_template("invoice").unwrap();
    insert_text(&mut ed, "invoice.total", "42.00");
    let opened = OpenedDocument::open(&ed.save_bytes().unwrap()).unwrap();
    let lock_pages = opened.lock().unwrap().geometry.pages.len();

    let pdf = export_opened(&opened, PdfScale::DEFAULT).unwrap();
    let parsed = lopdf::Document::load_mem(&pdf).unwrap();
    assert_eq!(parsed.get_pages().len(), lock_pages);
    let text = String::from_utf8_lossy(&pdf);
    assert!(!text.contains("appearance_hash="));
    assert!(!text.contains("K2F integrity verification"));
}

#[test]
fn unsigned_pdf_verify_page_matches_banner() {
    let mut ed = Editor::open_template("invoice").unwrap();
    insert_text(&mut ed, "invoice.total", "42.00");
    let opened = OpenedDocument::open(&ed.save_bytes().unwrap()).unwrap();
    assert_eq!(opened.banner(), Banner::Unsigned);

    let pdf = export_opened(&opened, PdfExportOptions::new(PdfScale::DEFAULT).with_trust_pack()).unwrap();
    let parsed = lopdf::Document::load_mem(&pdf).unwrap();
    let last_id = *parsed.get_pages().values().last().unwrap();
    let content = parsed.get_page_content(last_id).unwrap();
    let text = String::from_utf8_lossy(&content);
    assert!(text.contains("% k2f.verify status: UNSIGNED"), "{text}");
}
