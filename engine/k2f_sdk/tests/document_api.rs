mod common;

use k2f_package::{unpack_bytes, verify_package, VerifyStatus};
use k2f_sdk::validate_agent_json;
use serde_json::Value;

#[test]
fn sections_replace_text_and_chinese_contract_save() {
    let mut ed = common::open("legal");
    common::insert_heading(&mut ed, "root", "contract.title", 1, "独立顾问协议");
    common::insert_warning(
        &mut ed,
        "root",
        "contract.warning",
        "机密。本文件含专有信息，仅供签署人阅读，不得对外披露。",
    );
    let section = common::insert_section(&mut ed, "contract.services");
    common::insert_heading(&mut ed, &section, "contract.section_1", 2, "1. 服务内容");
    common::insert_text(
        &mut ed,
        &section,
        "contract.clause_1",
        "body",
        "承包方应按附件所述专业标准提供服务，并以本文件条款为准。服务应在约定日期内完成，质量不得低于行业通常要求。双方确认：纸面以本协议锁定版为准，口头承诺不构成变更。",
    );
    common::insert_list(
        &mut ed,
        "root",
        "contract.duties",
        &["仅供签署人阅读".into(), "不得对外披露".into()],
    );
    common::insert_text(&mut ed, "root", "contract.sign.label", "body", "签署栏");
    ed.set_role("contract.sign.label", "signature_block", None)
        .unwrap();
    ed.set_running_header("机密").unwrap();
    ed.set_running_footer("Page {{page_current}} of {{page_total}}")
        .unwrap();

    let json: Value =
        serde_json::from_str(&ed.get_node_json("contract.clause_1").unwrap()).unwrap();
    assert!(json["content"]["value"]
        .as_str()
        .unwrap()
        .contains("承包方"));
    ed.replace_text(
        "contract.clause_1",
        "客户应于收到发票后三十日内支付约定金额。",
    )
    .unwrap();
    let updated: Value =
        serde_json::from_str(&ed.get_node_json("contract.clause_1").unwrap()).unwrap();
    assert_eq!(
        updated["content"]["value"].as_str().unwrap(),
        "客户应于收到发票后三十日内支付约定金额。"
    );

    let bytes = ed.save_bytes().unwrap();
    let pkg = unpack_bytes(&bytes).unwrap();
    assert_eq!(verify_package(&pkg).unwrap(), VerifyStatus::Valid);
    assert!(pkg
        .fonts
        .contains_key("assets/fonts/NotoSansSC-Regular.otf"));
}

#[test]
fn warning_is_text_and_replace_text_uses_same_id() {
    let mut ed = common::open("legal");
    common::insert_warning(&mut ed, "root", "risk.overdue", "Pay now.");
    ed.replace_text("risk.overdue", "Pay later.").unwrap();
    let json: Value = serde_json::from_str(&ed.get_node_json("risk.overdue").unwrap()).unwrap();
    assert_eq!(json["content"]["value"], "Pay later.");
    assert_eq!(json["role"], "warning");
    assert!(json.get("layout").is_none());
    validate_agent_json(&json).unwrap();
}

#[test]
fn clinical_summary_saves_and_verifies() {
    let mut ed = common::open("clinical_summary");
    common::insert_heading(&mut ed, "root", "note.title", 1, "Clinical summary");
    common::insert_warning(&mut ed, "root", "note.allergy", "Allergy: penicillin.");
    common::insert_text(&mut ed, "root", "note.body", "body", "Patient is stable.");
    let bytes = ed.save_bytes().unwrap();
    let pkg = unpack_bytes(&bytes).unwrap();
    assert_eq!(verify_package(&pkg).unwrap(), VerifyStatus::Valid);
}

#[test]
fn add_math_saves_display_formula() {
    let mut ed = common::open("report");
    common::insert_heading(&mut ed, "root", "doc.h1", 1, "Energy");
    common::insert_math(&mut ed, "root", "eq.energy", "E=mc^2");
    let bytes = ed.save_bytes().unwrap();
    let pkg = unpack_bytes(&bytes).unwrap();
    assert_eq!(verify_package(&pkg).unwrap(), VerifyStatus::Valid);
    let mut found = false;
    k2f_core::for_each_node(&pkg.root, &mut |n| {
        if n.id == "eq.energy" {
            found = true;
            assert_eq!(n.role, "math");
            assert_eq!(n.content, k2f_core::NodeContent::Math("E=mc^2".into()));
        }
    });
    assert!(found);
    let md = k2f_sdk::k2f_to_markdown(&bytes).unwrap();
    assert!(md.contains("$$"), "{md}");
    assert!(md.contains("E=mc^2"), "{md}");
}

#[test]
fn invoice_allows_add_math() {
    let mut ed = common::open("invoice");
    common::insert_math(&mut ed, "root", "eq.x", "x");
    let bytes = ed.save_bytes().unwrap();
    assert_eq!(
        verify_package(&unpack_bytes(&bytes).unwrap()).unwrap(),
        VerifyStatus::Valid
    );
}

#[test]
fn system_prompt_is_short_and_forbids_geometry() {
    assert!(k2f_sdk::SYSTEM_PROMPT.contains("Do not look up named official templates"));
    assert!(k2f_sdk::SYSTEM_PROMPT.contains("Do not write paint, lock, x/y"));
    assert!(k2f_sdk::SYSTEM_PROMPT.contains("invoice.total"));
    assert!(k2f_sdk::SYSTEM_PROMPT.contains("open_dir"));
    assert!(k2f_sdk::SYSTEM_PROMPT.len() < 4000);
}
