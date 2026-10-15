use k2f_core::{looks_like_svg, ImageFit, LockFile, PaintOp, Rect};
use k2f_paint::{cover_src, letterbox_dest, lookup_image, raster_for_embed};
use miniz_oxide::deflate::{compress_to_vec_zlib, CompressionLevel};
use pdf_writer::{Filter, Finish, Name, Pdf, Ref};
use std::collections::BTreeMap;

use crate::coord::pdf_y;
use crate::draw::PageDraw;
use crate::error::PdfError;
use crate::ids::Alloc;

pub struct ImageRes {
    pub id: Ref,
    pub name: String,
    pub w: u32,
    pub h: u32,
}

pub fn embed_rgba(
    pdf: &mut Pdf,
    alloc: &mut Alloc,
    w: u32,
    h: u32,
    rgba: &[u8],
    name: &str,
) -> Result<ImageRes, PdfError> {
    let expect = (w as usize).saturating_mul(h as usize).saturating_mul(4);
    if w == 0 || h == 0 || rgba.len() != expect {
        return Err(PdfError::Write("rgba slice size mismatch".into()));
    }
    let mut rgb = Vec::with_capacity((w as usize) * (h as usize) * 3);
    let mut alpha = Vec::with_capacity((w as usize) * (h as usize));
    for px in rgba.chunks_exact(4) {
        rgb.extend_from_slice(&px[..3]);
        alpha.push(px[3]);
    }
    let level = CompressionLevel::DefaultLevel as u8;
    let mask_id = alloc.bump();
    let id = alloc.bump();
    {
        let encoded = compress_to_vec_zlib(&alpha, level);
        let mut s = pdf.image_xobject(mask_id, &encoded);
        s.filter(Filter::FlateDecode);
        s.width(w as i32);
        s.height(h as i32);
        s.color_space().device_gray();
        s.bits_per_component(8);
        s.finish();
    }
    {
        let encoded = compress_to_vec_zlib(&rgb, level);
        let mut x = pdf.image_xobject(id, &encoded);
        x.filter(Filter::FlateDecode);
        x.width(w as i32);
        x.height(h as i32);
        x.color_space().device_rgb();
        x.bits_per_component(8);
        x.s_mask(mask_id);
        x.finish();
    }
    Ok(ImageRes {
        id,
        name: name.to_string(),
        w,
        h,
    })
}

/// One-page PDF whose only image is `rgba`. Used to assert `/SMask` wiring.
pub fn pdf_embedding_rgba(w: u32, h: u32, rgba: &[u8]) -> Result<Vec<u8>, PdfError> {
    use pdf_writer::Content;

    let mut pdf = Pdf::new();
    pdf.set_version(1, 7);
    let mut alloc = Alloc::new();
    let catalog_id = alloc.bump();
    let pages_id = alloc.bump();
    let page_id = alloc.bump();
    let content_id = alloc.bump();
    let img = embed_rgba(&mut pdf, &mut alloc, w, h, rgba, "Im0")?;
    pdf.catalog(catalog_id).pages(pages_id);
    pdf.pages(pages_id).kids([page_id]).count(1);
    {
        let mut p = pdf.page(page_id);
        p.parent(pages_id);
        p.media_box(pdf_writer::Rect::new(0.0, 0.0, w as f32, h as f32));
        p.contents(content_id);
        p.resources()
            .x_objects()
            .pair(Name(img.name.as_bytes()), img.id);
    }
    let mut content = Content::new();
    content.save_state();
    content.transform([w as f32, 0.0, 0.0, h as f32, 0.0, 0.0]);
    content.x_object(Name(img.name.as_bytes()));
    content.restore_state();
    let compressed = compress_to_vec_zlib(
        content.finish().as_ref(),
        CompressionLevel::DefaultLevel as u8,
    );
    pdf.stream(content_id, &compressed)
        .filter(Filter::FlateDecode);
    Ok(pdf.finish())
}

pub fn embed_images(
    pdf: &mut Pdf,
    alloc: &mut Alloc,
    assets: &BTreeMap<String, Vec<u8>>,
    lock: &LockFile,
) -> Result<BTreeMap<String, ImageRes>, PdfError> {
    let mut out = BTreeMap::new();
    let mut n = 0u32;
    for page in &lock.render_plan.pages {
        for op in &page.ops {
            let PaintOp::DrawImage { src, rect, fit, .. } = op else {
                continue;
            };
            let bytes = lookup_image(assets, src)
                .ok_or_else(|| PdfError::Write(format!("missing image {src}")))?;
            let key = image_key(bytes, src, rect, *fit)?;
            if out.contains_key(&key) {
                continue;
            }
            let img =
                raster_for_embed(bytes, rect, *fit).map_err(|e| PdfError::Write(e.to_string()))?;
            let i = n;
            n += 1;
            let rgb = img.to_rgb8();
            let level = CompressionLevel::DefaultLevel as u8;
            let encoded = compress_to_vec_zlib(rgb.as_raw(), level);
            let id = alloc.bump();
            let mask = img.color().has_alpha().then(|| alloc.bump());
            {
                let mut x = pdf.image_xobject(id, &encoded);
                x.filter(Filter::FlateDecode);
                x.width(img.width() as i32);
                x.height(img.height() as i32);
                x.color_space().device_rgb();
                x.bits_per_component(8);
                if let Some(m) = mask {
                    x.s_mask(m);
                }
                x.finish();
            }
            if let Some(m) = mask {
                let alphas: Vec<u8> = img.to_rgba8().pixels().map(|p| p.0[3]).collect();
                let encoded = compress_to_vec_zlib(&alphas, level);
                let mut s = pdf.image_xobject(m, &encoded);
                s.filter(Filter::FlateDecode);
                s.width(img.width() as i32);
                s.height(img.height() as i32);
                s.color_space().device_gray();
                s.bits_per_component(8);
                s.finish();
            }
            out.insert(
                key,
                ImageRes {
                    id,
                    name: format!("Im{i}"),
                    w: img.width(),
                    h: img.height(),
                },
            );
        }
    }
    Ok(out)
}

