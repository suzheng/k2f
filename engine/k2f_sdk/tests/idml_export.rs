use std::io::Cursor;
use std::path::PathBuf;
use zip::ZipArchive;

fn invoice() -> Vec<u8> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/published/invoice.K2F"),
    )
    .unwrap()
}

/// Concatenate every Stories/*.xml part. IDML body text lives here, not in Spreads.
fn stories_xml(idml: &[u8]) -> String {
    let mut zip = ZipArchive::new(Cursor::new(idml)).unwrap();
    let mut names: Vec<_> = zip
        .file_names()
        .filter(|n| {
            let n = n.replace('\\', "/");
            n.starts_with("Stories/") && n.ends_with(".xml")
        })
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
fn export_idml_published_invoice_is_zip() {
    let idml = k2f_sdk::export_idml(&invoice()).unwrap();
    assert!(idml.starts_with(b"PK"));
}

#[test]
fn export_idml_rejects_idml_bytes() {
    let idml = k2f_sdk::export_idml(&invoice()).unwrap();
    let err = k2f_sdk::export_idml(&idml).unwrap_err();
    assert_eq!(err.code, k2f_sdk::IDML_IS_NOT_A_SOURCE);
}

#[test]
fn editor_export_idml_bytes_after_save() {
    let mut ed = k2f_sdk::Editor::open(&invoice()).unwrap();
    let a = ed.export_idml_bytes().unwrap();
    let saved = ed.save_bytes().unwrap();
    let b = k2f_sdk::export_idml(&saved).unwrap();
    assert_eq!(
        a, b,
        "Editor.export_idml_bytes must draw the in-memory package"
    );
    assert!(a.starts_with(b"PK"));
}

#[test]
fn export_idml_unlocked_template_maps_error_code() {
    let ed = k2f_sdk::Editor::open_dir(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates/invoice"),
    )
    .unwrap();
    let err = ed.export_idml_bytes().unwrap_err();
    assert_eq!(err.code, "UNLOCKED");
}

#[test]
fn export_idml_invoice_contains_semantic_text() {
    let ed = k2f_sdk::Editor::open(&invoice()).unwrap();
    let header = ed.node_text("invoice.header").unwrap();
    let xml = stories_xml(&ed.export_idml_bytes().unwrap());
    assert!(
        xml.contains(&header),
        "Stories XML must include visible invoice header text"
    );
}
