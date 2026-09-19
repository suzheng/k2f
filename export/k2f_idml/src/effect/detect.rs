use crate::IdmlError;
use k2f_core::{BoxDecoration, NodeContent, Pt, Rect, SemanticNode, ROLE_MATH};
use k2f_paint::resolve_fill;
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

/// Boxes that cannot be native InDesign rectangles: blur. Linear gradients
/// and translucent solids/strokes stay native (`Gradient` fill /
/// `FillTransparencySetting`). Fraction rules stay with math.
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
    // Linear gradients, opaque solids, and `#RRGGBBAA` fills/strokes are
    // native InDesign rectangles. Rasterizing a translucent scrim as opaque
    // RGB (page paper behind it) hid the picture underneath — hero overlays
    // became a gray plate. Same iceberg as PPTX/DOCX `a:alpha`.
    match resolve_fill(decoration) {
        Ok(_) => Ok(false),
        Err(k2f_paint::PaintError::UnresolvedRef(name)) => {
            Err(IdmlError::Write(format!("unresolved fill ref '{name}'")))
        }
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::{Blur, BlurRef, Fill, FillRef, Shadow, ShadowRef};

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
    fn four_side_translucent_stroke_is_native() {
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
        assert!(
            !box_is_effect("card.front", &dec).unwrap(),
            "translucent four-side stroke must stay native Fill+Stroke + opacity"
        );
    }

    #[test]
    fn linear_gradient_is_not_effect() {
        use k2f_core::{GradientStop, LinearGradient};

        let dec = BoxDecoration {
            background: Some(FillRef::Inline(Fill::LinearGradient {
                value: LinearGradient::Linear {
                    angle_degrees: 135,
                    stops: vec![
                        GradientStop {
                            pos: 0,
                            color: "#FF8A00".into(),
                        },
                        GradientStop {
                            pos: 1000,
                            color: "#FF5E00".into(),
                        },
                    ],
                },
            })),
            ..Default::default()
        };
        assert!(
            !box_is_effect("hero.right.labels.pptx", &dec).unwrap(),
            "rounded gradient pills must stay native Gradient fills"
        );
    }
}
