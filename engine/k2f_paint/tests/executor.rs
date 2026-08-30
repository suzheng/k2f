mod common;

use k2f_core::{
    BoxDecoration, FillRef, GeometryNode, LayoutResult, LockFile, Page, PageRenderPlan, PaintOp,
    Pt, Rect, RenderPlan,
};
use k2f_paint::{render_lockfile_page_to_png, single_font_map, PaintError, OFFICIAL_PNG_SCALE};

fn box_lock(decoration: BoxDecoration) -> LockFile {
    LockFile {
        engine_version: "0".into(),
        engine_commit_sha: "0".into(),
        content_hash: "0".into(),
        appearance_hash: "0".into(),
        geometry: LayoutResult {
            pages: vec![Page {
                index: 0,
                width: Pt(100000),
                height: Pt(100000),
                root: GeometryNode {
                    id: "box".into(),
                    x: Pt(0),
                    y: Pt(0),
                    width: Pt(100000),
                    height: Pt(100000),
                    glyphs: vec![],
                    text_runs: vec![],
                    fill_rects: vec![],
                    children: vec![],
                },
            }],
        },
        render_plan: RenderPlan {
            compositing: Default::default(),
            pages: vec![PageRenderPlan {
                index: 0,
                ops: vec![PaintOp::DrawBox {
                    node_id: "box".into(),
                    rect: Rect {
                        x: Pt(0),
                        y: Pt(0),
                        width: Pt(50000),
                        height: Pt(50000),
                    },
                    decoration,
                }],
            }],
        },
    }
}

#[test]
fn unresolved_fill_ref_is_hard_failure() {
    let lock = box_lock(BoxDecoration {
        background: Some(FillRef::Ref("missing_surface".into())),
        ..Default::default()
    });
    let err = render_lockfile_page_to_png(
        &lock,
        0,
        OFFICIAL_PNG_SCALE,
        &single_font_map(&common::font_bytes()),
        &Default::default(),
    )
    .unwrap_err();
    assert!(
        matches!(err, PaintError::UnresolvedRef(ref name) if name == "missing_surface"),
        "{err}"
    );
}

#[test]
fn non_positive_scale_is_hard_failure() {
    let lock = box_lock(BoxDecoration::default());
    let fonts = single_font_map(&common::font_bytes());
    let images = Default::default();
    assert!(matches!(
        render_lockfile_page_to_png(&lock, 0, 0.0, &fonts, &images),
        Err(PaintError::InvalidScale)
    ));
    assert!(matches!(
        render_lockfile_page_to_png(&lock, 0, -1.0, &fonts, &images),
        Err(PaintError::InvalidScale)
    ));
}
