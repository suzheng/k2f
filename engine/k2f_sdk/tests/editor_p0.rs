use k2f_package::unpack_bytes;
use k2f_paint::{Banner, OpenedDocument};
use k2f_sdk::{ChangeOp, ChangelogFile, Editor, CONTENT_HASH_MISMATCH};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn contract_bytes() -> Vec<u8> {
    std::fs::read(repo_root().join("examples/published/contract.K2F")).unwrap()
}

const CLAUSE_4_AFTER: &str = "本协议自双方签署之日起生效。任何一方终止须提前三十日书面通知对方。终止前须完成已确认工作并结清对应发票。终止不影响本文件中机密、报酬与签署栏条款的效力。";

#[test]
fn diff_and_changelog_on_clause_edit() {
    let mut editor = Editor::open(&contract_bytes()).unwrap();
    editor
        .replace_text("contract.clause_4", CLAUSE_4_AFTER)
        .unwrap();
    editor
        .set_role("contract.clause_4", "critical_warning", None)
        .unwrap();

    let diff = editor.diff();
    assert_eq!(diff.len(), 1);
    assert_eq!(diff[0].id, "contract.clause_4");
    assert_eq!(diff[0].op, ChangeOp::Update);

    editor.set_generated_by("demo-agent");
    let bytes = editor.save_bytes().unwrap();
    let pkg = unpack_bytes(&bytes).unwrap();
    let log: ChangelogFile = serde_json::from_str(&pkg.changelog_json).unwrap();
    assert_eq!(log.entries.len(), 1);
    assert_eq!(log.entries[0].agent.as_deref(), Some("demo-agent"));
    assert_eq!(log.entries[0].changes.len(), 1);
    assert_eq!(
        OpenedDocument::open(&bytes).unwrap().banner(),
        Banner::Unsigned
    );
}

#[test]
fn save_with_wrong_hash_rejects() {
    let mut editor = Editor::open(&contract_bytes()).unwrap();
    editor
        .replace_text("contract.clause_4", CLAUSE_4_AFTER)
        .unwrap();
    let err = editor
        .save_with(Some(
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
        ))
        .unwrap_err();
    assert_eq!(err.code, CONTENT_HASH_MISMATCH);
}

#[test]
fn insert_and_delete_node_roundtrip() {
    let mut editor = Editor::open(&contract_bytes()).unwrap();
    let node_json = r#"{"id":"contract.demo_clause","role":"body","content":{"type":"text","value":"Demo insert."}}"#;
    editor
        .insert_node("root", 1, node_json)
        .expect("insert under root container");
    assert!(editor.diff().iter().any(|c| c.id == "contract.demo_clause"));
    editor.delete_node("contract.demo_clause").unwrap();
    assert!(editor.diff().is_empty());
}