/// Bitmaps share one XObject per `src`. SVG is keyed with its export pixel size
/// so two boxes of the same file at different sizes stay separate.
pub(crate) fn image_key(
    bytes: &[u8],
    src: &str,
    rect: &Rect,
    fit: ImageFit,
) -> Result<String, PdfError> {
    if looks_like_svg(bytes) {
        let (w, h) = k2f_paint::svg_export_raster_px(bytes, rect, fit)
            .map_err(|e| PdfError::Write(e.to_string()))?;
        Ok(format!("{src}\u{1}{w}x{h}"))
    } else {
        Ok(src.to_string())
    }
}

pub fn draw_image(
    page: &mut PageDraw,
    rect: &Rect,
    src: &str,
    assets: &BTreeMap<String, Vec<u8>>,
    images: &BTreeMap<String, ImageRes>,
    fit: ImageFit,
    corner_radius_pt: Option<i64>,
) -> Result<(), PdfError> {
    let bytes =
        lookup_image(assets, src).ok_or_else(|| PdfError::Write(format!("missing image {src}")))?;
    let key = image_key(bytes, src, rect, fit)?;
    let img = images
        .get(&key)
        .ok_or_else(|| PdfError::Write(format!("image not embedded {src}")))?;
    page.note_image(rect);
    let box_w = rect.width.as_f64_pt().round().max(1.0) as u32;
    let box_h = rect.height.as_f64_pt().round().max(1.0) as u32;
    let radius = corner_radius_pt.map(|r| r as f64 / 1000.0).unwrap_or(0.0);
    let name = Name(img.name.as_bytes());
    page.content.save_state();
    if radius > 0.0 || matches!(fit, ImageFit::Cover) {
        crate::box_op::rounded_rect(&mut page.content, page.page_h, rect, radius);
        page.content.clip_nonzero();
        page.content.end_path();
    }
    let (dx, dy, dw, dh) = match fit {
        ImageFit::Cover => {
            let (sx, sy, sw, sh) =
                cover_src(img.w, img.h, box_w, box_h).ok_or(PdfError::Write("cover".into()))?;
            let dest_w = (img.w as f64) * (box_w as f64) / (sw as f64);
            let dest_h = (img.h as f64) * (box_h as f64) / (sh as f64);
            let dx = -(sx as f64) * (box_w as f64) / (sw as f64);
            let dy = -(sy as f64) * (box_h as f64) / (sh as f64);
            (dx, dy, dest_w, dest_h)
        }
        ImageFit::Contain => {
            let (dx, dy, dw, dh) = letterbox_dest(img.w, img.h, box_w, box_h)
                .ok_or(PdfError::Write("letterbox".into()))?;
            (dx as f64, dy as f64, dw as f64, dh as f64)
        }
    };
    let x = rect.x.as_f64_pt() + dx;
    let y_top = rect.y.as_f64_pt() + dy;
    let pdf_x = x as f32;
    let pdf_y = pdf_y(page.page_h, y_top + dh);
    page.content
        .transform([dw as f32, 0.0, 0.0, dh as f32, pdf_x, pdf_y]);
    page.content.x_object(name);
    page.content.restore_state();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::{LayoutResult, PageRenderPlan, Pt, RenderPlan};

    #[test]
    fn svg_xobject_is_box_pt_times_four() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="#00f"/></svg>"##;
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(10_000),
            height: Pt(10_000),
        };
        let mut assets = BTreeMap::new();
        assets.insert("assets/images/mark.svg".into(), svg.to_vec());
        let lock = LockFile {
            engine_version: String::new(),
            engine_commit_sha: String::new(),
            content_hash: String::new(),
            appearance_hash: String::new(),
            geometry: LayoutResult { pages: vec![] },
            render_plan: RenderPlan {
                pages: vec![PageRenderPlan {
                    index: 0,
                    ops: vec![PaintOp::DrawImage {
                        node_id: "img".into(),
                        rect: rect.clone(),
                        src: "assets/images/mark.svg".into(),
                        fit: ImageFit::Contain,
                        corner_radius_pt: None,
                    }],
                }],
                ..RenderPlan::default()
            },
        };
        let mut pdf = Pdf::new();
        let mut alloc = Alloc::new();
        let images = embed_images(&mut pdf, &mut alloc, &assets, &lock).unwrap();
        let img = images.values().next().unwrap();
        assert_eq!((img.w, img.h), (40, 40));
    }
}
