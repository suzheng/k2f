use super::ids::{page_self, spread_self};
use crate::coord::{fmt_pt, item_transform, local_rect_path, pt_val, SpreadSpace, DOM, NS};
use crate::ir::{LineDash, PictureBox, ShapeBox, TextBox};
use crate::xml::escape_xml;
use base64::Engine;

const XML_DECL: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#;

pub fn spread_xml(i: usize, space: &SpreadSpace, items: &str) -> String {
    let bounds = space.page_geometric_bounds();
    let page_name = i + 1;
    format!(
        r#"{XML_DECL}
<idPkg:Spread xmlns:idPkg="{NS}" DOMVersion="{DOM}">
  <Spread Self="{spread}" PageCount="1" BindingLocation="0" ItemTransform="1 0 0 1 0 0" FlattenerOverride="Default">
    <Page Self="{page}" AppliedMaster="kMaster" GeometricBounds="{bounds}" ItemTransform="1 0 0 1 0 0" Name="{page_name}"/>
{items}  </Spread>
</idPkg:Spread>
"#,
        spread = spread_self(i),
        page = page_self(i),
    )
}

pub fn textframe_xml(tb: &TextBox, space: &SpreadSpace, tf_self: &str, story_self: &str) -> String {
    let (tx, ty) = space.box_center(&tb.rect);
    let tf = item_transform(tx, ty);
    let w = pt_val(tb.rect.width);
    let h = pt_val(tb.rect.height);
    let geo = path_geometry_xml(w, h);
    let name = escape_xml(&tb.node_id);
    let vert = if tb.vert_center {
        "CenterAlign"
    } else {
        "TopAlign"
    };
    let inset = format!(
        "{} {} {} {}",
        fmt_pt(tb.inset_top),
        fmt_pt(tb.inset_left),
        fmt_pt(tb.inset_bottom),
        fmt_pt(tb.inset_right)
    );
    let autosize = if tb.autosize_width {
        let no_wrap = if tb.autosize_no_wrap {
            r#" UseNoLineBreaksForAutoSizing="true""#
        } else {
            r#" UseNoLineBreaksForAutoSizing="false""#
        };
        format!(
            r#" AutoSizingType="WidthOnly" AutoSizingReferencePoint="{}"{no_wrap}"#,
            tb.autosize_refer
        )
    } else if tb.autosize_height {
        let refer = match tb.align {
            crate::ir::TextAlign::Center => "TopCenterPoint",
            crate::ir::TextAlign::Right => "TopRightPoint",
            crate::ir::TextAlign::Left | crate::ir::TextAlign::Justify => "TopLeftPoint",
        };
        format!(r#" AutoSizingType="HeightOnly" AutoSizingReferencePoint="{refer}""#)
    } else {
        String::new()
    };
    format!(
        r#"    <TextFrame Self="{tf_self}" ParentStory="{story_self}" ContentType="TextType" ItemLayer="kLayer" FillColor="Swatch/None" StrokeWeight="0" ItemTransform="{tf}" Name="{name}" NextTextFrame="n" PreviousTextFrame="n">
{geo}
      <TextFramePreference TextColumnCount="1" VerticalJustification="{vert}" InsetSpacing="{inset}" FirstBaselineOffset="Ascent"{autosize}/>
    </TextFrame>
"#
    )
}

pub fn rectangle_xml(shape: &ShapeBox, space: &SpreadSpace, self_id: &str) -> String {
    let (tx, ty) = space.box_center(&shape.rect);
    let tf = item_transform(tx, ty);
    let w = pt_val(shape.rect.width);
    let h = pt_val(shape.rect.height);
    let geo = path_geometry_xml(w, h);
    let name = escape_xml(&shape.node_id);
    let fill = match &shape.fill_hex {
        Some(hex) => format!("FillColor=\"Color/k2f_{hex}\""),
        None => "FillColor=\"Swatch/None\"".into(),
    };
    let stroke = stroke_attrs(shape);
    let corners = if shape.corner_pt > 0.0 {
        // DOM 16 (CS5+ live corners) ignores CS4 CornerOption/CornerRadius unless
        // each corner is set. Write both so CS4 readers still round uniformly.
        let r = fmt_pt(shape.corner_pt);
        format!(
            " CornerOption=\"RoundedCorner\" CornerRadius=\"{r}\" \
             TopLeftCornerOption=\"RoundedCorner\" TopLeftCornerRadius=\"{r}\" \
             TopRightCornerOption=\"RoundedCorner\" TopRightCornerRadius=\"{r}\" \
             BottomLeftCornerOption=\"RoundedCorner\" BottomLeftCornerRadius=\"{r}\" \
             BottomRightCornerOption=\"RoundedCorner\" BottomRightCornerRadius=\"{r}\""
        )
    } else {
        String::new()
    };
    format!(
        r#"    <Rectangle Self="{self_id}" ContentType="Unassigned" ItemLayer="kLayer" {fill}{stroke}{corners} ItemTransform="{tf}" Name="{name}">
{geo}
    </Rectangle>
"#
    )
}

pub fn picture_xml(pic: &PictureBox, space: &SpreadSpace, rect_id: &str, img_id: &str) -> String {
    framed_image_xml(pic, space, rect_id, img_id, &pic.node_id)
}

pub fn raster_xml(pic: &PictureBox, space: &SpreadSpace, rect_id: &str, img_id: &str) -> String {
    framed_image_xml(
        pic,
        space,
        rect_id,
        img_id,
        &format!("k2f-raster:{}", pic.node_id),
    )
}

fn framed_image_xml(
    pic: &PictureBox,
    space: &SpreadSpace,
    rect_id: &str,
    img_id: &str,
    name: &str,
) -> String {
    let (tx, ty) = space.box_center(&pic.rect);
    let tf = item_transform(tx, ty);
    let w = pt_val(pic.rect.width);
    let h = pt_val(pic.rect.height);
    let geo = path_geometry_xml(w, h);
    let name = escape_xml(name);
    let type_name = image_type_name(&pic.ext);
    let b64 = b64_76(&pic.bytes);
    let left = fmt_pt(-w / 2.0);
    let top = fmt_pt(-h / 2.0);
    let right = fmt_pt(w / 2.0);
    let bottom = fmt_pt(h / 2.0);
    format!(
        r#"    <Rectangle Self="{rect_id}" ContentType="GraphicType" ItemLayer="kLayer" FillColor="Swatch/None" StrokeWeight="0" ItemTransform="{tf}" Name="{name}">
{geo}
      <Image Self="{img_id}" ImageTypeName="{type_name}" Space="RGB" ItemTransform="1 0 0 1 0 0">
        <Properties>
          <GraphicBounds Left="{left}" Top="{top}" Right="{right}" Bottom="{bottom}"/>
          <Contents>
            <![CDATA[{b64}]]>
          </Contents>
        </Properties>
      </Image>
    </Rectangle>
"#
    )
}

fn stroke_attrs(shape: &ShapeBox) -> String {
    match &shape.line_hex {
        Some(hex) => format!(
            " StrokeColor=\"Color/k2f_{hex}\" StrokeWeight=\"{}\" StrokeType=\"{}\"",
            fmt_pt(shape.line_w_pt),
            stroke_type(shape.line_dash)
        ),
        None => " StrokeWeight=\"0\"".into(),
    }
}

fn stroke_type(dash: LineDash) -> &'static str {
    match dash {
        LineDash::Solid => "$ID/Solid",
        LineDash::Dash => "$ID/Dashed",
        LineDash::Dot => "$ID/Dotted",
    }
}

