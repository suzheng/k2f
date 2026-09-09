use crate::ir::{PageElement, PictureBox, ShapeBox, TextBox};
use crate::picture::{pic_xml, raster_wsp_xml};
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
    let fill = shape_fill_xml(shape);
    // Empty in-front *opaque* shells need a pinned txBox so Word does not
    // size-to-fit. Full-page gradients and glass (alpha) must not be txBoxes:
    // LibreOffice Writer paints a page-sized empty text frame over later labels.
    let as_tx = !shape.behind_doc
        && shape.fill_hex.is_some()
        && shape.gradient.is_none()
        && shape.fill_alpha >= 255;
    let cnv = if as_tx {
        "                  <wps:cNvSpPr txBox=\"1\"/>\n"
    } else {
        "                  <wps:cNvSpPr/>\n"
    };
    let tail = if as_tx {
        // Word treats wrap=none empty txBoxes as size-to-text, so a page-width
        // shell (cover body, accent band) collapses to a leftover strip on the
        // left. Pin lock extent the same way leftover-width text frames do.
        "                  <wps:txbx>\n                    <w:txbxContent>\n                      <w:p/>\n                    </w:txbxContent>\n                  </wps:txbx>\n                  <wps:bodyPr wrap=\"square\" lIns=\"0\" tIns=\"0\" rIns=\"0\" bIns=\"0\" anchor=\"t\">\n                    <a:noAutofit/>\n                  </wps:bodyPr>\n"
    } else {
        "                  <wps:bodyPr/>\n"
    };
    format!(
        r#"                <wps:wsp>
{cnv}                  <wps:spPr>
                    <a:xfrm>
                      <a:off x="0" y="0"/>
                      <a:ext cx="{cx}" cy="{cy}"/>
                    </a:xfrm>
{geom}{fill}{ln}                  </wps:spPr>
{tail}                </wps:wsp>
"#,
        cx = shape.cx_emu,
        cy = shape.cy_emu,
        ln = line_xml(shape),
    )
}

fn shape_fill_xml(shape: &ShapeBox) -> String {
    if let Some(g) = &shape.gradient {
        return gradient_fill_xml(g);
    }
    match &shape.fill_hex {
        Some(hex) => solid_fill_xml(hex, shape.fill_alpha),
        None => "                    <a:noFill/>\n".into(),
    }
}

pub(crate) fn solid_fill_xml(hex: &str, alpha: u8) -> String {
    let hex = crate::xml::word_hex_color(hex);
    if alpha >= 255 {
        format!(
            "                    <a:solidFill>\n                      <a:srgbClr val=\"{hex}\"/>\n                    </a:solidFill>\n"
        )
    } else {
        format!(
            "                    <a:solidFill>\n                      <a:srgbClr val=\"{hex}\">\n                        <a:alpha val=\"{a}\"/>\n                      </a:srgbClr>\n                    </a:solidFill>\n",
            a = alpha_permille(alpha),
        )
    }
}

fn gradient_fill_xml(g: &crate::ir::GradientFill) -> String {
    let mut gs = String::new();
    for stop in &g.stops {
        let pos = (stop.pos.saturating_mul(100)).clamp(0, 100_000);
        gs.push_str(&format!(
            "                      <a:gs pos=\"{pos}\">\n{clr}                      </a:gs>\n",
            clr = srgb_clr_xml(&stop.hex, stop.alpha, "                        "),
        ));
    }
    // K2F 0° is +x (right), 90° is +y (down). OOXML `a:lin ang` is 1/60000 deg
    // with 0 = left-to-right, increasing clockwise — same as screen-y-down.
    let ang = g.angle_degrees.saturating_mul(60_000);
    format!(
        "                    <a:gradFill>\n                      <a:gsLst>\n{gs}                      </a:gsLst>\n                      <a:lin ang=\"{ang}\" scaled=\"0\"/>\n                    </a:gradFill>\n"
    )
}

fn srgb_clr_xml(hex: &str, alpha: u8, indent: &str) -> String {
    let hex = crate::xml::word_hex_color(hex);
    if alpha >= 255 {
        format!("{indent}<a:srgbClr val=\"{hex}\"/>\n")
    } else {
        format!(
            "{indent}<a:srgbClr val=\"{hex}\">\n{indent}  <a:alpha val=\"{a}\"/>\n{indent}</a:srgbClr>\n",
            a = alpha_permille(alpha),
        )
    }
}

