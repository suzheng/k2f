use k2f_core::{looks_like_svg, ImageFit, Rect};
use std::collections::{BTreeMap, HashMap};
use tiny_skia::{FillRule, Mask, Pixmap, PixmapPaint, Transform};

use super::decode::{
    decode_raster, raster_to_pixmap, render_svg_cover_pixmap, render_svg_uniform_pixmap,
    svg_intrinsic_px,
};
use super::letterbox::{cover_src, letterbox_dest};
use crate::error::PaintError;
use crate::geom::{pt_i64_to_px_i32, rect_to_px, rounded_rect_path};

/// Reuses an SVG raster when the same asset is painted more than once at the
/// same box size during one page. Keyed by source and destination pixels, not
/// by the SVG file's intrinsic size.
#[derive(Default)]
pub(crate) struct ImagePaintCache {
    svg: HashMap<SvgRasterKey, (u32, u32, Pixmap)>,
}

#[derive(Hash, Eq, PartialEq)]
struct SvgRasterKey {
    src: String,
    box_w: u32,
    box_h: u32,
    cover: bool,
}

pub fn draw_image(
    pixmap: &mut Pixmap,
    rect: &Rect,
    src: &str,
    images: &BTreeMap<String, Vec<u8>>,
    scale: f32,
    fit: ImageFit,
    corner_radius_pt: Option<i64>,
    cache: &mut ImagePaintCache,
) -> Result<(), PaintError> {
    let bytes =
        lookup_image(images, src).ok_or_else(|| PaintError::MissingImage(src.to_string()))?;
    let box_px = rect_to_px(rect, scale);
    let box_w = (box_px.right - box_px.left).max(1) as u32;
    let box_h = (box_px.bottom - box_px.top).max(1) as u32;
    let clip = rounded_clip(pixmap, &box_px, corner_radius_pt, scale);

    let (dx, dy, src_pm) = if looks_like_svg(bytes) {
        svg_pixmap(bytes, src, box_w, box_h, fit, cache)?
    } else {
        let img = decode_raster(bytes)?;
        match fit {
            ImageFit::Cover => {
                let (sx, sy, sw, sh) =
                    cover_src(img.width(), img.height(), box_w, box_h).ok_or(PaintError::Pixmap)?;
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

fn svg_pixmap(
    bytes: &[u8],
    src: &str,
    box_w: u32,
    box_h: u32,
    fit: ImageFit,
    cache: &mut ImagePaintCache,
) -> Result<(u32, u32, Pixmap), PaintError> {
    let cover = matches!(fit, ImageFit::Cover);
    let key = SvgRasterKey {
        src: src.to_string(),
        box_w,
        box_h,
        cover,
    };
    if let Some((x, y, pm)) = cache.svg.get(&key) {
        return Ok((*x, *y, pm.clone()));
    }
    let (x, y, pm) = if cover {
        (0, 0, render_svg_cover_pixmap(bytes, box_w, box_h)?)
    } else {
        let (iw, ih) = svg_intrinsic_px(bytes)?;
        let (x, y, dw, dh) = letterbox_dest(iw, ih, box_w, box_h).ok_or(PaintError::Pixmap)?;
        (x, y, render_svg_uniform_pixmap(bytes, dw, dh)?)
    };
    cache.svg.insert(key, (x, y, pm.clone()));
    Ok((x, y, pm))
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

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::Pt;

    #[test]
    fn draw_image_scale_4_stroke_is_not_nearest_upscale() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 20 20"><line x1="10" y1="0" x2="10" y2="20" stroke="#000" stroke-width="1"/></svg>"##;
        let mut images = BTreeMap::new();
        images.insert("stroke.svg".into(), svg.to_vec());
        let rect = Rect {
            x: Pt(0),
            y: Pt(0),
            width: Pt(20_000),
            height: Pt(20_000),
        };
        let mut page = Pixmap::new(80, 80).unwrap();
        let mut cache = ImagePaintCache::default();
        draw_image(
            &mut page,
            &rect,
            "stroke.svg",
            &images,
            4.0,
            ImageFit::Contain,
            None,
            &mut cache,
        )
        .unwrap();
        let solid = opaque_run(&page, 200);
        assert!(
            (3..=6).contains(&solid),
            "4× dest stroke should be about 4px of solid ink, got {solid}"
        );
        let low = super::super::decode::decode_svg(svg).unwrap();
        let nearest = raster_to_pixmap(&low, 80, 80).unwrap();
        assert_ne!(
            alpha_row(&page, 40),
            alpha_row(&nearest, 40),
            "draw_image must not be a nearest copy of the intrinsic raster"
        );
    }

    fn alpha_row(pm: &Pixmap, y: u32) -> Vec<u8> {
        let w = pm.width();
        let start = (y * w) as usize;
        pm.data()[start * 4..(start + w as usize) * 4]
            .chunks_exact(4)
            .map(|px| px[3])
            .collect()
    }

    fn opaque_run(pm: &Pixmap, min_alpha: u8) -> u32 {
        let row = alpha_row(pm, pm.height() / 2);
        let mut best = 0u32;
        let mut run = 0u32;
        for a in row {
            if a > min_alpha {
                run += 1;
                best = best.max(run);
            } else {
                run = 0;
            }
        }
        best
    }
}
