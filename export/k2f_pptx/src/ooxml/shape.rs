use crate::ir::{LineDash, ShapeBox};
use crate::shape::round_rect_adj;
use crate::xml::escape_xml;

pub(crate) fn shape_sp_xml(shape: &ShapeBox, cnv_id: u32) -> String {
    let name = escape_xml(&shape.node_id);
    let geom = geom_xml(shape);
    let fill = match &shape.fill_hex {
        Some(hex) => solid_fill_xml(hex, shape.fill_alpha),
        None => "        <a:noFill/>\n".into(),
    };
    let ln = line_xml(shape);
    format!(
        r#"    <p:sp>
      <p:nvSpPr>
        <p:cNvPr id="{id}" name="{name}"/>
        <p:cNvSpPr/>
        <p:nvPr/>
      </p:nvSpPr>
      <p:spPr>
        <a:xfrm>
          <a:off x="{x}" y="{y}"/>
          <a:ext cx="{cx}" cy="{cy}"/>
        </a:xfrm>
{geom}{fill}{ln}      </p:spPr>
      <p:txBody>
        <a:bodyPr wrap="square" lIns="0" tIns="0" rIns="0" bIns="0" rtlCol="0">
          <a:noAutofit/>
        </a:bodyPr>
        <a:lstStyle/>
        <a:p><a:endParaRPr lang="en-US"/></a:p>
      </p:txBody>
    </p:sp>
"#,
        id = cnv_id,
        x = shape.x_emu,
        y = shape.y_emu,
        cx = shape.cx_emu,
        cy = shape.cy_emu,
    )
}

fn geom_xml(shape: &ShapeBox) -> String {
    if shape.corner_emu <= 0 {
        return "        <a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom>\n".into();
    }
    let adj = round_rect_adj(shape.corner_emu, shape.cx_emu, shape.cy_emu);
    format!(
        "        <a:prstGeom prst=\"roundRect\"><a:avLst><a:gd name=\"adj\" fmla=\"val {adj}\"/></a:avLst></a:prstGeom>\n"
    )
}

fn solid_fill_xml(hex: &str, alpha: u8) -> String {
    format!(
        "        <a:solidFill>{}</a:solidFill>\n",
        srgb_clr_xml(hex, alpha)
    )
}

fn srgb_clr_xml(hex: &str, alpha: u8) -> String {
    if alpha >= 255 {
        format!("<a:srgbClr val=\"{hex}\"/>")
    } else {
        format!(
            "<a:srgbClr val=\"{hex}\"><a:alpha val=\"{a}\"/></a:srgbClr>",
            a = (i64::from(alpha) * 100_000 / 255).clamp(0, 100_000)
        )
    }
}

fn line_xml(shape: &ShapeBox) -> String {
    let Some(hex) = &shape.line_hex else {
        return "        <a:ln><a:noFill/></a:ln>\n".into();
    };
    let dash = match shape.line_dash {
        LineDash::Solid => "solid",
        LineDash::Dash => "dash",
        LineDash::Dot => "sysDot",
    };
    format!(
        "        <a:ln w=\"{w}\"><a:solidFill>{}</a:solidFill><a:prstDash val=\"{dash}\"/></a:ln>\n",
        srgb_clr_xml(hex, shape.line_alpha),
        w = shape.line_w_emu
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::LineDash;

    fn stroke(alpha: u8) -> ShapeBox {
        ShapeBox {
            node_id: "card".into(),
            x_emu: 0,
            y_emu: 0,
            cx_emu: 100_000,
            cy_emu: 50_000,
            fill_hex: None,
            fill_alpha: 255,
            corner_emu: 0,
            line_hex: Some("1E3A8A".into()),
            line_alpha: alpha,
            line_w_emu: 6_350,
            line_dash: LineDash::Solid,
        }
    }

    #[test]
    fn opaque_stroke_omits_alpha() {
        let xml = line_xml(&stroke(255));
        assert!(xml.contains(r#"<a:srgbClr val="1E3A8A"/>"#), "{xml}");
        assert!(!xml.contains("<a:alpha"), "{xml}");
    }

    #[test]
    fn translucent_stroke_emits_alpha() {
        let xml = line_xml(&stroke(0x66));
        assert!(xml.contains(r#"<a:srgbClr val="1E3A8A"><a:alpha val="40000"/>"#), "{xml}");
    }

    #[test]
    fn translucent_fill_emits_alpha() {
        let mut shape = stroke(255);
        shape.fill_hex = Some("FFFFFF".into());
        shape.fill_alpha = 0xE6;
        shape.line_hex = None;
        let xml = shape_sp_xml(&shape, 2);
        assert!(
            xml.contains(r#"<a:srgbClr val="FFFFFF"><a:alpha val="90196"/>"#),
            "{xml}"
        );
        assert!(!xml.contains("k2f-raster:"), "{xml}");
    }
}
