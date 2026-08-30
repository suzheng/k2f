mod common;

use common::repo_root;
use k2f_core::canonical_json_string;
use k2f_package::{inspect_package, unpack_bytes, IntegrityStatus};
use k2f_paint::{OpenedDocument, PaintError, OFFICIAL_PNG_SCALE};
use serde_json::{json, Value};
use std::fs;

#[test]
fn unknown_paint_op_is_broken_and_is_not_skipped() {
    let bytes = fs::read(repo_root().join("examples/published/contract.K2F")).unwrap();
    let mut pkg = unpack_bytes(&bytes).unwrap();
    let mut lock: Value = serde_json::from_str(pkg.lock_json.as_ref().unwrap()).unwrap();
    lock["render_plan"]["pages"][0]["ops"]
        .as_array_mut()
        .unwrap()
        .push(json!({"type": "draw_unicorn", "node_id": "x"}));
    pkg.lock_json = Some(canonical_json_string(&lock).unwrap());

    let report = inspect_package(&pkg).unwrap();
    assert_eq!(report.status, IntegrityStatus::UnknownPaintOp);
    assert_eq!(report.status.code(), "UNKNOWN_PAINT_OP");

    let doc = OpenedDocument::from_package(pkg).unwrap();
    assert_eq!(doc.status_code(), "UNKNOWN_PAINT_OP");
    assert!(matches!(
        doc.render_page(0, OFFICIAL_PNG_SCALE),
        Err(PaintError::UnknownOp)
    ));
}
