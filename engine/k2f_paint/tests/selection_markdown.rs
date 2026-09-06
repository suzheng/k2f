use k2f_core::{for_each_node, NodeContent};
use k2f_markdown::NodeCharRange;
use k2f_paint::OpenedDocument;

fn invoice_bytes() -> Vec<u8> {
    std::fs::read(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/published/invoice.K2F"),
    )
    .expect("examples/published/invoice.K2F")
}

fn contract_bytes() -> Vec<u8> {
    std::fs::read(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/published/contract.K2F"),
    )
    .expect("examples/published/contract.K2F")
}

#[test]
fn invoice_heading_selection_is_markdown_h1() {
    let doc = OpenedDocument::open(&invoice_bytes()).unwrap();
    let mut heading = None;
    for_each_node(doc.semantic_root(), &mut |n| {
        if n.role == "h1" {
            if let NodeContent::Text(t) = &n.content {
                heading = Some((n.id.clone(), t.chars().count()));
            }
        }
    });
    let (id, len) = heading.expect("invoice h1");
    let md = doc.selection_to_markdown(&[NodeCharRange {
        node_id: id,
        char_start: 0,
        char_end: len,
    }]);
    assert!(md.trim_start().starts_with("# "), "got {md}");
    assert!(!md.contains("<!--"), "clipboard must omit hints: {md}");
}

#[test]
fn invoice_table_cells_copy_as_gfm_subtable() {
    let doc = OpenedDocument::open(&invoice_bytes()).unwrap();
    let mut cells: Vec<(String, usize)> = Vec::new();
    for_each_node(doc.semantic_root(), &mut |n| {
        if n.role == "table_row_cell" || n.role == "table_header_cell" {
            if let NodeContent::Text(t) = &n.content {
                cells.push((n.id.clone(), t.chars().count()));
            }
        }
    });
    assert!(
        cells.len() >= 4,
        "expanded invoice table should have cells, got {}",
        cells.len()
    );
    // Pick two body cells from different columns if possible.
    let body: Vec<_> = cells
        .iter()
        .filter(|(id, _)| id.contains(".r"))
        .take(2)
        .collect();
    assert!(body.len() >= 2, "need body cells, got {cells:?}");
    let ranges: Vec<_> = body
        .iter()
        .map(|(id, len)| NodeCharRange {
            node_id: id.clone(),
            char_start: 0,
            char_end: *len,
        })
        .collect();
    let md = doc.selection_to_markdown(&ranges);
    assert!(md.contains('|'), "expected GFM table, got {md}");
    assert!(md.contains("---"), "expected GFM separator, got {md}");
}

#[test]
fn contract_text_selection_emits_markdown() {
    let doc = OpenedDocument::open(&contract_bytes()).unwrap();
    let mut pick = None;
    for_each_node(doc.semantic_root(), &mut |n| {
        if pick.is_some() {
            return;
        }
        if let NodeContent::Text(t) = &n.content {
            if !t.is_empty() {
                pick = Some((n.id.clone(), t.chars().count(), n.role.clone()));
            }
        }
    });
    let (id, len, role) = pick.expect("contract text node");
    let md = doc.selection_to_markdown(&[NodeCharRange {
        node_id: id,
        char_start: 0,
        char_end: len,
    }]);
    assert!(!md.trim().is_empty(), "role={role} md empty");
    assert!(!md.contains("<!--"), "clipboard omits hints: {md}");
}
