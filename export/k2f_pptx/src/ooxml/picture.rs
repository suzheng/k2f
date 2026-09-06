use crate::ir::PictureBox;
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
    format!(
        r#"    <p:pic>
      <p:nvPicPr>
        <p:cNvPr id="{id}" name="{name}"/>
        <p:cNvPicPr><a:picLocks noChangeAspect="0"/></p:cNvPicPr>
        <p:nvPr/>
      </p:nvPicPr>
      <p:blipFill>
        <a:blip r:embed="{rid}"/>
        <a:stretch><a:fillRect/></a:stretch>
      </p:blipFill>
      <p:spPr>
        <a:xfrm>
          <a:off x="{x}" y="{y}"/>
          <a:ext cx="{cx}" cy="{cy}"/>
        </a:xfrm>
        <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
      </p:spPr>
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
