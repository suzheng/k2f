use crate::ir::{DocIR, PageElement, PageIR};
use crate::xml::word_hex_color;
use std::collections::BTreeMap;

use super::drawing::{
    drawing_run, picture_anchor, raster_anchor, shape_anchor, textbox_anchor,
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
    // w:background is the page-0 Office slot (Dark Mode may hide it). Each
    // page also paints the lock's full-page solid as the only behindDoc fill.
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
        bg = word_hex_color(
            ir.pages
                .first()
                .map(|p| p.bg_hex.as_str())
                .unwrap_or("FFFFFF")
        ),
    )
}

fn page_paragraph(
    page: &PageIR,
    doc_pr_id: &mut u32,
    hyperlink_rids: &BTreeMap<String, String>,
    picture_rids: &BTreeMap<String, String>,
) -> String {
    let mut runs = String::new();
    // Page-0 w:background is a hint. The lock's per-page full-page solid is
    // already in `page.elements` as the behindDoc wash.
    //
    // LibreOffice Writer stacks floating anchors by document order (earlier =
    // on top) and largely ignores relativeHeight among wps shapes. Emit
    // highest relativeHeight first so LO matches Word's relativeHeight stack
    // (card fills under images, captions, strokes).
    for el in z_order_for_writer(&page.elements) {
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

/// Stable high→low relativeHeight order for floating anchors.
fn z_order_for_writer(elements: &[PageElement]) -> Vec<&PageElement> {
    let mut idxs: Vec<usize> = (0..elements.len()).collect();
    idxs.sort_by(|&a, &b| {
        elements[b]
            .relative_height()
            .cmp(&elements[a].relative_height())
            .then(a.cmp(&b))
    });
    idxs.into_iter().map(|i| &elements[i]).collect()
}

pub fn hdrftr_xml(
    tag: &str,
    elements: &[PageElement],
    hyperlink_rids: &BTreeMap<String, String>,
    picture_rids: &BTreeMap<String, String>,
) -> String {
    let mut runs = String::new();
    let mut doc_pr_id = 1u32;
    for el in z_order_for_writer(elements) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{LineDash, ShapeBox};

    fn shape(id: &str, rel: u32) -> PageElement {
        PageElement::Shape(ShapeBox {
            node_id: id.into(),
            x_emu: 0,
            y_emu: 0,
            cx_emu: 1000,
            cy_emu: 100,
            fill_hex: Some("111111".into()),
            fill_alpha: 255,
            gradient: None,
            corner_emu: 0,
            line_hex: None,
            line_alpha: 255,
            line_w_emu: 0,
            line_dash: LineDash::Solid,
            behind_doc: false,
            relative_height: rel,
            pin_empty_txbox: true,
        })
    }

    #[test]
    fn writer_emit_order_is_high_relative_height_first() {
        let elements = vec![shape("fill", 10), shape("edge", 41), shape("mid", 20)];
        let ordered: Vec<&str> = z_order_for_writer(&elements)
            .into_iter()
            .map(|el| match el {
                PageElement::Shape(s) => s.node_id.as_str(),
                _ => "",
            })
            .collect();
        assert_eq!(ordered, vec!["edge", "mid", "fill"]);
    }
}
