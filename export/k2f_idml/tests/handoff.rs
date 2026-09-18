mod common;

use k2f_idml::{
    export_bytes, export_handoff, export_handoff_bytes, export_opened, export_package_zip,
    write_handoff_output, DOCUMENT_FONTS,
};
use std::io::{Cursor, Read};
use zip::ZipArchive;

#[test]
fn invoice_handoff_dir_matches_export_opened() {
    let doc = common::invoice();
    let pkg = export_handoff(&doc).unwrap();
    assert_eq!(pkg.idml, export_opened(&doc).unwrap());
    let roboto = pkg
        .fonts
        .get("Roboto-Regular.ttf")
        .expect("invoice embeds Roboto-Regular.ttf");
    assert_eq!(
        roboto,
        doc.fonts()
            .get("assets/fonts/Roboto-Regular.ttf")
            .expect("package face")
    );
    assert!(!pkg.fonts.keys().any(|k| k.eq_ignore_ascii_case("default")));

    let dir = std::env::temp_dir().join(format!("k2f-idml-handoff-dir-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    pkg.clone().with_stem("invoice").write_dir(&dir).unwrap();
    assert_eq!(
        std::fs::read(dir.join("invoice.idml")).unwrap(),
        pkg.idml
    );
    assert_eq!(
        std::fs::read(dir.join(DOCUMENT_FONTS).join("Roboto-Regular.ttf")).unwrap(),
        *roboto
    );
}

#[test]
fn invoice_handoff_zip_contains_idml_and_document_fonts() {
    let bytes = common::invoice_bytes();
    let zip = export_package_zip(&bytes).unwrap();
    let names = zip_names(&zip);
    let font = names
        .iter()
        .find(|n| n.contains(DOCUMENT_FONTS) && n.ends_with("Roboto-Regular.ttf"))
        .unwrap_or_else(|| panic!("Document Fonts face, got {names:?}"));
    let inner = names
        .iter()
        .find(|n| n.ends_with(".idml"))
        .expect("inner idml");
    let idml = bytes_in(&zip, inner);
    assert_eq!(idml, export_bytes(&bytes).unwrap());
    assert_eq!(
        bytes_in(&zip, font),
        *common::invoice()
            .fonts()
            .get("assets/fonts/Roboto-Regular.ttf")
            .unwrap()
    );
    let (first, method) = common::zip_index0_name_and_method(&idml);
    assert_eq!(first, "mimetype");
    assert_eq!(method, zip::CompressionMethod::Stored);
}

#[test]
fn write_handoff_strips_idml_suffix_to_dir() {
    let pkg = export_handoff_bytes(&common::invoice_bytes()).unwrap();
    let base = std::env::temp_dir().join(format!("k2f-idml-strip-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let requested = base.join("invoice.idml");
    let written = write_handoff_output(&pkg, &requested, false).unwrap();
    assert_eq!(written, base.join("invoice"));
    assert!(written.join("invoice.idml").is_file());
    assert!(written
        .join(DOCUMENT_FONTS)
        .join("Roboto-Regular.ttf")
        .is_file());
    assert!(!requested.is_file());
}

#[test]
fn write_handoff_idml_only_matches_export_bytes() {
    let bytes = common::invoice_bytes();
    let pkg = export_handoff_bytes(&bytes).unwrap();
    let out = std::env::temp_dir().join(format!("k2f-idml-only-{}.idml", std::process::id()));
    let _ = std::fs::remove_file(&out);
    write_handoff_output(&pkg, &out, true).unwrap();
    assert_eq!(std::fs::read(&out).unwrap(), export_bytes(&bytes).unwrap());
}

#[test]
fn handoff_rejects_idml_source() {
    let idml = export_bytes(&common::invoice_bytes()).unwrap();
    let err = export_handoff_bytes(&idml).unwrap_err();
    assert!(err.to_string().contains("IDML_IS_NOT_A_SOURCE"));
}

fn zip_names(bytes: &[u8]) -> Vec<String> {
    let mut zip = ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut names: Vec<String> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().replace('\\', "/"))
        .collect();
    names.sort();
    names
}

fn bytes_in(zip: &[u8], name: &str) -> Vec<u8> {
    let mut z = ZipArchive::new(Cursor::new(zip)).unwrap();
    let mut f = z.by_name(name).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    buf
}
