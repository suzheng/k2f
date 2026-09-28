use k2f_core::{
    Border, BorderEdge, BorderStyle, BoxDecoration, Fill, FillRef, GradientStop, LinearGradient,
    PaintOp, Pt, Rect, Shadow, ShadowLayer, ShadowRef,
};
use k2f_paint::OpenedDocument;
use k2f_pdf::{box_is_slice, pdf_embedding_rgba, raster_slice, slices_for_page};
use std::path::PathBuf;

mod helpers;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn open_published(name: &str) -> OpenedDocument {
    let bytes = std::fs::read(repo_root().join(name)).unwrap();
    OpenedDocument::open(&bytes).unwrap()
}

fn rect(x: i128, y: i128, w: i128, h: i128) -> Rect {
    Rect {
        x: Pt(x),
        y: Pt(y),
        width: Pt(w),
        height: Pt(h),
    }
}

fn solid(color: &str) -> BoxDecoration {
    BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: color.into(),
        })),
        ..Default::default()
    }
}

fn shadow_layer(blur: i64) -> ShadowRef {
    ShadowRef::Inline(Shadow {
        layers: vec![ShadowLayer {
            offset_x_pt: 0,
            offset_y_pt: 2000,
            blur_radius_pt: blur,
            spread_radius_pt: 0,
            color: "#00000033".into(),
        }],
    })
}

fn page_ops(doc: &OpenedDocument, page_idx: usize) -> Vec<PaintOp> {
    let lock = doc.lock().unwrap();
    let page = &lock.geometry.pages[page_idx];
    lock.render_plan
        .pages
        .iter()
        .find(|p| p.index == page.index)
        .unwrap()
        .ops
        .clone()
}

#[test]
fn opaque_solid_is_not_a_slice_and_rule_is_not() {
    assert!(!box_is_slice("card", &solid("#112233")).unwrap());
    let mut ruled = solid("#112233");
    ruled.shadow = Some(shadow_layer(4000));
    assert!(!box_is_slice("math::rule_1", &ruled).unwrap());
}

