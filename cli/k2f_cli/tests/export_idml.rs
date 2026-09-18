use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;
use zip::ZipArchive;

fn k2f() -> Command {
    Command::new(env!("CARGO_BIN_EXE_k2f"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn invoice() -> PathBuf {
    repo_root().join("examples/published/invoice.K2F")
}

#[test]
fn export_idml_writes_package_dir() {
    let dir = std::env::temp_dir().join(format!("k2f-idml-pkg-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let out = dir.join("invoice");
    let export = k2f()
        .args(["export-idml", invoice().to_str().unwrap(), "-o", out.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        export.status.success(),
        "{}",
        String::from_utf8_lossy(&export.stderr)
    );
    let idml = out.join("invoice.idml");
    assert!(idml.is_file(), "package idml");
    assert!(
        out.join("Document Fonts").join("Roboto-Regular.ttf").is_file(),
        "Document Fonts face"
    );
    assert!(fs::read(&idml).unwrap().starts_with(b"PK"));
}

#[test]
fn export_idml_only_matches_crate_export_bytes() {
    let invoice = invoice();
    let bytes = fs::read(&invoice).unwrap();
    let expected = k2f_idml::export_bytes(&bytes).unwrap();
    let dir = std::env::temp_dir().join(format!("k2f-idml-only-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("invoice.idml");
    let export = k2f()
        .args([
            "export-idml",
            invoice.to_str().unwrap(),
            "--idml-only",
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        export.status.success(),
        "{}",
        String::from_utf8_lossy(&export.stderr)
    );
    assert_eq!(fs::read(&out).unwrap(), expected);
}

#[test]
fn export_idml_zip_contains_document_fonts() {
    let dir = std::env::temp_dir().join(format!("k2f-idml-zip-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("invoice.zip");
    let export = k2f()
        .args([
            "export-idml",
            invoice().to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        export.status.success(),
        "{}",
        String::from_utf8_lossy(&export.stderr)
    );
    let bytes = fs::read(&out).unwrap();
    let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().replace('\\', "/"))
        .collect();
    assert!(
        names.iter().any(|n| n.contains("Document Fonts") && n.ends_with("Roboto-Regular.ttf")),
        "got {names:?}"
    );
    assert!(names.iter().any(|n| n.ends_with(".idml")), "got {names:?}");
}

#[test]
fn export_idml_refuses_idml_as_input() {
    let dir = std::env::temp_dir().join(format!("k2f-idml-rej-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let good = dir.join("invoice.idml");
    assert!(k2f()
        .args([
            "export-idml",
            invoice().to_str().unwrap(),
            "--idml-only",
            "-o",
            good.to_str().unwrap(),
        ])
        .status()
        .unwrap()
        .success());
    let out = dir.join("out");
    let _ = fs::remove_dir_all(&out);
    let export = k2f()
        .args([
            "export-idml",
            good.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!export.status.success());
    let err = String::from_utf8_lossy(&export.stderr);
    assert!(err.contains("IDML_IS_NOT_A_SOURCE"), "got {err}");
    assert!(!out.exists(), "failed export must not write output");
}

#[test]
fn help_lists_export_idml() {
    let out = k2f().arg("--help").output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
    assert!(text.contains("export-idml"), "{text}");
}

#[test]
fn export_idml_mimetype_stored_first_inside_package() {
    let dir = std::env::temp_dir().join(format!("k2f-idml-mime-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let out = dir.join("invoice");
    let export = k2f()
        .args(["export-idml", invoice().to_str().unwrap(), "-o", out.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        export.status.success(),
        "{}",
        String::from_utf8_lossy(&export.stderr)
    );
    let bytes = fs::read(out.join("invoice.idml")).unwrap();
    let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
    let first = archive.by_index(0).unwrap();
    assert_eq!(first.name(), "mimetype");
    assert_eq!(first.compression(), zip::CompressionMethod::Stored);
    assert_eq!(
        std::io::read_to_string(first).unwrap().trim(),
        "application/vnd.adobe.indesign-idml-package"
    );
}
