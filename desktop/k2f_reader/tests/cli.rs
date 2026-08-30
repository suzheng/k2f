mod common;

use common::reader_bin;

#[test]
fn help_lists_headless_flags() {
    let out = reader_bin().arg("--help").output().expect("run --help");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("--export-pdf"), "{stdout}");
    assert!(stdout.contains("OUT_PDF"), "{stdout}");
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
}
