use crate::ir::{PageElement, PictureBox, ShapeBox, TextBox};
use crate::picture::{pic_xml, raster_pic_xml};
use crate::xml::escape_xml;
use std::collections::BTreeMap;

use super::textbox::textbox_wsp_xml;

const SHAPE_URI: &str = "http://schemas.microsoft.com/office/word/2010/wordprocessingShape";
const PIC_URI: &str = "http://schemas.openxmlformats.org/drawingml/2006/picture";

pub(crate) fn page_bg_anchor(fill_hex: &str, cx: i64, cy: i64, doc_pr_id: u32) -> String {
    let inner = format!(
        r#"                <wps:wsp>
                  <wps:cNvSpPr/>
                  <wps:spPr>
                    <a:xfrm>
                      <a:off x="0" y="0"/>
                      <a:ext cx="{cx}" cy="{cy}"/>
                    </a:xfrm>
                    <a:prstGeom prst="rect">
                      <a:avLst/>
                    </a:prstGeom>
                    <a:solidFill>
                      <a:srgbClr val="{fill_hex}"/>
                    </a:solidFill>
                    <a:ln>
                      <a:noFill/>
                    </a:ln>
                  </wps:spPr>
                  <wps:bodyPr/>
                </wps:wsp>
"#
    );
    wp_anchor(
        0,
        0,
        cx,
        cy,
        0,
        true,
        doc_pr_id,
        &format!("PageBackground{doc_pr_id}"),
        SHAPE_URI,
        &inner,
    )
}

pub(crate) fn shape_wsp_xml(shape: &ShapeBox) -> String {
    let geom = if shape.corner_emu <= 0 {
        "                    <a:prstGeom prst=\"rect\">\n                      <a:avLst/>\n                    </a:prstGeom>\n".to_string()
    } else {
        let adj = crate::shape::round_rect_adj(shape.corner_emu, shape.cx_emu, shape.cy_emu);
        format!(
            "                    <a:prstGeom prst=\"roundRect\">\n                      <a:avLst>\n                        <a:gd name=\"adj\" fmla=\"val {adj}\"/>\n                      </a:avLst>\n                    </a:prstGeom>\n"
        )
    };
    let fill = match &shape.fill_hex {
        Some(hex) => format!(
            "                    <a:solidFill>\n                      <a:srgbClr val=\"{hex}\"/>\n                    </a:solidFill>\n"
        ),
        None => "                    <a:noFill/>\n".into(),
    };
    format!(
        r#"                <wps:wsp>
                  <wps:cNvSpPr/>
                  <wps:spPr>
                    <a:xfrm>
                      <a:off x="0" y="0"/>
                      <a:ext cx="{cx}" cy="{cy}"/>
                    </a:xfrm>
{geom}{fill}{ln}                  </wps:spPr>
                  <wps:bodyPr/>
                </wps:wsp>
"#,
        cx = shape.cx_emu,
        cy = shape.cy_emu,
        ln = line_xml(shape),
    )
}

fn line_xml(shape: &ShapeBox) -> String {
    let Some(hex) = &shape.line_hex else {
        return "                    <a:ln>\n                      <a:noFill/>\n                    </a:ln>\n".into();
    };
    let dash = match shape.line_dash {
        crate::ir::LineDash::Solid => "solid",
        crate::ir::LineDash::Dash => "dash",
        crate::ir::LineDash::Dot => "sysDot",
    };
    format!(
        "                    <a:ln w=\"{w}\">\n                      <a:solidFill>\n                        <a:srgbClr val=\"{hex}\"/>\n                      </a:solidFill>\n                      <a:prstDash val=\"{dash}\"/>\n                    </a:ln>\n",
        w = shape.line_w_emu,
    )
}

pub(crate) fn textbox_anchor(
    tb: &TextBox,
    doc_pr_id: u32,
    hyperlink_rids: &BTreeMap<String, String>,
) -> String {
    wp_anchor(
        tb.x_emu,
        tb.y_emu,
        tb.cx_emu,
        tb.cy_emu,
        tb.relative_height,
        false,
        doc_pr_id,
        &tb.node_id,
        SHAPE_URI,
        &textbox_wsp_xml(tb, hyperlink_rids),
    )
}

