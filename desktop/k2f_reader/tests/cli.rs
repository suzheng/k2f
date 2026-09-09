mod common;

use common::reader_bin;

#[test]
fn help_lists_headless_flags() {
    let out = reader_bin().arg("--help").output().expect("run --help");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("--export-pdf"), "{stdout}");
    assert!(stdout.contains("OUT_PDF"), "{stdout}");
    assert!(stdout.contains("--export-pptx"), "{stdout}");
    assert!(stdout.contains("OUT_PPTX"), "{stdout}");
    assert!(stdout.contains("--export-docx"), "{stdout}");
    assert!(stdout.contains("OUT_DOCX"), "{stdout}");
    assert!(stdout.contains("--verify"), "{stdout}");
    assert!(stdout.contains("[FILE]"), "{stdout}");
}

#[test]
fn no_args_prints_usage_and_exits_2() {
    let out = reader_bin().output().expect("run with no args");
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("usage: k2f-reader"), "{stderr}");
    assert!(stderr.contains("--verify"), "{stderr}");
    assert!(stderr.contains("--export-pdf"), "{stderr}");
    assert!(stderr.contains("--export-pptx"), "{stderr}");
    assert!(stderr.contains("--export-docx"), "{stderr}");
}

#[test]
fn app_bundle_path_is_a_gui_launch() {
    use std::path::Path;
    assert!(k2f_reader::cli::exe_looks_like_gui_launch(Path::new(
        "/tmp/K2F Reader.app/Contents/MacOS/k2f-reader"
    )));
    assert!(k2f_reader::cli::exe_looks_like_gui_launch(Path::new(
        r"C:\Users\me\AppData\Local\K2F Reader\k2f-reader.exe"
    )));
    assert!(!k2f_reader::cli::exe_looks_like_gui_launch(Path::new(
        "target/release/k2f-reader"
    )));
}

#[test]
fn verify_opens_lowercase_k2f_extension() {
    let dir = common::scratch("lower-ext");
    let path = dir.join("invoice.k2f");
    std::fs::write(&path, common::invoice_bytes()).unwrap();
    let out = reader_bin()
        .args(["--verify", path.to_str().unwrap()])
        .output()
        .expect("verify .k2f");
    assert!(
        out.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
}
