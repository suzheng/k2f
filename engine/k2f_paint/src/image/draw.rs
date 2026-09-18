use k2f_core::{ImageFit, Rect};
use std::collections::BTreeMap;
use tiny_skia::{FillRule, Mask, Pixmap, PixmapPaint, Transform};

use super::decode::{decode_raster, raster_to_pixmap};
use super::letterbox::{cover_src, letterbox_dest};
use crate::error::PaintError;
use crate::geom::{pt_i64_to_px_i32, rect_to_px, rounded_rect_path};

pub fn draw_image(
    pixmap: &mut Pixmap,
    rect: &Rect,
    src: &str,
    images: &BTreeMap<String, Vec<u8>>,
    scale: f32,
    fit: ImageFit,
    corner_radius_pt: Option<i64>,
) -> Result<(), PaintError> {
    let bytes =
        lookup_image(images, src).ok_or_else(|| PaintError::MissingImage(src.to_string()))?;
    let img = decode_raster(bytes)?;
    let box_px = rect_to_px(rect, scale);
    let box_w = (box_px.right - box_px.left).max(1) as u32;
    let box_h = (box_px.bottom - box_px.top).max(1) as u32;
    let clip = rounded_clip(pixmap, &box_px, corner_radius_pt, scale);

    let (dx, dy, src_pm) = match fit {
        ImageFit::Cover => {
            let (sx, sy, sw, sh) = cover_src(img.width(), img.height(), box_w, box_h)
                .ok_or(PaintError::Pixmap)?;
            let cropped = img.crop_imm(sx, sy, sw, sh);
            let pm = raster_to_pixmap(&cropped, box_w, box_h)?;
            (0u32, 0u32, pm)
        }
        ImageFit::Contain => {
            let (x, y, w, h) = letterbox_dest(img.width(), img.height(), box_w, box_h)
                .ok_or(PaintError::Pixmap)?;
            let pm = raster_to_pixmap(&img, w, h)?;
            (x, y, pm)
        }
    };
    let x = box_px.left + dx as i32;
    let y = box_px.top + dy as i32;
    pixmap.draw_pixmap(
        x,
        y,
        src_pm.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        clip.as_ref(),
    );
    Ok(())
}

fn rounded_clip(
    pixmap: &Pixmap,
    box_px: &crate::geom::RectPx,
    corner_radius_pt: Option<i64>,
    scale: f32,
) -> Option<Mask> {
    let radius = corner_radius_pt.filter(|r| *r > 0)?;
    let x = box_px.left as f32;
    let y = box_px.top as f32;
    let w = (box_px.right - box_px.left).max(1) as f32;
    let h = (box_px.bottom - box_px.top).max(1) as f32;
    let r = pt_i64_to_px_i32(radius, scale).max(0) as f32;
    let path = rounded_rect_path(x, y, w, h, r)?;
    let mut mask = Mask::new(pixmap.width(), pixmap.height())?;
    mask.fill_path(&path, FillRule::Winding, true, Transform::identity());
    Some(mask)
}

pub fn lookup_image<'a>(
    images: &'a BTreeMap<String, Vec<u8>>,
    src: &'a str,
) -> Option<&'a Vec<u8>> {
    let stripped = src.strip_prefix("asset://").unwrap_or(src);
    let owned = [
        format!("assets/images/{stripped}"),
        format!("assets/images/{stripped}.png"),
        format!("assets/images/{stripped}.webp"),
        format!("assets/images/{stripped}.jpg"),
        format!("assets/images/{stripped}.jpeg"),
        format!("assets/images/{stripped}.svg"),
        format!("{stripped}.png"),
        format!("{stripped}.jpg"),
        format!("{stripped}.jpeg"),
        format!("{stripped}.svg"),
    ];
    for key in std::iter::once(src)
        .chain(std::iter::once(stripped))
        .chain(owned.iter().map(String::as_str))
    {
        if let Some(b) = images.get(key) {
            return Some(b);
        }
    }
    None
}
