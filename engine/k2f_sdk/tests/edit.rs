mod common;

use k2f_package::{inspect_package, unpack_bytes, IntegrityStatus, VerifyStatus};
use k2f_paint::{Banner, OpenedDocument};
use k2f_sdk::{generate_key, sign, Editor};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn invoice_bytes() -> Vec<u8> {
    std::fs::read(repo_root().join("examples/published/invoice.K2F")).unwrap()
}

#[test]
fn replace_invoice_total_relocks_and_drops_signature() {
    let mut ed = common::open("invoice");
    common::insert_text(&mut ed, "root", "invoice.total", "body", "100");
    let unsigned = ed.save_bytes().unwrap();
    let key = generate_key().unwrap();
    let signed = sign(
        &unsigned,
        &key.secret_hex,
        Some("Jane Doe"),
        Some(1_704_067_200),
    )
    .unwrap();
    assert_eq!(
        OpenedDocument::open(&signed).unwrap().banner(),
        Banner::Signed
    );

    let mut editor = Editor::open(&signed).unwrap();
    assert_eq!(editor.node_text("invoice.total").unwrap(), "100");
    editor.replace_text("invoice.total", "110").unwrap();
    assert_eq!(editor.node_text("invoice.total").unwrap(), "110");
    let json: serde_json::Value =
        serde_json::from_str(&editor.get_node_json("invoice.total").unwrap()).unwrap();
    assert_eq!(json["content"]["value"], "110");

    let relocked = editor.save_bytes().unwrap();
    let opened = OpenedDocument::open(&relocked).unwrap();
    assert_eq!(opened.banner(), Banner::Unsigned);
    assert_eq!(opened.status(), VerifyStatus::Valid);
    let pkg = unpack_bytes(&relocked).unwrap();
    assert!(pkg.signatures_json.is_none());
    assert_eq!(
        inspect_package(&pkg).unwrap().status,
        IntegrityStatus::Unsigned
    );
    assert!(k2f_core::find_node(&pkg.root, "invoice.total").is_some());
}

#[test]
fn real_invoice_total_edit_keeps_other_ids() {
    let mut editor = Editor::open(&invoice_bytes()).unwrap();
    let before = editor.node_text("invoice.header").unwrap();
    editor
        .replace_text("invoice.total", "Grand Total: $110.00")
        .unwrap();
    let bytes = editor.save_bytes().unwrap();
    let pkg = unpack_bytes(&bytes).unwrap();
    assert_eq!(verify_ok(&pkg), VerifyStatus::Valid);
    match &k2f_core::find_node(&pkg.root, "invoice.total")
        .unwrap()
        .content
    {
        k2f_core::NodeContent::Text(t) => assert_eq!(t, "Grand Total: $110.00"),
        other => panic!("{other:?}"),
    }
    match &k2f_core::find_node(&pkg.root, "invoice.header")
        .unwrap()
        .content
    {
        k2f_core::NodeContent::Text(t) => assert_eq!(t.as_str(), before),
        other => panic!("{other:?}"),
    }
    assert!(k2f_core::find_node(&pkg.root, "invoice.logo").is_some());
    assert!(k2f_core::find_node(&pkg.root, "invoice.table").is_some());
}

#[test]
fn invoice_cell_edit_writes_asset_and_relocks() {
    let mut editor = Editor::open(&invoice_bytes()).unwrap();
    let before = editor.node_text("invoice.row_1.amount").unwrap();
    assert_ne!(before, "110");
    editor.replace_text("invoice.row_1.amount", "110").unwrap();
    let bytes = editor.save_bytes().unwrap();
    let pkg = unpack_bytes(&bytes).unwrap();
    assert_eq!(verify_ok(&pkg), VerifyStatus::Valid);
    match &k2f_core::find_node(&pkg.root, "invoice.table")
        .unwrap()
        .content
    {
        k2f_core::NodeContent::Table(spec) => match &spec.data {
            k2f_core::TableDataSource::Asset { source } => {
                assert_eq!(source, "assets/data/invoice_data.json")
            }
            other => panic!("table should stay asset-backed, got {other:?}"),
        },
        other => panic!("{other:?}"),
    }
    let again = Editor::open(&bytes).unwrap();
    assert_eq!(again.node_text("invoice.row_1.amount").unwrap(), "110");
    assert_eq!(
        again
            .node_text("invoice.header")
            .unwrap()
            .contains("STATEMENT"),
        true
    );
}

fn verify_ok(pkg: &k2f_package::Package) -> VerifyStatus {
    k2f_package::verify_package(pkg).unwrap()
}