fn alpha_permille(alpha: u8) -> i64 {
    (i64::from(alpha) * 100_000 / 255).clamp(0, 100_000)
}

fn line_xml(shape: &ShapeBox) -> String {
    let Some(hex) = &shape.line_hex else {
        return "                    <a:ln>\n                      <a:noFill/>\n                    </a:ln>\n".into();
    };
    let hex = crate::xml::word_hex_color(hex);
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
    // Keep behindDoc=0. Writer paints behindDoc under white `w:background`, so a
    // gradient/glass slice would vanish. Shape z-order (document order +
    // relativeHeight) keeps later text boxes in front.
    wp_anchor(
        pic.x_emu,
        pic.y_emu,
        pic.cx_emu,
        pic.cy_emu,
        pic.relative_height,
        false,
        doc_pr_id,
        &name,
        SHAPE_URI,
        &raster_wsp_xml(pic, embed_rid),
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
    use crate::ir::{LineDash, PictureBox, ShapeBox};

    fn box_at(corner_emu: i64) -> ShapeBox {
        ShapeBox {
            node_id: "card".into(),
            x_emu: 0,
            y_emu: 0,
            cx_emu: 100_000,
            cy_emu: 50_000,
            fill_hex: Some("1A73E8".into()),
            fill_alpha: 255,
            gradient: None,
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

    #[test]
    fn in_front_fill_is_empty_textbox() {
        let xml = shape_wsp_xml(&box_at(0));
        assert!(xml.contains("txBox=\"1\""), "{xml}");
        assert!(xml.contains("<w:txbxContent>"), "{xml}");
        assert!(xml.contains(r#"wrap="square""#), "{xml}");
        assert!(xml.contains("<a:noAutofit/>"), "{xml}");
        let mut behind = box_at(0);
        behind.behind_doc = true;
        let xml = shape_wsp_xml(&behind);
        assert!(!xml.contains("txBox="), "{xml}");
        assert!(!xml.contains("txbxContent"), "{xml}");
        assert!(!xml.contains("<a:noAutofit/>"), "{xml}");
    }

    #[test]
    fn raster_anchor_is_shape_with_blip_fill() {
        let pic = PictureBox {
            node_id: "snap.shell".into(),
            x_emu: 0,
            y_emu: 0,
            cx_emu: 100_000,
            cy_emu: 100_000,
            media_name: "raster1.png".into(),
            bytes: vec![],
            relative_height: 0,
        };
        let xml = raster_anchor(&pic, 2, "rId5");
        assert!(xml.contains("name=\"k2f-raster:snap.shell\""), "{xml}");
        assert!(xml.contains("<wps:wsp>"), "{xml}");
        assert!(xml.contains("<a:blipFill>"), "{xml}");
        assert!(xml.contains(r#"r:embed="rId5""#), "{xml}");
        assert!(!xml.contains("<pic:pic>"), "{xml}");
        assert!(xml.contains(r#"behindDoc="0""#), "{xml}");
        assert!(xml.contains("<a:noAutofit/>"), "{xml}");
        assert!(
            xml.contains("wordprocessingShape"),
            "raster must use the shape graphicData uri, got {xml}"
        );
    }

    #[test]
    fn gradient_shape_emits_grad_fill() {
        let mut s = box_at(0);
        s.fill_hex = None;
        s.gradient = Some(crate::ir::GradientFill {
            angle_degrees: 135,
            stops: vec![
                crate::ir::GradientStopFill {
                    pos: 0,
                    hex: "4338CA".into(),
                    alpha: 255,
                },
                crate::ir::GradientStopFill {
                    pos: 1000,
                    hex: "F59E0B".into(),
                    alpha: 255,
                },
            ],
        });
        let xml = shape_wsp_xml(&s);
        assert!(xml.contains("<a:gradFill>"), "{xml}");
        assert!(xml.contains(r#"ang="8100000""#), "{xml}");
        assert!(xml.contains("4338CA"), "{xml}");
        assert!(!xml.contains("<a:solidFill>"), "{xml}");
        assert!(
            !xml.contains("txBox="),
            "gradient must not be an empty txBox, got {xml}"
        );
    }
}
