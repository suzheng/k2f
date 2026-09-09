use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn k2f() -> Command {
    Command::new(env!("CARGO_BIN_EXE_k2f"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn pack_and_compile(src: &std::path::Path, pkg: &std::path::Path) -> String {
    let pack = k2f()
        .args(["pack", src.to_str().unwrap(), "-o", pkg.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        pack.status.success(),
        "{}",
        String::from_utf8_lossy(&pack.stderr)
    );
    let compile = k2f()
        .args(["compile", pkg.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    String::from_utf8_lossy(&compile.stderr).into_owned()
}

fn write_starter_package(src: &std::path::Path, root_json: &str) {
    fs::create_dir_all(src.join("content")).unwrap();
    fs::create_dir_all(src.join("styles")).unwrap();
    fs::create_dir_all(src.join("assets/fonts")).unwrap();
    let starter = repo_root().join("skills/k2f/starter");
    fs::copy(
        starter.join("styles/theme.json"),
        src.join("styles/theme.json"),
    )
    .unwrap();
    fs::copy(
        starter.join("assets/fonts/Roboto-Regular.ttf"),
        src.join("assets/fonts/Roboto-Regular.ttf"),
    )
    .unwrap();
    fs::write(
        src.join("manifest.json"),
        r#"{
  "title": "slack-fixture",
  "canvas_mode": "paged",
  "page_config": { "width": 595000, "height": 842000, "margin": [0, 0, 0, 0] },
  "engine_version": "0.1.0"
}
"#,
    )
    .unwrap();
    fs::write(src.join("content/root.json"), root_json).unwrap();
}

#[test]
fn compile_prints_layout_slack_for_underfilled_shell() {
    let dir = std::env::temp_dir().join(format!("k2f-slack-stack-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let src = dir.join("src");
    write_starter_package(
        &src,
        r#"{
  "id": "root",
  "role": "document",
  "content": {
    "type": "container",
    "value": {
      "children": [
        {
          "id": "doc.shell",
          "role": "section",
          "break_inside": "avoid",
          "content": {
            "type": "container",
            "value": {
              "children": [
                {
                  "id": "doc.hero",
                  "role": "h1",
                  "content": { "type": "text", "value": "Top only" }
                }
              ]
            }
          },
          "layout": { "type": "stack", "direction": "vertical", "height": 842000 }
        }
      ]
    }
  }
}
"#,
    );
    let err = pack_and_compile(&src, &dir.join("out.K2F"));
    assert!(
        err.contains("LAYOUT_SLACK id=doc.shell"),
        "missing LAYOUT_SLACK: {err}"
    );
    assert!(
        err.contains("hint=nest {fr:1} in the grower; do not pack an auto-height stack"),
        "{err}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn compile_prints_layout_slack_for_hollow_grower() {
    let dir = std::env::temp_dir().join(format!("k2f-slack-grower-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let src = dir.join("src");
    write_starter_package(
        &src,
        r#"{
  "id": "root",
  "role": "document",
  "content": {
    "type": "container",
    "value": {
      "children": [
        {
          "id": "doc.shell",
          "role": "section",
          "break_inside": "avoid",
          "layout": {
            "type": "grid",
            "width": 595000,
            "height": 842000,
            "columns": [{ "fr": 1 }],
            "rows": [{ "auto": true }, { "fr": 1 }, { "auto": true }],
            "gap": 8000,
            "cell_align": { "x": "stretch", "y": "stretch" }
          },
          "content": {
            "type": "container",
            "value": {
              "children": [
                {
                  "id": "doc.header",
                  "role": "h1",
                  "content": { "type": "text", "value": "Header" }
                },
                {
                  "id": "doc.body",
                  "role": "section",
                  "layout": { "type": "stack", "direction": "vertical", "gap": 0 },
                  "content": {
                    "type": "container",
                    "value": {
                      "children": [
                        {
                          "id": "doc.body.hero",
                          "role": "body",
                          "content": { "type": "text", "value": "Short body." }
                        }
                      ]
                    }
                  }
                },
                {
                  "id": "doc.footer",
                  "role": "body",
                  "content": { "type": "text", "value": "Footer" }
                }
              ]
            }
          }
        }
      ]
    }
  }
}
"#,
    );
    let err = pack_and_compile(&src, &dir.join("out.K2F"));
    assert!(
        err.contains("LAYOUT_SLACK id=doc.body"),
        "missing grower LAYOUT_SLACK: {err}"
    );
    assert!(
        !err.contains("LAYOUT_SLACK id=doc.shell"),
        "shell should stay silent when the footer pins the bottom: {err}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn compile_prints_page_underfill_for_short_one_page_stack() {
    let dir = std::env::temp_dir().join(format!("k2f-page-underfill-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let src = dir.join("src");
    write_starter_package(
        &src,
        r#"{
  "id": "root",
  "role": "document",
  "content": {
    "type": "container",
    "value": {
      "children": [
        {
          "id": "doc.title",
          "role": "h1",
          "content": { "type": "text", "value": "Short invoice title" }
        }
      ]
    }
  }
}
"#,
    );
    let err = pack_and_compile(&src, &dir.join("out.K2F"));
    assert!(
        err.contains("PAGE_UNDERFILL page=0"),
        "missing PAGE_UNDERFILL: {err}"
    );
    assert!(
        err.contains("hint=one-page form: copy ex_filled_page.json"),
        "{err}"
    );
    let _ = fs::remove_dir_all(&dir);
}
