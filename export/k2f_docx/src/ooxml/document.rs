use crate::ir::{DocIR, PageElement, PageIR};
use std::collections::BTreeMap;

use super::drawing::{
    drawing_run, element_has_bg_shape, page_bg_anchor, picture_anchor, raster_anchor, shape_anchor,
    textbox_anchor,
};

const NS: &str = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
            xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
            xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
            xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"
            xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"
            xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"
            xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"
            mc:Ignorable="w14""#;

pub fn document_xml(
    ir: &DocIR,
    hyperlink_rids: &BTreeMap<String, String>,
    picture_rids: &BTreeMap<String, String>,
    has_header: bool,
    has_footer: bool,
) -> String {
    let mut body = String::new();
    let mut doc_pr_id = 1u32;
    let n = ir.pages.len();
    for (i, page) in ir.pages.iter().enumerate() {
        body.push_str(&page_paragraph(
            page,
            ir.page_width_emu,
            ir.page_height_emu,
            &mut doc_pr_id,
            hyperlink_rids,
            picture_rids,
        ));
        if i + 1 < n {
            body.push_str(
                r#"    <w:p>
      <w:r>
        <w:br w:type="page"/>
      </w:r>
    </w:p>
"#,
            );
        }
    }
    let mut sect = String::new();
    if has_header {
        sect.push_str(r#"      <w:headerReference w:type="default" r:id="rIdH1"/>"#);
        sect.push('\n');
    }
    if has_footer {
        sect.push_str(r#"      <w:footerReference w:type="default" r:id="rIdF1"/>"#);
        sect.push('\n');
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document {NS}>
  <w:background w:color="{bg}"/>
  <w:body>
{body}    <w:sectPr>
{sect}      <w:pgSz w:w="{w}" w:h="{h}"{orient}/>
      <w:pgMar w:top="0" w:right="0" w:bottom="0" w:left="0"
               w:header="0" w:footer="0" w:gutter="0"/>
    </w:sectPr>
  </w:body>
</w:document>
"#,
        w = ir.page_width_twips,
        orient = if ir.page_width_twips > ir.page_height_twips {
            r#" w:orient="landscape""#
        } else {
            ""
        },
        h = ir.page_height_twips,
        bg = paper_hex(
            ir.pages
                .first()
                .map(|p| p.bg_hex.as_str())
                .unwrap_or("FFFFFF")
        ),
    )
}

fn paper_hex(hex: &str) -> String {
    match hex.to_ascii_uppercase().as_str() {
        "FFFFFF" => "FFFFFE".into(),
        "000000" => "000001".into(),
        other => other.to_string(),
    }
}

fn page_paragraph(
    page: &PageIR,
    cx: i64,
    cy: i64,
    doc_pr_id: &mut u32,
    hyperlink_rids: &BTreeMap<String, String>,
    picture_rids: &BTreeMap<String, String>,
) -> String {
    let mut runs = String::new();
    if !page.elements.iter().any(element_has_bg_shape) {
        runs.push_str(&drawing_run(&page_bg_anchor(
            &page.bg_hex,
            cx,
            cy,
            *doc_pr_id,
        )));
        *doc_pr_id += 1;
    }
    for el in &page.elements {
        runs.push_str(&drawing_run(&element_anchor(
            el,
            *doc_pr_id,
            hyperlink_rids,
            picture_rids,
        )));
        *doc_pr_id += 1;
    }
    format!(
        r#"    <w:p>
{runs}    </w:p>
"#
    )
}

pub fn hdrftr_xml(
    tag: &str,
    elements: &[PageElement],
    hyperlink_rids: &BTreeMap<String, String>,
    picture_rids: &BTreeMap<String, String>,
) -> String {
    let mut runs = String::new();
    let mut doc_pr_id = 1u32;
    for el in elements {
        runs.push_str(&drawing_run(&element_anchor(
            el,
            doc_pr_id,
            hyperlink_rids,
            picture_rids,
        )));
        doc_pr_id += 1;
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:{tag} {NS}>
  <w:p>
{runs}  </w:p>
</w:{tag}>
"#
    )
}

fn element_anchor(
    el: &PageElement,
    doc_pr_id: u32,
    hyperlink_rids: &BTreeMap<String, String>,
    picture_rids: &BTreeMap<String, String>,
) -> String {
    match el {
        PageElement::TextBox(tb) => textbox_anchor(tb, doc_pr_id, hyperlink_rids),
        PageElement::Shape(s) => shape_anchor(s, doc_pr_id),
        PageElement::Picture(p) => {
            let rid = picture_rids
                .get(&p.media_name)
                .map(String::as_str)
                .unwrap_or("");
            picture_anchor(p, doc_pr_id, rid)
        }
        PageElement::Raster(p) => {
            let rid = picture_rids
                .get(&p.media_name)
                .map(String::as_str)
                .unwrap_or("");
            raster_anchor(p, doc_pr_id, rid)
        }
        PageElement::Table(t) => crate::table::table_anchor(t, doc_pr_id, hyperlink_rids),
    }
}