fn image_type_name(ext: &str) -> &'static str {
    match ext {
        "jpg" | "jpeg" => "$ID/JPEG",
        _ => "$ID/PNG",
    }
}

fn b64_76(bytes: &[u8]) -> String {
    let s = base64::engine::general_purpose::STANDARD.encode(bytes);
    let mut out = String::with_capacity(s.len() + s.len() / 76);
    for (i, chunk) in s.as_bytes().chunks(76).enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(std::str::from_utf8(chunk).unwrap());
    }
    out
}

pub(crate) fn path_geometry_xml(w: f64, h: f64) -> String {
    let [tl, tr, br, bl] = local_rect_path(w, h);
    format!(
        r#"      <Properties>
        <PathGeometry>
          <GeometryPathType PathOpen="false">
            <PathPointArray>
              <PathPointType Anchor="{tl}" LeftDirection="{tl}" RightDirection="{tl}"/>
              <PathPointType Anchor="{tr}" LeftDirection="{tr}" RightDirection="{tr}"/>
              <PathPointType Anchor="{br}" LeftDirection="{br}" RightDirection="{br}"/>
              <PathPointType Anchor="{bl}" LeftDirection="{bl}" RightDirection="{bl}"/>
            </PathPointArray>
          </GeometryPathType>
        </PathGeometry>
      </Properties>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{LineDash, ShapeBox};
    use k2f_core::{Pt, Rect};

    #[test]
    fn rounded_rect_emits_live_corner_attrs() {
        let shape = ShapeBox {
            node_id: "pill".into(),
            rect: Rect {
                x: Pt(0),
                y: Pt(0),
                width: Pt(80_000),
                height: Pt(20_000),
            },
            fill_hex: Some("E05A47".into()),
            fill_alpha: 255,
            corner_pt: 20.0,
            line_hex: None,
            line_w_pt: 0.0,
            line_dash: LineDash::Solid,
        };
        let space = SpreadSpace {
            page_w: 960.0,
            page_h: 540.0,
        };
        let xml = rectangle_xml(&shape, &space, "kRect0");
        for attr in [
            "TopLeftCornerOption=\"RoundedCorner\"",
            "TopRightCornerOption=\"RoundedCorner\"",
            "BottomLeftCornerOption=\"RoundedCorner\"",
            "BottomRightCornerOption=\"RoundedCorner\"",
            "TopLeftCornerRadius=\"20.000\"",
            "TopRightCornerRadius=\"20.000\"",
            "BottomLeftCornerRadius=\"20.000\"",
            "BottomRightCornerRadius=\"20.000\"",
        ] {
            assert!(xml.contains(attr), "missing {attr} in {xml}");
        }
    }
}
