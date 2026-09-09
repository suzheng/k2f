mod common;

use k2f_core::{BreakInside, ColumnSpan, NodeContent, SemanticNode};
use k2f_sdk::export_pdf;

#[test]
fn paper_manuscript_columns_saves_and_exports_pdf() {
    let mut ed = common::open("report");
    common::insert_heading(
        &mut ed,
        "root",
        "paper.title",
        1,
        "A Deterministic Column Flow",
    );
    common::insert_text(
        &mut ed,
        "root",
        "paper.abstract",
        "body",
        "Abstract. This short note exercises multi-column continuous flow with a full-width figure.",
    );
    let cols = common::insert_columns(&mut ed, "paper.body", 2, 12_000);
    common::insert_text(
        &mut ed,
        &cols,
        "paper.p1",
        "body",
        "Introduction. Column packing fills the left column, then the right, then the next page. Agents never assign x/y.",
    );
    common::insert_node(
        &mut ed,
        &cols,
        &SemanticNode {
            id: "paper.fig".to_string(),
            role: "warning".to_string(),
            break_inside: BreakInside::Avoid,
            column_span: ColumnSpan::All,
            content: NodeContent::Text("Figure 1. Full-width callout inside columns.".to_string()),
            ..Default::default()
        },
    );
    common::insert_text(
        &mut ed,
        &cols,
        "paper.p2",
        "body",
        "Conclusion. After the span, body text resumes in columns from the left.",
    );
    ed.validate_package().unwrap();
    let bytes = ed.save_bytes().unwrap();
    assert!(bytes.len() > 100);
    let pdf = export_pdf(&bytes).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
}
