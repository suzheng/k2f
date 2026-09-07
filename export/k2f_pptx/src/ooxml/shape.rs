use crate::ir::{LineDash, ShapeBox};
use crate::shape::round_rect_adj;
use crate::xml::escape_xml;

pub(crate) fn shape_sp_xml(shape: &ShapeBox, cnv_id: u32) -> String {
    let name = escape_xml(&shape.node_id);
    let geom = geom_xml(shape);
    let fill = match &shape.fill_hex {
        Some(hex) => format!("        <a:solidFill><a:srgbClr val=\"{hex}\"/></a:solidFill>\n"),
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
        "        <a:ln w=\"{w}\"><a:solidFill><a:srgbClr val=\"{hex}\"/></a:solidFill><a:prstDash val=\"{dash}\"/></a:ln>\n",
        w = shape.line_w_emu
    )
}
