use std::io::Cursor;
use std::path::PathBuf;
use zip::ZipArchive;

fn invoice() -> Vec<u8> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/published/invoice.K2F"),
    )
    .unwrap()
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = ZipArchive::new(Cursor::new(docx)).unwrap();
    let mut file = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut file, &mut xml).unwrap();
    xml
}

#[test]
fn export_docx_published_invoice_is_zip() {
    let docx = k2f_sdk::export_docx(&invoice()).unwrap();
    assert!(docx.starts_with(b"PK"));
}

#[test]
fn export_docx_rejects_docx_bytes() {
    let docx = k2f_sdk::export_docx(&invoice()).unwrap();
    let err = k2f_sdk::export_docx(&docx).unwrap_err();
    assert_eq!(err.code, k2f_sdk::DOCX_IS_NOT_A_SOURCE);
}

#[test]
fn editor_export_docx_bytes_after_save() {
    let mut ed = k2f_sdk::Editor::open(&invoice()).unwrap();
    let a = ed.export_docx_bytes().unwrap();
    let saved = ed.save_bytes().unwrap();
    let b = k2f_sdk::export_docx(&saved).unwrap();
    assert_eq!(
        a, b,
        "Editor.export_docx_bytes must draw the in-memory package"
    );
    assert!(a.starts_with(b"PK"));
}

#[test]
fn export_docx_unlocked_template_maps_error_code() {
    let ed = k2f_sdk::Editor::open_dir(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates/invoice"),
    )
    .unwrap();
    let err = ed.export_docx_bytes().unwrap_err();
    assert_eq!(err.code, "UNLOCKED");
}

#[test]
fn export_docx_invoice_contains_semantic_text() {
    let ed = k2f_sdk::Editor::open(&invoice()).unwrap();
    let header = ed.node_text("invoice.header").unwrap();
    let xml = document_xml(&ed.export_docx_bytes().unwrap());
    assert!(
        xml.contains(&header),
        "document.xml must include visible invoice header text"
    );
}
