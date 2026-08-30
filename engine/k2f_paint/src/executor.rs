use k2f_core::{GeometryNode, LockFile, PaintOp};
use std::collections::{BTreeMap, HashMap};
use tiny_skia::{Color, Pixmap};
use ttf_parser::Face;

use crate::blur::apply_backdrop_blur;
use crate::box_op::draw_box;
use crate::error::PaintError;
use crate::geom::pt_to_px_u32;
use crate::image::draw_image;
use crate::placeholders::draw_table_placeholder;
use crate::text::draw_text;

pub fn single_font_map(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut m = BTreeMap::new();
    m.insert("default".to_string(), bytes.to_vec());
    m
}

pub fn load_faces<'a>(
    fonts: &'a BTreeMap<String, Vec<u8>>,
) -> Result<HashMap<String, Face<'a>>, PaintError> {
    if fonts.is_empty() {
        return Err(PaintError::Font("package has no embedded font".into()));
    }
    let mut faces: HashMap<String, Face<'_>> = HashMap::new();
    for (name, bytes) in fonts {
        let face = Face::parse(bytes, 0).map_err(|e| PaintError::Font(format!("{name}: {e:?}")))?;
        faces.insert(name.clone(), face);
        if let Some(stem) = std::path::Path::new(name).file_stem() {
            let stem = stem.to_string_lossy().into_owned();
            if !faces.contains_key(&stem) {
                let face = Face::parse(bytes, 0)
                    .map_err(|e| PaintError::Font(format!("{stem}: {e:?}")))?;
                faces.insert(stem, face);
            }
        }
    }
    if !faces.contains_key("default") {
        let (name, bytes) = fonts.iter().next().unwrap();
        let face = Face::parse(bytes, 0).map_err(|e| PaintError::Font(format!("{name}: {e:?}")))?;
        faces.insert("default".into(), face);
    }
    Ok(faces)
}

pub fn render_lockfile_page_rgb(
    lock: &LockFile,
    page_idx: usize,
    scale: f32,
    fonts: &BTreeMap<String, Vec<u8>>,
    images: &BTreeMap<String, Vec<u8>>,
) -> Result<(u32, u32, Vec<u8>), PaintError> {
    if lock.has_unknown_paint_ops() {
        return Err(PaintError::UnknownOp);
    }
    if !scale.is_finite() || scale <= 0.0 {
        return Err(PaintError::InvalidScale);
    }
    let faces = load_faces(fonts)?;
    let pixmap = render_lockfile_page_to_pixmap(lock, page_idx, scale, &faces, images)?;
    Ok((
        pixmap.width(),
        pixmap.height(),
        crate::rgb::pixmap_to_rgb8(&pixmap),
    ))
}

pub fn render_lockfile_page_to_png(
    lock: &LockFile,
    page_idx: usize,
    scale: f32,
    fonts: &BTreeMap<String, Vec<u8>>,
    images: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<u8>, PaintError> {
    let faces = load_faces(fonts)?;
    let pixmap = render_lockfile_page_to_pixmap(lock, page_idx, scale, &faces, images)?;
    pixmap
        .encode_png()
        .map_err(|e| PaintError::Png(e.to_string()))
}

fn render_lockfile_page_to_pixmap(
    lock: &LockFile,
    page_idx: usize,
    scale: f32,
    faces: &HashMap<String, Face<'_>>,
    images: &BTreeMap<String, Vec<u8>>,
) -> Result<Pixmap, PaintError> {
    let page = lock
        .geometry
        .pages
        .get(page_idx)
        .ok_or(PaintError::PageOutOfRange(page_idx))?;
    let plan = lock
        .render_plan
        .pages
        .iter()
        .find(|p| p.index == page.index)
        .ok_or(PaintError::MissingRenderPlan(page.index))?;

    let w_px = pt_to_px_u32(page.width, scale).max(1);
    let h_px = pt_to_px_u32(page.height, scale).max(1);
    let mut pixmap = Pixmap::new(w_px, h_px).ok_or(PaintError::Pixmap)?;
    pixmap.fill(Color::from_rgba8(255, 255, 255, 255));

    let mut geo_by_id: HashMap<String, Vec<&GeometryNode>> = HashMap::new();
    crate::geo_index::index_geometry_multi(&page.root, &mut geo_by_id);

    for op in &plan.ops {
        execute_op(&mut pixmap, faces, images, &geo_by_id, op, scale)?;
    }

    Ok(pixmap)
}

fn execute_op(
    pixmap: &mut Pixmap,
    faces: &HashMap<String, Face<'_>>,
    images: &BTreeMap<String, Vec<u8>>,
    geo_by_id: &HashMap<String, Vec<&GeometryNode>>,
    op: &PaintOp,
    scale: f32,
) -> Result<(), PaintError> {
    match op {
        PaintOp::BackdropBlur {
            rect,
            radius_pt,
            corner_radius_pt,
            ..
        } => {
            apply_backdrop_blur(pixmap, rect, *radius_pt, *corner_radius_pt, scale);
            Ok(())
        }
        PaintOp::DrawBox {
            rect, decoration, ..
        } => draw_box(pixmap, rect, decoration, scale),
        PaintOp::DrawText {
            node_id,
            rect,
            runs,
        } => draw_text(pixmap, faces, geo_by_id, node_id, rect, runs, scale),
        PaintOp::DrawImage { rect, src, .. } => draw_image(pixmap, rect, src, images, scale),
        PaintOp::DrawTableReference { rect, .. } => {
            draw_table_placeholder(pixmap, rect, scale);
            Ok(())
        }
        PaintOp::Unknown => Err(PaintError::UnknownOp),
    }
}
