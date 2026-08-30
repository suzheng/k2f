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
fn compile_prints_layout_slack_for_underfilled_shell() {
    let dir = std::env::temp_dir().join(format!("k2f-slack-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let src = dir.join("src");
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
    fs::write(
        src.join("content/root.json"),
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
    )
    .unwrap();

    let pkg = dir.join("out.K2F");
    let pack = k2f()
        .args(["pack", src.to_str().unwrap(), "-o", pkg.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(pack.status.success(), "{}", String::from_utf8_lossy(&pack.stderr));

    let compile = k2f()
        .args(["compile", pkg.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let err = String::from_utf8_lossy(&compile.stderr);
    assert!(
        err.contains("LAYOUT_SLACK id=doc.shell"),
        "missing LAYOUT_SLACK: {err}"
    );
    assert!(err.contains("hint=use {fr:1} body row"), "{err}");

    let _ = fs::remove_dir_all(&dir);
}
