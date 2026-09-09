use crate::coord::{millipt_to_emu, pt_to_emu};
use crate::ir::{LineDash, ShapeBox};
use crate::PptxError;
use k2f_core::{BorderStyle, BoxDecoration, Fill, Rect};
use k2f_paint::{parse_hex_rgba, resolve_fill};

pub(crate) fn shape_from_box(
    node_id: &str,
    rect: &Rect,
    decoration: &BoxDecoration,
) -> Result<Option<ShapeBox>, PptxError> {
    if crate::effect::is_rule_id(node_id) || crate::effect::box_is_effect(node_id, decoration)? {
        return Ok(None);
    }
    let fill = match resolve_fill(decoration) {
        Ok(fill) => fill,
        Err(k2f_paint::PaintError::UnresolvedRef(name)) => {
            return Err(PptxError::Write(format!("unresolved fill ref '{name}'")));
        }
        Err(e) => return Err(e.into()),
    };
    let fill_hex = match fill {
        Some(Fill::LinearGradient { .. }) => return Ok(None),
        Some(Fill::Solid { color }) => Some(srgb_hex(&color)?),
        None => None,
    };
    let (line_hex, line_w_emu, line_dash) = line_from(decoration)?;
    if fill_hex.is_none() && line_hex.is_none() {
        return Ok(None);
    }
    Ok(Some(ShapeBox {
        node_id: node_id.to_string(),
        x_emu: pt_to_emu(rect.x),
        y_emu: pt_to_emu(rect.y),
        cx_emu: pt_to_emu(rect.width),
        cy_emu: pt_to_emu(rect.height),
        fill_hex,
        fill_alpha_ppt: None,
        corner_emu: millipt_to_emu(decoration.corner_radius_pt.unwrap_or(0).max(0)),
        line_hex,
        line_w_emu,
        line_dash,
    }))
}

fn line_from(decoration: &BoxDecoration) -> Result<(Option<String>, i64, LineDash), PptxError> {
    let Some(border) = decoration.border.as_ref() else {
        return Ok((None, 0, LineDash::Solid));
    };
    if border.width_pt <= 0 {
        return Ok((None, 0, LineDash::Solid));
    }
    Ok((
        Some(srgb_hex(&border.color)?),
        millipt_to_emu(border.width_pt),
        match border.style {
            BorderStyle::Solid => LineDash::Solid,
            BorderStyle::Dashed => LineDash::Dash,
            BorderStyle::Dotted => LineDash::Dot,
        },
    ))
}

fn srgb_hex(color: &str) -> Result<String, PptxError> {
    let [r, g, b, _] = parse_hex_rgba(color)
        .ok_or_else(|| PptxError::Write(format!("unparseable color '{color}'")))?;
    Ok(crate::text::pin_office_srgb(&format!("{r:02X}{g:02X}{b:02X}")))
}

pub(crate) fn round_rect_adj(corner_emu: i64, cx: i64, cy: i64) -> i64 {
    let min = cx.min(cy);
    if corner_emu <= 0 || min <= 0 {
        return 0;
    }
    (corner_emu.saturating_mul(100_000) / min).clamp(0, 50_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::{FillRef, Pt, Shadow, ShadowRef};

    fn rect() -> Rect {
        Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(100_000),
            height: Pt(50_000),
        }
    }

    #[test]
    fn skips_shadow_and_rule_and_empty() {
        let mut dec = BoxDecoration {
            background: Some(FillRef::Inline(Fill::Solid {
                color: "#FFFFFF".into(),
            })),
            shadow: Some(ShadowRef::Inline(Shadow { layers: vec![] })),
            ..Default::default()
        };
        assert!(shape_from_box("card", &rect(), &dec).unwrap().is_none());
        dec.shadow = None;
        assert!(shape_from_box("eq::rule_1", &rect(), &dec)
            .unwrap()
            .is_none());
        let empty = BoxDecoration::default();
        assert!(shape_from_box("empty", &rect(), &empty).unwrap().is_none());
    }

    #[test]
    fn emits_opaque_page_background() {
        let dec = BoxDecoration {
            background: Some(FillRef::Inline(Fill::Solid {
                color: "#FFFFFF".into(),
            })),
            ..Default::default()
        };
        let s = shape_from_box("root::page_0::background", &rect(), &dec)
            .unwrap()
            .expect("page bg");
        assert_eq!(s.fill_hex.as_deref(), Some("FFFFFE"));
        assert_eq!(s.corner_emu, 0);
    }

    #[test]
    fn adj_clamped_to_half_min_side() {
        assert_eq!(round_rect_adj(0, 10_000, 10_000), 0);
        assert_eq!(round_rect_adj(1_000, 10_000, 20_000), 10_000);
        assert_eq!(round_rect_adj(10_000, 10_000, 10_000), 50_000);
    }

    #[test]
    fn unresolved_fill_ref_fails_export() {
        let dec = BoxDecoration {
            background: Some(FillRef::Ref("missing_surface".into())),
            ..Default::default()
        };
        let err = shape_from_box("card", &rect(), &dec).unwrap_err();
        match err {
            PptxError::Write(msg) => assert!(msg.contains("unresolved fill ref")),
            other => panic!("expected Write, got {other:?}"),
        }
    }
}
