use k2f_core::find_node;
use k2f_package::{inspect_package, unpack_bytes, IntegrityStatus};
use k2f_paint::{Banner, OpenedDocument};
use k2f_sdk::Editor;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn contract_bytes() -> Vec<u8> {
    std::fs::read(repo_root().join("examples/published/contract.K2F")).unwrap()
}

const CLAUSE_4_AFTER: &str = "本协议自双方签署之日起生效。任何一方终止须提前三十日书面通知对方。终止前须完成已确认工作并结清对应发票。终止不影响本文件中机密、报酬与签署栏条款的效力。";

#[test]
fn edit_contract_clause_4_warning_relocks_unsigned() {
    let mut editor = Editor::open(&contract_bytes()).unwrap();
    let before_2 = editor.node_text("contract.clause_2").unwrap();
    let before_sig = editor.node_text("contract.signatures.client").unwrap();

    editor
        .replace_text("contract.clause_4", CLAUSE_4_AFTER)
        .unwrap();
    editor
        .set_role("contract.clause_4", "critical_warning", None)
        .unwrap();

    let bytes = editor.save_bytes().unwrap();
    let opened = OpenedDocument::open(&bytes).unwrap();
    assert_eq!(opened.banner(), Banner::Unsigned);

    let pkg = unpack_bytes(&bytes).unwrap();
    assert!(pkg.signatures_json.is_none());
    assert_eq!(
        inspect_package(&pkg).unwrap().status,
        IntegrityStatus::Unsigned
    );

    match &find_node(&pkg.root, "contract.clause_4").unwrap().content {
        k2f_core::NodeContent::Text(t) => assert_eq!(t, CLAUSE_4_AFTER),
        other => panic!("{other:?}"),
    }
    assert_eq!(
        find_node(&pkg.root, "contract.clause_4").unwrap().role,
        "critical_warning"
    );
    match &find_node(&pkg.root, "contract.clause_2").unwrap().content {
        k2f_core::NodeContent::Text(t) => assert_eq!(t.as_str(), before_2),
        other => panic!("{other:?}"),
    }
    assert_eq!(
        editor::reopen_text(&bytes, "contract.signatures.client"),
        before_sig
    );
}

mod editor {
    pub fn reopen_text(bytes: &[u8], id: &str) -> String {
        k2f_sdk::Editor::open(bytes).unwrap().node_text(id).unwrap()
    }
}
