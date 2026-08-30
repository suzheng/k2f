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
fn markdown_file_compiles_to_zip_package() {
    let dir = std::env::temp_dir().join(format!("k2f-md-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let out = dir.join("readme_simple.K2F");
    let convert = k2f()
        .args([
            "markdown",
            repo_root()
                .join("tests/fixtures/markdown/readme_simple.md")
                .to_str()
                .unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        convert.status.success(),
        "{}",
        String::from_utf8_lossy(&convert.stderr)
    );
    let bytes = fs::read(&out).unwrap();
    assert_eq!(&bytes[..2], b"PK");

    let verify = k2f().args(["verify", out.to_str().unwrap()]).output().unwrap();
    assert!(
        verify.status.success(),
        "{}",
        String::from_utf8_lossy(&verify.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&verify.stdout).trim(), "UNSIGNED");
}

#[test]
fn markdown_dir_mirrors_md_to_k2f() {
    let dir = std::env::temp_dir().join(format!("k2f-md-dir-{}", std::process::id()));
    let src = dir.join("src");
    let nested = src.join("nested");
    fs::create_dir_all(&nested).unwrap();
    fs::write(src.join("a.md"), "# A\n\nHello.\n").unwrap();
    fs::write(nested.join("b.md"), "# B\n\nWorld.\n").unwrap();
    let out = dir.join("out");
    let convert = k2f()
        .args([
            "markdown",
            src.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        convert.status.success(),
        "{}",
        String::from_utf8_lossy(&convert.stderr)
    );
    assert!(out.join("a.K2F").is_file());
    assert!(out.join("nested/b.K2F").is_file());
}
