use crate::ir::PictureBox;
use crate::shape::round_rect_adj;
use crate::xml::escape_xml;

pub(crate) fn picture_xml(pic: &PictureBox, cnv_id: u32, embed_rid: &str) -> String {
    pic_xml(pic, cnv_id, embed_rid, &pic.node_id)
}

pub(crate) fn raster_xml(pic: &PictureBox, cnv_id: u32, embed_rid: &str) -> String {
    pic_xml(
        pic,
        cnv_id,
        embed_rid,
        &format!("k2f-raster:{}", pic.node_id),
    )
}

fn pic_xml(pic: &PictureBox, cnv_id: u32, embed_rid: &str, name: &str) -> String {
    let name = escape_xml(name);
    let src_rect = src_rect_xml(pic);
    let geom = geom_xml(pic);
    format!(
        r#"    <p:pic>
      <p:nvPicPr>
        <p:cNvPr id="{id}" name="{name}"/>
        <p:cNvPicPr><a:picLocks noChangeAspect="0"/></p:cNvPicPr>
        <p:nvPr/>
      </p:nvPicPr>
      <p:blipFill>
        <a:blip r:embed="{rid}"/>
{src_rect}        <a:stretch><a:fillRect/></a:stretch>
      </p:blipFill>
      <p:spPr>
        <a:xfrm>
          <a:off x="{x}" y="{y}"/>
          <a:ext cx="{cx}" cy="{cy}"/>
        </a:xfrm>
{geom}      </p:spPr>
    </p:pic>
"#,
        id = cnv_id,
        rid = embed_rid,
        x = pic.x_emu,
        y = pic.y_emu,
        cx = pic.cx_emu,
        cy = pic.cy_emu,
    )
}

fn src_rect_xml(pic: &PictureBox) -> String {
    if pic.src_l == 0 && pic.src_t == 0 && pic.src_r == 0 && pic.src_b == 0 {
        String::new()
    } else {
        format!(
            "        <a:srcRect l=\"{}\" t=\"{}\" r=\"{}\" b=\"{}\"/>\n",
            pic.src_l, pic.src_t, pic.src_r, pic.src_b
        )
    }
}

fn geom_xml(pic: &PictureBox) -> String {
    if pic.corner_emu <= 0 {
        return "        <a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom>\n".into();
    }
    let adj = round_rect_adj(pic.corner_emu, pic.cx_emu, pic.cy_emu);
    format!(
        "        <a:prstGeom prst=\"roundRect\"><a:avLst><a:gd name=\"adj\" fmla=\"val {adj}\"/></a:avLst></a:prstGeom>\n"
    )
}
