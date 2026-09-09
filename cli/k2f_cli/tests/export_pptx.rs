use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn k2f() -> Command {
    Command::new(env!("CARGO_BIN_EXE_k2f"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn export_pptx_writes_zip() {
    let dir = std::env::temp_dir().join(format!("k2f-pptx-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("invoice.pptx");
    let export = k2f()
        .args([
            "export-pptx",
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
fn export_pptx_refuses_pptx_as_input() {
    let dir = std::env::temp_dir().join(format!("k2f-pptx-rej-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let good = dir.join("invoice.pptx");
    assert!(k2f()
        .args([
            "export-pptx",
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
    let out = dir.join("out.pptx");
    let _ = fs::remove_file(&out);
    let export = k2f()
        .args([
            "export-pptx",
            good.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!export.status.success());
    let err = String::from_utf8_lossy(&export.stderr);
    assert!(err.contains("PPTX_IS_NOT_A_SOURCE"), "got {err}");
    assert!(
        !out.is_file(),
        "failed export must not write an output file"
    );
}

#[test]
fn help_lists_export_pptx() {
    let out = k2f().arg("--help").output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
    assert!(text.contains("export-pptx"), "{text}");
}
