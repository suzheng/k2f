use k2f_core::Rect;
use k2f_paint::{decode_raster, letterbox_dest, lookup_image};
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

pub fn embed_rgb(
    pdf: &mut Pdf,
    alloc: &mut Alloc,
    w: u32,
    h: u32,
    rgb: &[u8],
    name: &str,
) -> ImageRes {
    let level = CompressionLevel::DefaultLevel as u8;
    let encoded = compress_to_vec_zlib(rgb, level);
    let id = alloc.bump();
    {
        let mut x = pdf.image_xobject(id, &encoded);
        x.filter(Filter::FlateDecode);
        x.width(w as i32);
        x.height(h as i32);
        x.color_space().device_rgb();
        x.bits_per_component(8);
        x.finish();
    }
    ImageRes {
        id,
        name: name.to_string(),
        w,
        h,
    }
}

pub fn embed_images(
    pdf: &mut Pdf,
    alloc: &mut Alloc,
    assets: &BTreeMap<String, Vec<u8>>,
    srcs: &[String],
) -> Result<BTreeMap<String, ImageRes>, PdfError> {
    let mut out = BTreeMap::new();
    for (i, src) in srcs.iter().enumerate() {
        if out.contains_key(src) {
            continue;
        }
        let bytes = lookup_image(assets, src)
            .ok_or_else(|| PdfError::Write(format!("missing image {src}")))?;
        let img = decode_raster(bytes)?;
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
            src.clone(),
            ImageRes {
                id,
                name: format!("Im{i}"),
                w: img.width(),
                h: img.height(),
            },
        );
    }
    Ok(out)
}

pub fn draw_image(
    page: &mut PageDraw,
    rect: &Rect,
    src: &str,
    images: &BTreeMap<String, ImageRes>,
) -> Result<(), PdfError> {
    let img = images
        .get(src)
        .ok_or_else(|| PdfError::Write(format!("image not embedded {src}")))?;
    page.note_image(rect);
    let box_w = rect.width.as_f64_pt().round().max(1.0) as u32;
    let box_h = rect.height.as_f64_pt().round().max(1.0) as u32;
    let (dx, dy, dw, dh) =
        letterbox_dest(img.w, img.h, box_w, box_h).ok_or(PdfError::Write("letterbox".into()))?;
    let x = rect.x.as_f64_pt() + dx as f64;
    let y_top = rect.y.as_f64_pt() + dy as f64;
    let pdf_x = x as f32;
    let pdf_y = pdf_y(page.page_h, y_top + dh as f64);
    let name = Name(img.name.as_bytes());
    page.content.save_state();
    page.content
        .transform([dw as f32, 0.0, 0.0, dh as f32, pdf_x, pdf_y]);
    page.content.x_object(name);
    page.content.restore_state();
    Ok(())
}

pub fn collect_image_srcs(lock: &k2f_core::LockFile) -> Vec<String> {
    let mut srcs = Vec::new();
    for page in &lock.render_plan.pages {
        for op in &page.ops {
            if let k2f_core::PaintOp::DrawImage { src, .. } = op {
                if !srcs.contains(src) {
                    srcs.push(src.clone());
                }
            }
        }
    }
    srcs
}
