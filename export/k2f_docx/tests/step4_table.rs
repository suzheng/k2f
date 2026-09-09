mod common;

use k2f_core::{
    for_each_node, node_text, Border, BorderEdge, BorderStyle, GridTrack, NodeContent,
    SemanticNode, TableDataSource, TableSpec,
};
use k2f_docx::{can_emit_native_table, export_opened, table_cell_wml, TextAlign};

fn invoice_sample_cell(doc: &k2f_paint::OpenedDocument) -> (String, String) {
    let mut sample = None;
    for_each_node(doc.semantic_root(), &mut |n| {
        if let NodeContent::Table(spec) = &n.content {
            if let TableDataSource::Inline { rows } = &spec.data {
                if let Some(cell) = rows.get(1).and_then(|r| r.first()) {
                    if let Some(text) = node_text(cell) {
                        sample = Some((cell.id.clone(), text.to_string()));
                    }
                }
            } else {
                panic!("invoice table still Asset; OpenedDocument should expand");
            }
        }
    });
    sample.expect("invoice table has a text cell in row 1")
}

fn invoice_cell_ids(doc: &k2f_paint::OpenedDocument) -> Vec<String> {
    let mut ids = Vec::new();
    for_each_node(doc.semantic_root(), &mut |n| {
        if let NodeContent::Table(spec) = &n.content {
            if let TableDataSource::Inline { rows } = &spec.data {
                ids.extend(rows.iter().flatten().map(|c| c.id.clone()));
            }
        }
    });
    ids
}

fn text_cell(id: &str) -> SemanticNode {
    SemanticNode {
        id: id.into(),
        role: "body".into(),
        content: NodeContent::Text("x".into()),
        ..Default::default()
    }
}

#[test]
fn invoice_table_is_boxes_not_w_tbl() {
    let doc = common::invoice();
    let (cell_id, text) = invoice_sample_cell(&doc);
    assert!(!text.is_empty(), "sample cell text");
    let docx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    assert!(
        !xml.contains("<w:tbl>"),
        "Word Dark Mode inverts w:tbl / w:shd; table cells must be lock boxes + text"
    );
    assert!(
        !xml.contains("<wps:style>"),
        "wps:style fillRef follows the Office theme; lock RGB lives in spPr / w:rPr, got style"
    );
    assert!(
        xml.contains(&text),
        "cell {text:?} ({cell_id}) missing from Word text boxes"
    );
}

#[allow(non_snake_case)]
#[test]
fn gridCol_writer_still_matches_spec_shape() {
    let spec = TableSpec {
        column_widths: vec![GridTrack::Fr { fr: 1 }, GridTrack::Fr { fr: 1 }],
        header_rows: 0,
        gap: 0,
        row_gap: None,
        column_gap: None,
        data: TableDataSource::Inline {
            rows: vec![vec![text_cell("a"), text_cell("b")]],
        },
    };
    assert!(can_emit_native_table(&spec));
    assert_eq!(spec.column_widths.len(), 2);
}

#[test]
fn cell_jc_matches_infer() {
    let center = table_cell_wml(TextAlign::Center, None);
    assert!(
        center.contains(r#"w:jc w:val="center""#),
        "table writer must emit infer Center, got {center}"
    );
    let right = table_cell_wml(TextAlign::Right, None);
    assert!(
        right.contains(r#"w:jc w:val="right""#),
        "table writer must emit infer Right, got {right}"
    );
    assert!(
        !center.contains(r#"w:jc w:val="left""#) || center.contains(r#"w:jc w:val="center""#),
        "must not hardcode left over Center"
    );
}

#[test]
fn cell_borders_follow_lock_edges() {
    let border = Border {
        width_pt: 1_000,
        color: "#1A73E8".into(),
        edges: vec![BorderEdge::Bottom],
        style: BorderStyle::Solid,
    };
    let xml = table_cell_wml(TextAlign::Left, Some(&border));
    assert!(
        xml.contains("D0D0D0") == false,
        "must not fake four-side #D0D0D0, got {xml}"
    );
    let bottom = xml.find("<w:bottom").expect("w:bottom");
    let after = &xml[bottom..];
    let tag_end = after
        .find("/>")
        .or_else(|| after.find('>'))
        .expect("bottom tag");
    let bottom_tag = &after[..tag_end];
    assert!(
        bottom_tag.contains("1A73E8") || bottom_tag.contains("1a73e8"),
        "bottom edge must keep lock color, got {bottom_tag}"
    );
    assert!(
        !bottom_tag.contains(r#"w:val="nil""#),
        "bottom must be drawn"
    );
    for edge in ["top", "left", "right"] {
        assert!(
            xml.contains(&format!(r#"<w:{edge} w:val="nil""#)),
            "missing edge {edge} must be nil, got {xml}"
        );
    }
}

#[test]
fn table_cells_export_as_named_textboxes() {
    let doc = common::invoice();
    let cell_ids = invoice_cell_ids(&doc);
    assert!(cell_ids.iter().any(|id| id == "invoice.row_1.item"));
    let docx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let names: Vec<&str> = parsed
        .descendants()
        .filter(|n| n.has_tag_name("docPr"))
        .filter_map(|pr| common::local_attr(&pr, "name"))
        .collect();
    assert!(
        names.iter().any(|n| *n == "invoice.row_1.item"),
        "table cell must be a named text box, got {names:?}"
    );
}

#[test]
fn empty_cell_emits_w_p() {
    let xml = table_cell_wml(TextAlign::Left, None);
    assert!(
        xml.contains("<w:p>") || xml.contains("<w:p "),
        "empty cell must contain w:p, got {xml}"
    );
    assert!(
        !xml.contains("<w:t>"),
        "empty cell should not require w:t, got {xml}"
    );
}

#[test]
fn nested_cell_returns_none() {
    let image = TableSpec {
        column_widths: vec![GridTrack::Fr { fr: 1 }],
        header_rows: 0,
        gap: 0,
        row_gap: None,
        column_gap: None,
        data: TableDataSource::Inline {
            rows: vec![vec![SemanticNode {
                id: "img".into(),
                role: "body".into(),
                content: NodeContent::Image {
                    src: "assets/x.png".into(),
                    width: k2f_core::Pt(10_000),
                    height: k2f_core::Pt(10_000),
                },
                ..Default::default()
            }]],
        },
    };
    assert!(!can_emit_native_table(&image));
    let nested = TableSpec {
        column_widths: vec![GridTrack::Fr { fr: 1 }],
        header_rows: 0,
        gap: 0,
        row_gap: None,
        column_gap: None,
        data: TableDataSource::Inline {
            rows: vec![vec![SemanticNode {
                id: "wrap".into(),
                role: "body".into(),
                content: NodeContent::Container {
                    children: vec![text_cell("inner")],
                },
                ..Default::default()
            }]],
        },
    };
    assert!(!can_emit_native_table(&nested));
    let plain = TableSpec {
        column_widths: vec![GridTrack::Fr { fr: 1 }],
        header_rows: 0,
        gap: 0,
        row_gap: None,
        column_gap: None,
        data: TableDataSource::Inline {
            rows: vec![vec![text_cell("c")]],
        },
    };
    assert!(can_emit_native_table(&plain));
    let asset = TableSpec {
        column_widths: vec![GridTrack::Fr { fr: 1 }],
        header_rows: 0,
        gap: 0,
        row_gap: None,
        column_gap: None,
        data: TableDataSource::Asset {
            source: "assets/data/t.json".into(),
        },
    };
    assert!(!can_emit_native_table(&asset));
}
