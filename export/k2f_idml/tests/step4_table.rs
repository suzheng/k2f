mod common;

use k2f_core::{
    for_each_node, node_text, Border, BorderEdge, BorderStyle, GridTrack, NodeContent, Pt,
    SemanticNode, TableDataSource, TableSpec,
};
use k2f_idml::{cell_borders, cell_edge_attrs, export_opened, harvestable_inline_rows};

fn invoice_table_col_count(doc: &k2f_paint::OpenedDocument) -> usize {
    let mut found = None;
    for_each_node(doc.semantic_root(), &mut |n| {
        if let NodeContent::Table(spec) = &n.content {
            found = Some(spec.column_widths.len());
        }
    });
    found.expect("invoice semantic tree has a Table node")
}

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

fn story_files(idml: &[u8]) -> Vec<String> {
    common::unzip_names(idml)
        .into_iter()
        .filter(|n| n.starts_with("Stories/"))
        .map(|n| common::xml_in(idml, &n))
        .collect()
}

fn stories_blob(idml: &[u8]) -> String {
    story_files(idml).join("\n")
}

fn body_textframe_names(idml: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    for name in common::unzip_names(idml) {
        if !name.starts_with("Spreads/") {
            continue;
        }
        let xml = common::xml_in(idml, &name);
        let doc = roxmltree::Document::parse(&xml).unwrap();
        for n in doc.descendants().filter(|n| n.has_tag_name("TextFrame")) {
            if let Some(nm) = n.attribute("Name") {
                names.push(nm.to_string());
            }
        }
    }
    names
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
fn invoice_has_table() {
    let idml = export_opened(&common::invoice()).unwrap();
    let xml = stories_blob(&idml);
    assert!(
        xml.contains("<Table"),
        "invoice line items must be a Story Table"
    );
    assert!(
        xml.contains("ColumnCount="),
        "Table must set ColumnCount, got no attribute"
    );
}

#[test]
fn tbl_column_count_matches_spec() {
    let doc = common::invoice();
    let want = invoice_table_col_count(&doc);
    let idml = export_opened(&doc).unwrap();
    let mut got: Option<usize> = None;
    for xml in story_files(&idml) {
        let parsed = roxmltree::Document::parse(&xml).unwrap();
        if let Some(table) = parsed.descendants().find(|n| n.has_tag_name("Table")) {
            got = Some(
                table
                    .attribute("ColumnCount")
                    .expect("ColumnCount")
                    .parse()
                    .unwrap(),
            );
            break;
        }
    }
    assert_eq!(
        got.expect("Table"),
        want,
        "ColumnCount vs spec.column_widths"
    );
}

#[test]
fn cell_text_matches_semantic() {
    let doc = common::invoice();
    let (cell_id, text) = invoice_sample_cell(&doc);
    assert!(!text.is_empty(), "sample cell text");
    let idml = export_opened(&doc).unwrap();
    let mut found = false;
    for xml in story_files(&idml) {
        let parsed = roxmltree::Document::parse(&xml).unwrap();
        found |= parsed.descendants().any(|cell| {
            if !cell.has_tag_name("Cell") {
                return false;
            }
            let blob: String = cell
                .descendants()
                .filter(|n| n.has_tag_name("Content"))
                .filter_map(|n| n.text())
                .collect();
            blob.contains(&text)
        });
    }
    assert!(
        found,
        "cell {text:?} ({cell_id}) missing from Table Cell Content"
    );
}

#[test]
fn table_cells_not_duplicated_as_textframes() {
    let doc = common::invoice();
    let cell_ids = invoice_cell_ids(&doc);
    assert!(cell_ids.iter().any(|id| id == "invoice.row_1.item"));
    let idml = export_opened(&doc).unwrap();
    let names = body_textframe_names(&idml);
    for id in &cell_ids {
        assert!(
            !names.iter().any(|n| n == id),
            "cell {id} still exported as body TextFrame/@Name"
        );
    }
}

#[test]
fn nested_or_image_cell_does_not_emit_table() {
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
                    width: Pt(10_000),
                    height: Pt(10_000),
                },
                ..Default::default()
            }]],
        },
    };
    assert!(harvestable_inline_rows(&image).is_none());
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
    assert!(harvestable_inline_rows(&nested).is_none());
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
    assert!(harvestable_inline_rows(&plain).is_some());
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
    assert!(harvestable_inline_rows(&asset).is_none());
}

#[test]
fn cell_border_follows_edges_not_d0d0d0() {
    let border = Border {
        width_pt: 250,
        color: "#1A73E8".into(),
        edges: vec![BorderEdge::Bottom],
        style: BorderStyle::Solid,
    };
    let borders = cell_borders(Some(&border)).unwrap();
    let xml = cell_edge_attrs(&borders);
    assert!(
        !xml.contains("D0D0D0") && !xml.contains("d0d0d0"),
        "must not fake four-side #D0D0D0, got {xml}"
    );
    assert!(borders.bottom.is_some(), "bottom edge must be drawn");
    assert!(
        borders.top.is_none() && borders.left.is_none() && borders.right.is_none(),
        "only Bottom should have a stroke"
    );
    let bottom_w = attr_val(&xml, "BottomEdgeStrokeWeight").expect("bottom weight");
    let top_w = attr_val(&xml, "TopEdgeStrokeWeight").expect("top weight");
    let left_w = attr_val(&xml, "LeftEdgeStrokeWeight").expect("left weight");
    let right_w = attr_val(&xml, "RightEdgeStrokeWeight").expect("right weight");
    assert_ne!(bottom_w, "0", "drawn bottom must have weight, got {xml}");
    assert_eq!(top_w, "0", "missing top must be weight 0");
    assert_eq!(left_w, "0");
    assert_eq!(right_w, "0");
    assert!(
        !(bottom_w == top_w && bottom_w == left_w && bottom_w == right_w),
        "must not give all four edges the same StrokeWeight, got {xml}"
    );
}

fn attr_val<'a>(xml: &'a str, name: &str) -> Option<&'a str> {
    let key = format!("{name}=\"");
    let i = xml.find(&key)?;
    let rest = &xml[i + key.len()..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}