#[test]
fn gradient_translucent_fill_and_border_are_slices() {
    let grad = BoxDecoration {
        background: Some(FillRef::Inline(Fill::LinearGradient {
            value: LinearGradient::Linear {
                angle_degrees: 90,
                stops: vec![
                    GradientStop {
                        pos: 0,
                        color: "#000000".into(),
                    },
                    GradientStop {
                        pos: 1000,
                        color: "#FFFFFF".into(),
                    },
                ],
            },
        })),
        ..Default::default()
    };
    assert!(box_is_slice("band", &grad).unwrap());
    assert!(box_is_slice("veil", &solid("#00000080")).unwrap());
    let border = BoxDecoration {
        border: Some(Border {
            width_pt: 500,
            color: "#2021244D".into(),
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
    assert!(box_is_slice("frame", &border).unwrap());
}

#[test]
fn shadow_crop_is_larger_than_the_box_and_clamped_to_the_page() {
    let mut dec = solid("#092230");
    dec.shadow = Some(shadow_layer(5000));
    let ops = vec![PaintOp::DrawBox {
        node_id: "plaque".into(),
        rect: rect(10_000, 10_000, 20_000, 20_000),
        decoration: dec.clone(),
    }];
    let slices = slices_for_page(&ops, Pt(100_000), Pt(100_000)).unwrap();
    assert_eq!(slices.len(), 1);
    assert!(slices[0].crop.width.0 > 20_000);
    assert!(slices[0].crop.height.0 > 20_000);
    assert!(slices[0].color_ops.iter().all(|op| !matches!(op, PaintOp::DrawText { .. })));

    let edge = vec![PaintOp::DrawBox {
        node_id: "plaque".into(),
        rect: rect(0, 0, 8_000, 8_000),
        decoration: dec,
    }];
    let clamped = slices_for_page(&edge, Pt(10_000), Pt(10_000)).unwrap();
    let crop = &clamped[0].crop;
    assert!(crop.x.0 >= 0 && crop.y.0 >= 0);
    assert!(crop.x.0 + crop.width.0 <= 10_000);
    assert!(crop.y.0 + crop.height.0 <= 10_000);
    assert!(crop.width.0 >= 1 && crop.height.0 >= 1);
}

#[test]
fn invoice_pages_have_no_slices() {
    let doc = open_published("examples/published/invoice.K2F");
    let lock = doc.lock().unwrap();
    for i in 0..lock.geometry.pages.len() {
        let page = &lock.geometry.pages[i];
        let ops = page_ops(&doc, i);
        let slices = slices_for_page(&ops, page.width, page.height).unwrap();
        assert!(slices.is_empty(), "invoice page {i} must stay vector");
    }
    let pdf = k2f_pdf::export_opened(&doc, k2f_pdf::PdfScale::DEFAULT).unwrap();
    assert!(
        !pdf.windows(6).any(|w| w == b"/SMask"),
        "vector invoice must not embed effect slices"
    );
}

#[test]
fn glass_card_merges_blur_and_plate_and_keeps_its_text_vector() {
    let doc = helpers::glass_card();
    let lock = doc.lock().unwrap();
    let page = &lock.geometry.pages[0];
    let ops = page_ops(&doc, 0);
    let slices = slices_for_page(&ops, page.width, page.height).unwrap();
    let blur_at = ops.iter().position(|op| {
        matches!(op, PaintOp::BackdropBlur { node_id, .. } if node_id == "card1")
    });
    let blur_at = blur_at.expect("card1 backdrop blur");
    let slice = slices
        .iter()
        .find(|s| s.anchor == blur_at)
        .expect("one slice anchored on the blur");
    assert!(
        slice.consumed.len() >= 2,
        "blur and the following plate share a slice"
    );
    assert!(
        slice
            .color_ops
            .iter()
            .all(|op| !matches!(op, PaintOp::DrawText { node_id, .. } if node_id == "card1")),
        "card text stays out of the chrome raster"
    );
    let text_at = ops.iter().position(|op| {
        matches!(op, PaintOp::DrawText { node_id, .. } if node_id == "card1")
    });
    let text_at = text_at.expect("card1 text");
    assert!(!slice.consumed.contains(&text_at));
    assert!(
        slice.cover.is_some(),
        "glass blur keeps a cover rect separate from later vector text"
    );
}

#[test]
fn rgba_embed_writes_soft_mask() {
    let rgba = vec![10, 20, 30, 128, 0, 0, 0, 0, 1, 2, 3, 255, 4, 5, 6, 7];
    let pdf = pdf_embedding_rgba(2, 2, &rgba).unwrap();
    assert!(pdf.windows(6).any(|w| w == b"/SMask"));
    assert!(pdf.windows(10).any(|w| w == b"/DeviceRGB"));
}

#[test]
fn glass_slice_is_narrower_than_the_page_and_scales() {
    let doc = helpers::glass_card();
    let lock = doc.lock().unwrap();
    let page = &lock.geometry.pages[0];
    let ops = page_ops(&doc, 0);
    let slices = slices_for_page(&ops, page.width, page.height).unwrap();
    let slice = slices
        .iter()
        .find(|s| {
            ops.get(s.anchor).is_some_and(|op| {
                matches!(op, PaintOp::BackdropBlur { node_id, .. } if node_id == "card1")
            })
        })
        .expect("card slice");
    let (w2, _, _) = raster_slice(&doc, 0, slice, 2.0).unwrap();
    let (w4, _, _) = raster_slice(&doc, 0, slice, 4.0).unwrap();
    let page_px = (page.width.as_f64_pt() * 4.0).round() as u32;
    assert!(
        w4 + 8 < page_px,
        "slice {w4}px must be smaller than the page {page_px}px"
    );
    let ratio = w4 as f64 / w2 as f64;
    assert!(
        (ratio - 2.0).abs() < 0.05,
        "4x width {w4} vs 2x {w2} ratio {ratio}"
    );
}
