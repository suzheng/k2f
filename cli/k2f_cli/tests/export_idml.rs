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

#[test]
fn export_idml_writes_zip() {
    let dir = std::env::temp_dir().join(format!("k2f-idml-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("invoice.idml");
    let export = k2f()
        .args([
            "export-idml",
            repo_root()
                .join("examples/published/invoice.K2F")
                .to_str()
                .unwrap(),
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
    assert!(fs::read(&out).unwrap().starts_with(b"PK"));
}

#[test]
fn export_idml_refuses_idml_as_input() {
    let dir = std::env::temp_dir().join(format!("k2f-idml-rej-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let good = dir.join("invoice.idml");
    assert!(k2f()
        .args([
            "export-idml",
            repo_root()
                .join("examples/published/invoice.K2F")
                .to_str()
                .unwrap(),
            "-o",
            good.to_str().unwrap(),
        ])
        .status()
        .unwrap()
        .success());
    let out = dir.join("out.idml");
    let _ = fs::remove_file(&out);
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
    assert!(
        !out.is_file(),
        "failed export must not write an output file"
    );
}

#[test]
fn help_lists_export_idml() {
    let out = k2f().arg("--help").output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
    assert!(text.contains("export-idml"), "{text}");
}

#[test]
fn export_idml_matches_crate_export_bytes() {
    let invoice = repo_root().join("examples/published/invoice.K2F");
    let bytes = fs::read(&invoice).unwrap();
    let expected = k2f_idml::export_bytes(&bytes).unwrap();

    let dir = std::env::temp_dir().join(format!("k2f-idml-bytes-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("invoice.idml");
    let export = k2f()
        .args([
            "export-idml",
            invoice.to_str().unwrap(),
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
fn export_idml_mimetype_stored_first() {
    let dir = std::env::temp_dir().join(format!("k2f-idml-zip-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("invoice.idml");
    let export = k2f()
        .args([
            "export-idml",
            repo_root()
                .join("examples/published/invoice.K2F")
                .to_str()
                .unwrap(),
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
    let first = archive.by_index(0).unwrap();
    assert_eq!(first.name(), "mimetype");
    assert_eq!(first.compression(), zip::CompressionMethod::Stored);
    assert_eq!(
        std::io::read_to_string(first).unwrap().trim(),
        "application/vnd.adobe.indesign-idml-package"
    );
}