pub(crate) fn shape_anchor(shape: &ShapeBox, doc_pr_id: u32) -> String {
    wp_anchor(
        shape.x_emu,
        shape.y_emu,
        shape.cx_emu,
        shape.cy_emu,
        shape.relative_height,
        shape.behind_doc,
        doc_pr_id,
        &shape.node_id,
        SHAPE_URI,
        &shape_wsp_xml(shape),
    )
}

pub(crate) fn picture_anchor(pic: &PictureBox, doc_pr_id: u32, embed_rid: &str) -> String {
    wp_anchor(
        pic.x_emu,
        pic.y_emu,
        pic.cx_emu,
        pic.cy_emu,
        pic.relative_height,
        false,
        doc_pr_id,
        &pic.node_id,
        PIC_URI,
        &pic_xml(pic, embed_rid, doc_pr_id),
    )
}

pub(crate) fn raster_anchor(pic: &PictureBox, doc_pr_id: u32, embed_rid: &str) -> String {
    let name = format!("k2f-raster:{}", pic.node_id);
    wp_anchor(
        pic.x_emu,
        pic.y_emu,
        pic.cx_emu,
        pic.cy_emu,
        pic.relative_height,
        false,
        doc_pr_id,
        &name,
        PIC_URI,
        &raster_pic_xml(pic, embed_rid, doc_pr_id),
    )
}

pub(crate) fn drawing_run(anchor: &str) -> String {
    format!(
        r#"      <w:r>
        <w:drawing>
{anchor}        </w:drawing>
      </w:r>
"#
    )
}

pub(crate) fn element_has_bg_shape(el: &PageElement) -> bool {
    matches!(el, PageElement::Shape(s) if s.behind_doc)
}

pub(crate) fn wp_anchor(
    x: i64,
    y: i64,
    cx: i64,
    cy: i64,
    relative_height: u32,
    behind_doc: bool,
    doc_pr_id: u32,
    name: &str,
    graphic_uri: &str,
    inner: &str,
) -> String {
    let behind = if behind_doc { "1" } else { "0" };
    format!(
        r#"          <wp:anchor distT="0" distB="0" distL="0" distR="0" simplePos="0" relativeHeight="{relative_height}" behindDoc="{behind}" locked="0" layoutInCell="1" allowOverlap="1">
            <wp:simplePos x="0" y="0"/>
            <wp:positionH relativeFrom="page">
              <wp:posOffset>{x}</wp:posOffset>
            </wp:positionH>
            <wp:positionV relativeFrom="page">
              <wp:posOffset>{y}</wp:posOffset>
            </wp:positionV>
            <wp:extent cx="{cx}" cy="{cy}"/>
            <wp:effectExtent l="0" t="0" r="0" b="0"/>
            <wp:wrapNone/>
            <wp:docPr id="{doc_pr_id}" name="{name}"/>
            <wp:cNvGraphicFramePr>
              <a:graphicFrameLocks noChangeAspect="1"/>
            </wp:cNvGraphicFramePr>
            <a:graphic>
              <a:graphicData uri="{graphic_uri}">
{inner}              </a:graphicData>
            </a:graphic>
          </wp:anchor>
"#,
        name = escape_xml(name),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{LineDash, ShapeBox};

    fn box_at(corner_emu: i64) -> ShapeBox {
        ShapeBox {
            node_id: "card".into(),
            x_emu: 0,
            y_emu: 0,
            cx_emu: 100_000,
            cy_emu: 50_000,
            fill_hex: Some("1A73E8".into()),
            corner_emu,
            line_hex: None,
            line_w_emu: 0,
            line_dash: LineDash::Solid,
            behind_doc: false,
            relative_height: 10,
        }
    }

    #[test]
    fn zero_corner_uses_rect_geom() {
        let xml = shape_wsp_xml(&box_at(0));
        assert!(xml.contains("prst=\"rect\""), "{xml}");
        assert!(!xml.contains("roundRect"), "{xml}");
    }

    #[test]
    fn corner_uses_round_rect_adj() {
        let xml = shape_wsp_xml(&box_at(5_000));
        assert!(xml.contains("prst=\"roundRect\""), "{xml}");
        assert!(xml.contains("name=\"adj\""), "{xml}");
    }
}
