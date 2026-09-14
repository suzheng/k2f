use super::ids::{bg_self, page_self, spread_self};
use crate::coord::{item_transform, local_rect_path, SpreadSpace, DOM, NS};
use crate::IdmlError;
use k2f_core::{Fill, Page, PaintOp, Pt};
use k2f_paint::{parse_hex_rgba, resolve_fill};

const XML_DECL: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#;

pub fn spread_xml(i: usize, space: &SpreadSpace, fill_hex: Option<&str>) -> String {
    let bounds = space.page_geometric_bounds();
    let page_name = i + 1;
    let mut body = format!(
        r#"    <Page Self="{page}" AppliedMaster="kMaster" GeometricBounds="{bounds}" ItemTransform="1 0 0 1 0 0" Name="{page_name}"/>
"#,
        page = page_self(i),
    );
    if let Some(hex) = fill_hex {
        body.push_str(&page_fill_rectangle(i, space, hex));
    }
    format!(
        r#"{XML_DECL}
<idPkg:Spread xmlns:idPkg="{NS}" DOMVersion="{DOM}">
  <Spread Self="{spread}" PageCount="1" BindingLocation="0" ItemTransform="1 0 0 1 0 0" FlattenerOverride="Default">
{body}  </Spread>
</idPkg:Spread>
"#,
        spread = spread_self(i),
    )
}

fn page_fill_rectangle(i: usize, space: &SpreadSpace, hex: &str) -> String {
    let tf = item_transform(0.0, 0.0);
    let geo = path_geometry_xml(space.page_w, space.page_h);
    format!(
        r#"    <Rectangle Self="{id}" ContentType="Unassigned" ItemLayer="kLayer" FillColor="Color/k2f_{hex}" StrokeWeight="0" ItemTransform="{tf}">
{geo}
    </Rectangle>
"#,
        id = bg_self(i),
    )
}

fn path_geometry_xml(w: f64, h: f64) -> String {
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

/// Full-page opaque solid from ops[0] only. Does not scan later DrawBox/DrawText.
pub fn page_fill_hex(page: &Page, ops: &[PaintOp]) -> Result<Option<String>, IdmlError> {
    let Some(PaintOp::DrawBox {
        rect, decoration, ..
    }) = ops.first()
    else {
        return Ok(None);
    };
    if !is_full_page(page, rect) {
        return Ok(None);
    }
    match resolve_fill(decoration) {
        Ok(Some(Fill::Solid { color })) => Ok(opaque_srgb_hex(&color)),
        Ok(_) => Ok(None),
        Err(e) => Err(IdmlError::Write(format!("unresolved fill ref: {e}"))),
    }
}

fn is_full_page(page: &Page, rect: &k2f_core::Rect) -> bool {
    rect.x == Pt(0) && rect.y == Pt(0) && rect.width == page.width && rect.height == page.height
}

fn opaque_srgb_hex(color: &str) -> Option<String> {
    let [r, g, b, a] = parse_hex_rgba(color)?;
    if a != 255 {
        return None;
    }
    Some(format!("{r:02X}{g:02X}{b:02X}"))
}
