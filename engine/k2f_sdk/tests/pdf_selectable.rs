mod common;

#[path = "../../k2f_pdf/tests/common/extract.rs"]
mod extract;

use k2f_sdk::export_pdf;

#[test]
fn legal_chinese_pdf_text_is_extractable() {
    let mut ed = common::open("legal");
    common::insert_heading(&mut ed, "root", "contract.title", 1, "本合同");
    common::insert_text(
        &mut ed,
        "root",
        "contract.clause",
        "body",
        "承包方应按附件所述专业标准提供服务。",
    );
    let bytes = ed.save_bytes().unwrap();
    let pdf = export_pdf(&bytes).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    assert!(
        extract::page_content_has(&pdf, 0, "3 Tr"),
        "Noto CFF OpenType must still use an invisible text layer"
    );
    let extracted = extract::extract_pdf_text(&pdf);
    assert!(
        extracted.contains("合同") && extracted.contains("承包方"),
        "extracted {extracted:?}"
    );
}
