use crate::IdmlError;
use k2f_core::{BoxDecoration, Fill, NodeContent, Pt, Rect, SemanticNode, ROLE_MATH};
use k2f_paint::{parse_hex_rgba, resolve_fill};
use std::collections::HashSet;

pub fn is_rule_id(node_id: &str) -> bool {
    node_id.contains("::rule_")
}

pub fn is_math_node(node: &SemanticNode) -> bool {
    node.role == ROLE_MATH || matches!(node.content, NodeContent::Math(_))
}

pub fn box_belongs_to_effect(node_id: &str, effect_ids: &HashSet<String>) -> bool {
    if effect_ids.contains(node_id) {
        return true;
    }
    effect_ids.iter().any(|id| {
        node_id.len() > id.len()
            && node_id.starts_with(id.as_str())
            && node_id[id.len()..].starts_with("::rule_")
    })
}

pub fn is_full_page(page_w: Pt, page_h: Pt, rect: &Rect) -> bool {
    rect.x == Pt(0) && rect.y == Pt(0) && rect.width == page_w && rect.height == page_h
}

/// Boxes that cannot be native InDesign rectangles: blur, linear gradient,
/// or translucent solid fill/stroke. Fraction rules stay with math.
///
/// Shadow-only boxes stay native fill+stroke. An opaque shadow PNG is
/// expanded for blur/spread, so it covers earlier labels that sit in the
/// glow halo (invoice totals, raised plaques). Same iceberg as PPTX/DOCX.
/// Keep rasters for blur (no native equivalent). Glow is a v1 gap.
pub fn box_is_effect(node_id: &str, decoration: &BoxDecoration) -> Result<bool, IdmlError> {
    if is_rule_id(node_id) {
        return Ok(false);
    }
    if decoration.blur.is_some() {
        return Ok(true);
    }
    if translucent_border(decoration)? {
        return Ok(!partial_translucent_border_is_native_edge(decoration)?);
    }
    match resolve_fill(decoration) {
        Ok(Some(Fill::LinearGradient { .. })) => Ok(true),
        Ok(Some(Fill::Solid { color })) => translucent_solid(&color),
        Ok(None) => Ok(false),
        Err(k2f_paint::PaintError::UnresolvedRef(name)) => {
            Err(IdmlError::Write(format!("unresolved fill ref '{name}'")))
        }
        Err(e) => Err(e.into()),
    }
}

fn translucent_solid(color: &str) -> Result<bool, IdmlError> {
    let [_, _, _, a] = parse_hex_rgba(color)
        .ok_or_else(|| IdmlError::Write(format!("unparseable fill color '{color}'")))?;
    Ok(a < 255)
}

/// Opaque native Stroke strips the alpha byte, so a 10% hairline becomes a
/// solid ink rim. Slice those boxes instead (same path as translucent fill).
fn translucent_border(decoration: &BoxDecoration) -> Result<bool, IdmlError> {
    let Some(border) = decoration.border.as_ref() else {
        return Ok(false);
    };
    if border.width_pt <= 0 || border.edges.is_empty() {
        return Ok(false);
    }
    let [_, _, _, a] = parse_hex_rgba(&border.color)
        .ok_or_else(|| IdmlError::Write(format!("unparseable color '{}'", border.color)))?;
    Ok(a > 0 && a < 255)
}

/// Partial-edge translucent borders on signature underlines share a node with
/// native text. Slicing the full box duplicates the label; emit `::edge_*`
/// bars with opaque RGB instead (v1 alpha loss on a hairline is acceptable).
fn partial_translucent_border_is_native_edge(decoration: &BoxDecoration) -> Result<bool, IdmlError> {
    let Some(border) = decoration.border.as_ref() else {
        return Ok(false);
    };
    if border.width_pt <= 0 || border.edges.is_empty() {
        return Ok(false);
    }
    if border.is_full_rect_stroke() || border.draws_all_four_edges() {
        return Ok(false);
    }
    match resolve_fill(decoration) {
        Ok(Some(Fill::LinearGradient { .. })) => Ok(false),
        Ok(Some(Fill::Solid { color })) => {
            let [_, _, _, a] = parse_hex_rgba(&color)
                .ok_or_else(|| IdmlError::Write(format!("unparseable fill color '{color}'")))?;
            Ok(a >= 255)
        }
        Ok(None) => Ok(true),
        Err(k2f_paint::PaintError::UnresolvedRef(name)) => {
            Err(IdmlError::Write(format!("unresolved fill ref '{name}'")))
        }
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::{Blur, BlurRef, FillRef, Shadow, ShadowRef};

    fn opaque_glow_plaque() -> BoxDecoration {
        BoxDecoration {
            background: Some(FillRef::Inline(Fill::Solid {
                color: "#092230".into(),
            })),
            shadow: Some(ShadowRef::Ref("glow_cyan".into())),
            ..Default::default()
        }
    }

    #[test]
    fn shadow_only_opaque_box_is_not_effect() {
        let dec = opaque_glow_plaque();
        assert!(
            !box_is_effect("card.grand_total", &dec).unwrap(),
            "engine shadow must not force a k2f-raster slice"
        );
        let layers = BoxDecoration {
            background: Some(FillRef::Inline(Fill::Solid {
                color: "#0F1424".into(),
            })),
            shadow: Some(ShadowRef::Inline(Shadow { layers: vec![] })),
            ..Default::default()
        };
        assert!(!box_is_effect("card.panel", &layers).unwrap());
    }

    #[test]
    fn blur_still_is_effect() {
        let mut dec = opaque_glow_plaque();
        dec.blur = Some(BlurRef::Inline(Blur { radius_pt: 8 }));
        assert!(box_is_effect("card.glass", &dec).unwrap());
    }

    #[test]
    fn partial_translucent_underline_is_native_not_effect() {
        use k2f_core::{Border, BorderEdge, BorderStyle};

        let dec = BoxDecoration {
            border: Some(Border {
                width_pt: 500,
                color: "#C5A05966".into(),
                edges: vec![BorderEdge::Bottom],
                style: BorderStyle::Solid,
            }),
            ..Default::default()
        };
        assert!(
            !box_is_effect("card.back.date_line", &dec).unwrap(),
            "partial translucent underline must not force a full-box raster"
        );
    }

    #[test]
    fn four_side_translucent_stroke_stays_effect() {
        use k2f_core::{Border, BorderEdge, BorderStyle, FillRef};

        let dec = BoxDecoration {
            background: Some(FillRef::Inline(Fill::Solid {
                color: "#F7FAFA".into(),
            })),
            border: Some(Border {
                width_pt: 400,
                color: "#0E2A2A1A".into(),
                edges: vec![
                    BorderEdge::Top,
                    BorderEdge::Right,
                    BorderEdge::Bottom,
                    BorderEdge::Left,
                ],
                style: BorderStyle::Solid,
            }),
            ..Default::default()
        };
        assert!(box_is_effect("card.front", &dec).unwrap());
    }
}
