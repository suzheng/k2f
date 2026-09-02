use crate::ir::{SlideElement, SlideIR};
use crate::ooxml::{picture, shape, table, textbox};
use crate::xml::escape_xml;
use std::collections::BTreeMap;

pub struct SlideBindings {
    pub hyperlink_rids: BTreeMap<String, String>,
    pub picture_rids: BTreeMap<String, String>,
}

pub fn bind_slide(slide: &SlideIR) -> SlideBindings {
    let mut next = 2u32;
    let mut picture_rids = BTreeMap::new();
    for el in &slide.elements {
        match el {
            SlideElement::Picture(p) | SlideElement::Raster(p) => {
                picture_rids.insert(p.media_name.clone(), format!("rId{next}"));
                next = next.saturating_add(1);
            }
            SlideElement::TextBox(_) | SlideElement::Shape(_) | SlideElement::Table(_) => {}
        }
    }
    let hyperlink_rids = textbox::collect_hyperlink_urls(slide)
        .into_iter()
        .map(|url| {
            let rid = format!("rId{next}");
            next = next.saturating_add(1);
            (url, rid)
        })
        .collect();
    SlideBindings {
        hyperlink_rids,
        picture_rids,
    }
}

pub fn slide_xml(slide: &SlideIR, bindings: &SlideBindings) -> String {
    let mut shapes = String::new();
    let mut id = 2u32;
    for el in &slide.elements {
        match el {
            SlideElement::TextBox(tb) => {
                shapes.push_str(&textbox::textbox_sp_xml(tb, id, &bindings.hyperlink_rids));
                id = id.saturating_add(1);
            }
            SlideElement::Shape(s) => {
                shapes.push_str(&shape::shape_sp_xml(s, id));
                id = id.saturating_add(1);
            }
            SlideElement::Picture(p) => {
                let rid = bindings
                    .picture_rids
                    .get(&p.media_name)
                    .map(String::as_str)
                    .unwrap_or("");
                shapes.push_str(&picture::picture_xml(p, id, rid));
                id = id.saturating_add(1);
            }
            SlideElement::Raster(p) => {
                let rid = bindings
                    .picture_rids
                    .get(&p.media_name)
                    .map(String::as_str)
                    .unwrap_or("");
                shapes.push_str(&picture::raster_xml(p, id, rid));
                id = id.saturating_add(1);
            }
            SlideElement::Table(t) => {
                shapes.push_str(&table::table_graphic_frame_xml(
                    t,
                    id,
                    &bindings.hyperlink_rids,
                ));
                id = id.saturating_add(1);
            }
        }
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:bg>
      <p:bgPr>
        <a:solidFill><a:srgbClr val="{bg}"/></a:solidFill>
        <a:effectLst/>
      </p:bgPr>
    </p:bg>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr>
        <a:xfrm>
          <a:off x="0" y="0"/>
          <a:ext cx="{w}" cy="{h}"/>
          <a:chOff x="0" y="0"/>
          <a:chExt cx="{w}" cy="{h}"/>
        </a:xfrm>
      </p:grpSpPr>
{shapes}    </p:spTree>
  </p:cSld>
  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sld>
"#,
        bg = slide.bg_hex,
        w = slide.width_emu,
        h = slide.height_emu,
    )
}

pub fn slide_rels(bindings: &SlideBindings) -> String {
    let mut rels = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
"#,
    );
    let mut pics: Vec<_> = bindings.picture_rids.iter().collect();
    pics.sort_by_key(|(_, rid)| rid.as_str());
    for (media, rid) in pics {
        rels.push_str(&format!(
            r#"  <Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/{}"/>
"#,
            escape_xml(media)
        ));
    }
    let mut links: Vec<_> = bindings.hyperlink_rids.iter().collect();
    links.sort_by_key(|(_, rid)| rid.as_str());
    for (url, rid) in links {
        rels.push_str(&format!(
            r#"  <Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="{}" TargetMode="External"/>
"#,
            escape_xml(url)
        ));
    }
    rels.push_str("</Relationships>\n");
    rels
}
