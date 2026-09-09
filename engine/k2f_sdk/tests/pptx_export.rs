use std::io::Cursor;
use std::path::PathBuf;
use zip::ZipArchive;

fn invoice() -> Vec<u8> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/published/invoice.K2F"),
    )
    .unwrap()
}

fn slides_xml(pptx: &[u8]) -> String {
    let mut zip = ZipArchive::new(Cursor::new(pptx)).unwrap();
    let mut names: Vec<_> = zip
        .file_names()
        .filter(|n| n.starts_with("ppt/slides/slide") && n.ends_with(".xml"))
        .map(str::to_string)
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let mut file = zip.by_name(&name).unwrap();
            let mut xml = String::new();
            std::io::Read::read_to_string(&mut file, &mut xml).unwrap();
            xml
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn export_pptx_published_invoice_is_zip() {
    let pptx = k2f_sdk::export_pptx(&invoice()).unwrap();
    assert!(pptx.starts_with(b"PK"));
}

#[test]
fn export_pptx_rejects_pptx_bytes() {
    let pptx = k2f_sdk::export_pptx(&invoice()).unwrap();
    let err = k2f_sdk::export_pptx(&pptx).unwrap_err();
    assert_eq!(err.code, k2f_sdk::PPTX_IS_NOT_A_SOURCE);
}

#[test]
fn editor_export_pptx_bytes_after_save() {
    let mut ed = k2f_sdk::Editor::open(&invoice()).unwrap();
    let a = ed.export_pptx_bytes().unwrap();
    let saved = ed.save_bytes().unwrap();
    let b = k2f_sdk::export_pptx(&saved).unwrap();
    assert_eq!(
        a, b,
        "Editor.export_pptx_bytes must draw the in-memory package"
    );
    assert!(a.starts_with(b"PK"));
}

#[test]
fn editor_edit_appears_in_pptx_after_save() {
    let mut ed = k2f_sdk::Editor::open(&invoice()).unwrap();
    let token = "UNIQUE_SDK_PPTX_RELOCK_TOKEN";
    ed.replace_text("invoice.header", token).unwrap();
    // PPTX maps lock glyph ranges onto semantic text; dirty tree/lock pairs
    // are not a stable export. Relock (save) is the contract.
    let saved = ed.save_bytes().unwrap();
    let saved_xml = slides_xml(&k2f_sdk::export_pptx(&saved).unwrap());
    assert!(
        saved_xml.contains(token),
        "relocked export must keep edited header text"
    );
}

#[test]
fn export_pptx_unlocked_template_maps_error_code() {
    let ed = k2f_sdk::Editor::open_dir(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates/invoice"),
    )
    .unwrap();
    let err = ed.export_pptx_bytes().unwrap_err();
    assert_eq!(err.code, "UNLOCKED");
}

#[test]
fn export_pptx_invoice_contains_semantic_text() {
    let ed = k2f_sdk::Editor::open(&invoice()).unwrap();
    let header = ed.node_text("invoice.header").unwrap();
    let xml = slides_xml(&ed.export_pptx_bytes().unwrap());
    assert!(
        xml.contains(&header),
        "pptx slide XML must include visible invoice header text"
    );
}
