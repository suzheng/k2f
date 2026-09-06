use k2f_core::Rect;
use std::collections::BTreeMap;
use tiny_skia::{Pixmap, PixmapPaint, Transform};

use super::decode::{decode_raster, letterbox_pixmap};
use crate::error::PaintError;
use crate::geom::rect_to_px;

pub fn draw_image(
    pixmap: &mut Pixmap,
    rect: &Rect,
    src: &str,
    images: &BTreeMap<String, Vec<u8>>,
    scale: f32,
) -> Result<(), PaintError> {
    let bytes =
        lookup_image(images, src).ok_or_else(|| PaintError::MissingImage(src.to_string()))?;
    let img = decode_raster(bytes)?;
    let box_px = rect_to_px(rect, scale);
    let box_w = (box_px.right - box_px.left).max(1) as u32;
    let box_h = (box_px.bottom - box_px.top).max(1) as u32;
    let (dx, dy, src_pm) = letterbox_pixmap(&img, box_w, box_h)?;
    let x = box_px.left + dx as i32;
    let y = box_px.top + dy as i32;
    pixmap.draw_pixmap(
        x,
        y,
        src_pm.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
    Ok(())
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
