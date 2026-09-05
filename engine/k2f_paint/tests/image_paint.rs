mod common;

use k2f_core::{
    GeometryNode, LayoutResult, LockFile, Page, PageRenderPlan, PaintOp, Pt, Rect, RenderPlan,
};
use k2f_paint::{letterbox_dest, render_lockfile_page_to_png, single_font_map, OFFICIAL_PNG_SCALE};
use std::collections::BTreeMap;

fn solid_png(w: u32, h: u32, r: u8, g: u8, b: u8) -> Vec<u8> {
    let mut img = image::RgbImage::new(w, h);
    for p in img.pixels_mut() {
        *p = image::Rgb([r, g, b]);
    }
    let mut buf = Vec::new();
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .unwrap();
    buf
}

#[test]
fn letterbox_math_is_stable() {
    assert_eq!(letterbox_dest(20, 10, 100, 100), Some((0, 25, 100, 50)));
}

#[test]
fn paints_real_png_not_gray_placeholder() {
    let png = solid_png(20, 10, 200, 30, 30);
    let mut images = BTreeMap::new();
    images.insert("assets/images/logo.png".to_string(), png);
    let lock = LockFile {
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
                    id: "img".into(),
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
                ops: vec![PaintOp::DrawImage {
                    node_id: "img".into(),
                    rect: Rect {
                        x: Pt(0),
                        y: Pt(0),
                        width: Pt(100000),
                        height: Pt(100000),
                    },
                    src: "assets/images/logo.png".into(),
                }],
            }],
        },
    };
    let out = render_lockfile_page_to_png(
        &lock,
        0,
        OFFICIAL_PNG_SCALE,
        &single_font_map(&common::font_bytes()),
        &images,
    )
    .unwrap();
    let decoded = image::load_from_memory(&out).unwrap().to_rgba8();
    let cx = decoded.width() / 2;
    let cy = decoded.height() / 2;
    let p = decoded.get_pixel(cx, cy);
    assert!(
        p[0] > 150 && p[1] < 80,
        "center should be the red logo, got {p:?}"
    );
    let corner = decoded.get_pixel(2, 2);
    assert!(
        corner[0] > 200 && corner[1] > 200 && corner[2] > 200,
        "letterbox bars should stay near white, got {corner:?}"
    );
}

fn solid_jpeg(w: u32, h: u32, r: u8, g: u8, b: u8) -> Vec<u8> {
    let mut img = image::RgbImage::new(w, h);
    for p in img.pixels_mut() {
        *p = image::Rgb([r, g, b]);
    }
    let mut buf = Vec::new();
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Jpeg)
        .unwrap();
    buf
}

#[test]
fn paints_real_jpeg_not_gray_placeholder() {
    let jpeg = solid_jpeg(20, 10, 30, 30, 200);
    let mut images = BTreeMap::new();
    images.insert("assets/images/logo.jpg".to_string(), jpeg);
    let lock = LockFile {
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
                    id: "img".into(),
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
                ops: vec![PaintOp::DrawImage {
                    node_id: "img".into(),
                    rect: Rect {
                        x: Pt(0),
                        y: Pt(0),
                        width: Pt(100000),
                        height: Pt(100000),
                    },
                    src: "assets/images/logo.jpg".into(),
                }],
            }],
        },
    };
    let out = render_lockfile_page_to_png(
        &lock,
        0,
        OFFICIAL_PNG_SCALE,
        &single_font_map(&common::font_bytes()),
        &images,
    )
    .unwrap();
    let decoded = image::load_from_memory(&out).unwrap().to_rgba8();
    let p = decoded.get_pixel(decoded.width() / 2, decoded.height() / 2);
    assert!(
        p[2] > 150 && p[0] < 80,
        "center should be the blue logo, got {p:?}"
    );
}

fn solid_webp(w: u32, h: u32, r: u8, g: u8, b: u8) -> Vec<u8> {
    let mut img = image::RgbImage::new(w, h);
    for p in img.pixels_mut() {
        *p = image::Rgb([r, g, b]);
    }
    let mut buf = Vec::new();
    image::DynamicImage::ImageRgb8(img)
        .write_to(
            &mut std::io::Cursor::new(&mut buf),
            image::ImageFormat::WebP,
        )
        .unwrap();
    buf
}

#[test]
fn paints_real_webp_not_gray_placeholder() {
    let webp = solid_webp(16, 16, 20, 160, 40);
    let mut images = BTreeMap::new();
    images.insert("assets/images/mark.webp".to_string(), webp);
    let lock = LockFile {
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
                    id: "img".into(),
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
                ops: vec![PaintOp::DrawImage {
                    node_id: "img".into(),
                    rect: Rect {
                        x: Pt(0),
                        y: Pt(0),
                        width: Pt(100000),
                        height: Pt(100000),
                    },
                    src: "assets/images/mark.webp".into(),
                }],
            }],
        },
    };
    let out = render_lockfile_page_to_png(
        &lock,
        0,
        OFFICIAL_PNG_SCALE,
        &single_font_map(&common::font_bytes()),
        &images,
    )
    .unwrap();
    let decoded = image::load_from_memory(&out).unwrap().to_rgba8();
    let p = decoded.get_pixel(decoded.width() / 2, decoded.height() / 2);
    assert!(
        p[1] > 120 && p[0] < 80,
        "center should be the green mark, got {p:?}"
    );
}
