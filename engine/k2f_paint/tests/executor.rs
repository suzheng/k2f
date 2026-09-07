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

#[test]
fn load_faces_skips_license_txt() {
    let mut fonts = std::collections::BTreeMap::new();
    fonts.insert(
        "assets/fonts/Roboto-Regular.ttf".into(),
        common::font_bytes(),
    );
    fonts.insert(
        "assets/fonts/licenses/Roboto-Apache.txt".into(),
        b"Apache-2.0".to_vec(),
    );
    k2f_paint::load_faces(&fonts).unwrap();
}

#[test]
fn load_faces_rejects_invalid_ttf() {
    let mut fonts = std::collections::BTreeMap::new();
    fonts.insert("assets/fonts/bad.ttf".into(), b"not-a-font".to_vec());
    let err = k2f_paint::load_faces(&fonts).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("FONT_INVALID"), "{msg}");
    assert!(!msg.contains("UnknownMagic"), "{msg}");
}

#[test]
fn starter_with_license_txt_renders() {
    let mut pkg = k2f_package::load_dir(&common::repo_root().join("skills/k2f/starter")).unwrap();
    let assets: std::collections::HashMap<_, _> = pkg.assets.clone().into_iter().collect();
    let lock = k2f_layout::compile_manifest(
        pkg.engine_manifest(),
        &pkg.theme_json,
        &pkg.fonts,
        if assets.is_empty() {
            None
        } else {
            Some(&assets)
        },
    )
    .unwrap();
    pkg.set_lock(&lock).unwrap();
    let png = k2f_paint::render_lockfile_page_to_png(
        &lock,
        0,
        OFFICIAL_PNG_SCALE,
        &pkg.fonts,
        &pkg.assets,
    )
    .unwrap();
    assert!(png.starts_with(b"\x89PNG"));
}
