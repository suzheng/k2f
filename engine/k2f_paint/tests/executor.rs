mod common;

use k2f_core::{
    Border, BorderEdge, BorderStyle, BoxDecoration, Fill, FillRef, GeometryNode, GradientStop,
    LayoutResult, LinearGradient, LockFile, Page, PageRenderPlan, PaintOp, Pt, Rect, RenderPlan,
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

#[test]
fn four_edge_border_follows_corner_radius() {
    // Inset box so the stroke is not clipped at the pixmap edge.
    let mut lock = box_lock(BoxDecoration {
        background: Some(FillRef::Inline(Fill::Solid {
            color: "#00FF00".into(),
        })),
        border: Some(Border {
            width_pt: 4000,
            color: "#FF0000".into(),
            edges: vec![
                BorderEdge::Top,
                BorderEdge::Right,
                BorderEdge::Bottom,
                BorderEdge::Left,
            ],
            style: BorderStyle::Solid,
        }),
        corner_radius_pt: Some(20_000),
        ..Default::default()
    });
    lock.geometry.pages[0].root.width = Pt(100_000);
    lock.geometry.pages[0].root.height = Pt(100_000);
    match &mut lock.render_plan.pages[0].ops[0] {
        PaintOp::DrawBox { rect, .. } => {
            *rect = Rect {
                x: Pt(10_000),
                y: Pt(10_000),
                width: Pt(50_000),
                height: Pt(50_000),
            };
        }
        _ => panic!("expected DrawBox"),
    }

    let png = render_lockfile_page_to_png(
        &lock,
        0,
        OFFICIAL_PNG_SCALE,
        &single_font_map(&common::font_bytes()),
        &Default::default(),
    )
    .unwrap();
    let decoded = image::load_from_memory(&png).unwrap().to_rgba8();
    // scale 2: box [20,20]–[120,120], radius 40px. Ear (22,22) is inside the
    // axis-aligned rect but outside the rounded fill/stroke.
    let ear = decoded.get_pixel(22, 22);
    assert!(
        ear[0] > 200 && ear[1] > 200 && ear[2] > 200,
        "square-corner ear must not be the red border, got {ear:?}"
    );
    let mid_top = decoded.get_pixel(70, 20);
    assert!(
        mid_top[0] > 150 && mid_top[1] < 80,
        "mid-edge stroke should stay red, got {mid_top:?}"
    );
    let interior = decoded.get_pixel(70, 70);
    assert!(
        interior[1] > 150 && interior[0] < 80,
        "interior should stay green fill, got {interior:?}"
    );
}

fn gradient_lock(angle_degrees: i64, stops: Vec<GradientStop>) -> LockFile {
    box_lock(BoxDecoration {
        background: Some(FillRef::Inline(Fill::LinearGradient {
            value: LinearGradient::Linear {
                angle_degrees,
                stops,
            },
        })),
        ..Default::default()
    })
}

fn render_box_png(lock: &LockFile) -> image::RgbaImage {
    let png = render_lockfile_page_to_png(
        lock,
        0,
        OFFICIAL_PNG_SCALE,
        &single_font_map(&common::font_bytes()),
        &Default::default(),
    )
    .unwrap();
    image::load_from_memory(&png).unwrap().to_rgba8()
}

#[test]
fn linear_gradient_90_is_top_red_bottom_blue() {
    let lock = gradient_lock(
        90,
        vec![
            GradientStop {
                pos: 0,
                color: "#FF0000".into(),
            },
            GradientStop {
                pos: 1000,
                color: "#0000FF".into(),
            },
        ],
    );
    let decoded = render_box_png(&lock);
    // scale 2: 50pt box at origin → [0,0]–[100,100] px
    let top = decoded.get_pixel(50, 8);
    let bottom = decoded.get_pixel(50, 92);
    assert!(
        top[0] > 180 && top[2] < 80,
        "90° must paint red at the top, got {top:?}"
    );
    assert!(
        bottom[2] > 180 && bottom[0] < 80,
        "90° must paint blue at the bottom, got {bottom:?}"
    );
}

#[test]
fn linear_gradient_bad_stop_color_fails_closed() {
    let lock = gradient_lock(
        90,
        vec![
            GradientStop {
                pos: 0,
                color: "#FF0000".into(),
            },
            GradientStop {
                pos: 1000,
                color: "not-a-hex".into(),
            },
        ],
    );
    let err = render_lockfile_page_to_png(
        &lock,
        0,
        OFFICIAL_PNG_SCALE,
        &single_font_map(&common::font_bytes()),
        &Default::default(),
    )
    .unwrap_err();
    assert!(matches!(err, PaintError::Gradient), "{err}");
}
