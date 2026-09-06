mod common;

use k2f_core::{
    for_each_node, node_text, Border, BorderEdge, BorderStyle, GridTrack, NodeContent,
    SemanticNode, TableDataSource, TableSpec,
};
use k2f_docx::{
    can_emit_native_table, export_opened, infer_text_align, millipt_to_twips, table_cell_wml,
    TextAlign,
};

fn invoice_table_spec(doc: &k2f_paint::OpenedDocument) -> (String, usize) {
    let mut found = None;
    for_each_node(doc.semantic_root(), &mut |n| {
        if let NodeContent::Table(spec) = &n.content {
            found = Some((n.id.clone(), spec.column_widths.len()));
        }
    });
    found.expect("invoice semantic tree has a Table node after asset expand")
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

fn text_cell(id: &str) -> SemanticNode {
    SemanticNode {
        id: id.into(),
        role: "body".into(),
        content: NodeContent::Text("x".into()),
        ..Default::default()
    }
}

fn tc_of_text<'a, 'input>(
    doc: &'a roxmltree::Document<'input>,
    text: &str,
) -> Option<roxmltree::Node<'a, 'input>> {
    doc.descendants().find(|n| {
        n.has_tag_name("tc")
            && n.descendants()
                .any(|t| t.has_tag_name("t") && t.text() == Some(text))
    })
}

#[test]
fn has_w_tbl() {
    let docx = export_opened(&common::invoice()).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    assert!(
        xml.contains("<w:tbl>"),
        "invoice line items must export as w:tbl"
    );
    assert!(
        xml.contains("<w:tblGrid>"),
        "native table must include w:tblGrid"
    );
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let tbl = parsed
        .descendants()
        .find(|n| n.has_tag_name("tbl"))
        .expect("w:tbl");
    assert!(
        tbl.ancestors().any(|a| a.has_tag_name("txbxContent")),
        "w:tbl must sit in w:txbxContent of a positioned text box"
    );
}

#[allow(non_snake_case)]
#[test]
fn gridCol_count_matches_spec() {
    let doc = common::invoice();
    let (_, want) = invoice_table_spec(&doc);
    let docx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let tbl = parsed
        .descendants()
        .find(|n| n.has_tag_name("tbl"))
        .expect("w:tbl");
    let cols = tbl
        .descendants()
        .filter(|n| n.has_tag_name("gridCol"))
        .count();
    assert_eq!(cols, want, "w:gridCol count vs spec.column_widths");
}

#[test]
fn cell_text_in_tc() {
    let doc = common::invoice();
    let (cell_id, text) = invoice_sample_cell(&doc);
    assert!(!text.is_empty(), "sample cell text");
    let docx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let tc = tc_of_text(&parsed, &text).unwrap_or_else(|| panic!("no w:tc contains {text:?}"));
    let blob: String = tc
        .descendants()
        .filter(|n| n.has_tag_name("t"))
        .filter_map(|n| n.text())
        .collect();
    assert!(
        blob.contains(&text),
        "cell {text:?} ({cell_id}) missing from w:tc"
    );
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

    let doc = common::invoice();
    let lock = doc.lock().expect("locked");
    let page = &lock.geometry.pages[1];
    let cell_id = "invoice.th.item";
    let geo = find_geo(&page.root, cell_id).expect("header cell geo");
    let node = k2f_core::find_in_trees(doc.semantic_root(), doc.running_blocks(), cell_id)
        .expect("header cell node");
    let text = node_text(node).unwrap_or("");
    let want = infer_text_align(&geo, text);
    let docx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let tc = tc_of_text(&parsed, "Item").expect("header Item cell");
    let jc = tc
        .descendants()
        .find(|n| n.has_tag_name("jc"))
        .and_then(|n| common::local_attr(&n, "val"))
        .expect("w:jc on cell paragraph");
    assert_eq!(jc, want.jc_val());
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
fn table_cells_not_duplicated_as_page_textboxes() {
    let doc = common::invoice();
    let cell_ids = invoice_cell_ids(&doc);
    assert!(cell_ids.iter().any(|id| id == "invoice.row_1.item"));
    let docx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    for pr in parsed.descendants().filter(|n| n.has_tag_name("docPr")) {
        let Some(name) = common::local_attr(&pr, "name") else {
            continue;
        };
        let Some(anchor) = pr.ancestors().find(|n| n.has_tag_name("anchor")) else {
            continue;
        };
        let is_txbx = anchor.descendants().any(|n| n.has_tag_name("txbxContent"))
            && !anchor.descendants().any(|n| n.has_tag_name("tbl"));
        if !is_txbx {
            continue;
        }
        assert!(
            !cell_ids.iter().any(|c| c == name),
            "cell {name} still exported as a page text box"
        );
    }
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
        data: TableDataSource::Inline {
            rows: vec![vec![text_cell("c")]],
        },
    };
    assert!(can_emit_native_table(&plain));
    let asset = TableSpec {
        column_widths: vec![GridTrack::Fr { fr: 1 }],
        header_rows: 0,
        gap: 0,
        data: TableDataSource::Asset {
            source: "assets/data/t.json".into(),
        },
    };
    assert!(!can_emit_native_table(&asset));
}

#[test]
fn row_heights_follow_lock_y_delta() {
    let doc = common::invoice();
    let (table_id, ncols) = invoice_table_spec(&doc);
    let lock = doc.lock().expect("locked");
    let mut table_geo = None;
    for page in &lock.geometry.pages {
        if let Some(g) = find_geo(&page.root, &table_id) {
            table_geo = Some(g);
            break;
        }
    }
    let geo = table_geo.expect("invoice table geo");
    let complete = geo.children.len() / ncols;
    assert!(complete >= 2, "invoice table needs at least two rows");
    let mut want = Vec::with_capacity(complete);
    for r in 0..complete {
        let row = &geo.children[r * ncols..(r + 1) * ncols];
        let y = row.first().map(|c| c.y.0).unwrap_or(geo.y.0);
        let next_y = if r + 1 < complete {
            geo.children[(r + 1) * ncols].y.0
        } else {
            geo.y.0 + geo.height.0
        };
        let from_gap = next_y - y;
        let from_cell = row.iter().map(|c| c.height.0).max().unwrap_or(0);
        want.push(millipt_to_twips(
            i64::try_from(from_gap.max(from_cell).max(0)).unwrap_or(0),
        ));
    }
    let cell_only: Vec<i64> = (0..complete)
        .map(|r| {
            let row = &geo.children[r * ncols..(r + 1) * ncols];
            millipt_to_twips(
                i64::try_from(row.iter().map(|c| c.height.0).max().unwrap_or(0).max(0))
                    .unwrap_or(0),
            )
        })
        .collect();
    assert!(
        want.iter().zip(&cell_only).any(|(w, c)| w > c),
        "invoice table gap must enlarge at least one row vs cell box: want={want:?} cell={cell_only:?}"
    );

    let docx = export_opened(&doc).unwrap();
    let xml = common::xml_in(&docx, "word/document.xml");
    let parsed = roxmltree::Document::parse(&xml).unwrap();
    let tbl = parsed
        .descendants()
        .find(|n| n.has_tag_name("tbl"))
        .expect("w:tbl");
    let got: Vec<i64> = tbl
        .descendants()
        .filter(|n| n.has_tag_name("trHeight"))
        .map(|n| {
            common::local_attr(&n, "val")
                .expect("w:val")
                .parse::<i64>()
                .expect("twips")
        })
        .collect();
    assert_eq!(got, want, "w:trHeight must include lock table gap");
}

fn find_geo(node: &k2f_core::GeometryNode, id: &str) -> Option<k2f_core::GeometryNode> {
    if node.id == id {
        return Some(node.clone());
    }
    node.children.iter().find_map(|c| find_geo(c, id))
}