#[test]
fn suggestion_accept_writes_text_reject_vanishes() {
    let mut editor = Editor::open(&invoice_bytes()).unwrap();
    editor
        .suggest("invoice.total", "Grand Total: $1.00")
        .unwrap();
    assert!(editor.suggestions_json().unwrap().contains("invoice.total"));
    editor.reject_suggestion("invoice.total").unwrap();
    assert_eq!(editor.suggestions_json().unwrap(), "{}");
    editor
        .suggest("invoice.total", "Grand Total: $2.00")
        .unwrap();
    editor.accept_suggestion("invoice.total").unwrap();
    assert_eq!(
        editor.node_text("invoice.total").unwrap(),
        "Grand Total: $2.00"
    );
    assert_eq!(editor.suggestions_json().unwrap(), "{}");
}

#[test]
fn set_role_uses_package_theme_not_lock_geometry() {
    let mut editor = Editor::open(&invoice_bytes()).unwrap();
    editor.set_role("invoice.details", "h1", None).unwrap();
    let sel = editor.selection("invoice.details").unwrap();
    assert_eq!(sel.role, "h1");
    assert_eq!(sel.node["keep_with_next"], true);
    assert!(sel.node.get("x").is_none());
    let err = editor
        .set_role("invoice.details", "not_a_role", None)
        .unwrap_err();
    assert_eq!(err.code, "UNKNOWN_ROLE");
}

#[test]
fn editor_set_role_warning_avoids_page_split() {
    let mut ed = common::open("invoice");
    common::insert_text(&mut ed, "root", "invoice.details", "body", "Net 30");
    let mut editor = Editor::open(&ed.save_bytes().unwrap()).unwrap();
    editor.set_role("invoice.details", "warning", None).unwrap();
    let sel = editor.selection("invoice.details").unwrap();
    assert_eq!(sel.role, "warning");
    assert_eq!(sel.node["break_inside"], "avoid");
    let pkg = unpack_bytes(&editor.save_bytes().unwrap()).unwrap();
    assert_eq!(verify_ok(&pkg), VerifyStatus::Valid);
    let node = k2f_core::find_node(&pkg.root, "invoice.details").unwrap();
    assert_eq!(node.break_inside, k2f_core::BreakInside::Avoid);
}

#[test]
fn suggest_rejects_non_text_and_clipboard_omits_geometry() {
    let mut editor = Editor::open(&invoice_bytes()).unwrap();
    let err = editor.suggest("invoice.table", "x").unwrap_err();
    assert_eq!(err.code, "WRONG_CONTENT");
    let clip = editor.clipboard("invoice.total").unwrap();
    assert_eq!(clip.id, "invoice.total");
    assert!(clip.text.unwrap().contains("7,047.00"));
    assert!(clip.node.get("x").is_none());
}

#[test]
fn compensation_amount_hit_edit_relock_drops_signature() {
    let mut ed = common::open("legal");
    common::insert_heading(
        &mut ed,
        "root",
        "contract.title",
        1,
        "Independent Contractor Agreement",
    );
    common::insert_text(
        &mut ed,
        "root",
        "contract.compensation.amount",
        "body",
        "USD 100",
    );
    let unsigned = ed.save_bytes().unwrap();
    let key = generate_key().unwrap();
    let signed = sign(
        &unsigned,
        &key.secret_hex,
        Some("Acme Legal"),
        Some(1_704_067_200),
    )
    .unwrap();
    let opened = OpenedDocument::open(&signed).unwrap();
    assert_eq!(opened.banner(), Banner::Signed);
    let b = &opened.boxes_for("contract.compensation.amount")[0];
    let hit = opened
        .hit_test(b.page, b.x + b.width / 2, b.y + b.height / 2)
        .unwrap();
    assert_eq!(hit.leaf(), Some("contract.compensation.amount"));
    assert_eq!(
        opened
            .selection("contract.compensation.amount")
            .unwrap()
            .text
            .as_deref(),
        Some("USD 100")
    );

    let before = unpack_bytes(&signed).unwrap();
    let before_lock = before.lock_json.clone().unwrap();
    let mut editor = Editor::open(&signed).unwrap();
    editor
        .replace_text("contract.compensation.amount", "USD 110")
        .unwrap();
    let relocked = editor.save_bytes().unwrap();
    let pkg = unpack_bytes(&relocked).unwrap();
    assert_eq!(verify_ok(&pkg), VerifyStatus::Valid);
    assert!(pkg.signatures_json.is_none());
    assert_ne!(pkg.lock_json.as_ref().unwrap(), &before_lock);
    match &k2f_core::find_node(&pkg.root, "contract.compensation.amount")
        .unwrap()
        .content
    {
        k2f_core::NodeContent::Text(t) => assert_eq!(t, "USD 110"),
        other => panic!("{other:?}"),
    }
    assert!(k2f_core::find_node(&pkg.root, "contract.title").is_some());
    let again = OpenedDocument::open(&relocked).unwrap();
    assert_eq!(again.banner(), Banner::Unsigned);
    assert_eq!(
        Editor::open(&relocked)
            .unwrap()
            .node_text("contract.compensation.amount")
            .unwrap(),
        "USD 110"
    );
}
