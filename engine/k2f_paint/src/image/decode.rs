use image::{DynamicImage, ImageBuffer, ImageFormat, RgbaImage};
use tiny_skia::Pixmap;

use super::letterbox::letterbox_dest;
use crate::error::PaintError;

pub fn decode_raster(bytes: &[u8]) -> Result<DynamicImage, PaintError> {
    match image::guess_format(bytes) {
        Ok(ImageFormat::Png) | Ok(ImageFormat::WebP) | Ok(ImageFormat::Jpeg) => {
            image::load_from_memory(bytes).map_err(|e| PaintError::Image(e.to_string()))
        }
        Ok(other) => Err(PaintError::Image(format!(
            "unsupported image format {other:?}; embed PNG, JPEG, WebP, or SVG"
        ))),
        Err(_) if looks_like_svg(bytes) => decode_svg(bytes),
        Err(e) => Err(PaintError::Image(e.to_string())),
    }
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    let s = String::from_utf8_lossy(bytes);
    let trimmed = s.trim_start();
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("<svg") || (lower.starts_with("<?xml") && lower.contains("<svg"))
}

/// Rasterize SVG with a fixed-version resvg.
/// Built without the `text` feature so host fonts never affect canonical output.
pub fn decode_svg(bytes: &[u8]) -> Result<DynamicImage, PaintError> {
    reject_svg_text(bytes)?;
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(bytes, &opt)
        .map_err(|e| PaintError::Image(format!("SVG parse failed: {e}")))?;
    let size = tree.size();
    let w = size.width().ceil().max(1.0) as u32;
    let h = size.height().ceil().max(1.0) as u32;
    let mut pixmap = Pixmap::new(w, h).ok_or(PaintError::Pixmap)?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    premul_pixmap_to_dynamic(&pixmap)
}

/// `<text>` / `<tspan>` need host fonts. Fail closed instead of dropping glyphs.
pub(crate) fn reject_svg_text(bytes: &[u8]) -> Result<(), PaintError> {
    if svg_contains_text_element(bytes) {
        return Err(PaintError::Image(
            "SVG contains <text> — convert labels to <path> (engine rasterizes SVG without system fonts)"
                .into(),
        ));
    }
    Ok(())
}

fn svg_contains_text_element(bytes: &[u8]) -> bool {
    let lower = String::from_utf8_lossy(bytes).to_ascii_lowercase();
    let mut rest = lower.as_str();
    while let Some(i) = rest.find("<text") {
        let after = rest.get(i + 5..).unwrap_or("");
        if after.starts_with('>')
            || after.starts_with('/')
            || after.starts_with(|c: char| c.is_ascii_whitespace())
        {
            return true;
        }
        rest = after;
    }
    let mut rest = lower.as_str();
    while let Some(i) = rest.find("<tspan") {
        let after = rest.get(i + 6..).unwrap_or("");
        if after.starts_with('>')
            || after.starts_with('/')
            || after.starts_with(|c: char| c.is_ascii_whitespace())
        {
            return true;
        }
        rest = after;
    }
    false
}

fn premul_pixmap_to_dynamic(pixmap: &Pixmap) -> Result<DynamicImage, PaintError> {
    let w = pixmap.width();
    let h = pixmap.height();
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for px in pixmap.data().chunks_exact(4) {
        let a = px[3] as u16;
        if a == 0 {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else if a == 255 {
            rgba.extend_from_slice(px);
        } else {
            // Un-premultiply for image crate Rgba.
            let r = ((px[0] as u16 * 255 + a / 2) / a) as u8;
            let g = ((px[1] as u16 * 255 + a / 2) / a) as u8;
            let b = ((px[2] as u16 * 255 + a / 2) / a) as u8;
            rgba.extend_from_slice(&[r, g, b, a as u8]);
        }
    }
    let img: RgbaImage = ImageBuffer::from_raw(w, h, rgba).ok_or(PaintError::Pixmap)?;
    Ok(DynamicImage::ImageRgba8(img))
}

pub fn raster_to_pixmap(
    img: &DynamicImage,
    dest_w: u32,
    dest_h: u32,
) -> Result<Pixmap, PaintError> {
    let resized = img.resize_exact(dest_w, dest_h, image::imageops::FilterType::Nearest);
    let rgba = resized.to_rgba8();
    let mut premul = rgba.into_raw();
    for px in premul.chunks_exact_mut(4) {
        let a = px[3] as u16;
        px[0] = ((px[0] as u16 * a + 127) / 255) as u8;
        px[1] = ((px[1] as u16 * a + 127) / 255) as u8;
        px[2] = ((px[2] as u16 * a + 127) / 255) as u8;
    }
    Pixmap::from_vec(
        premul,
        tiny_skia::IntSize::from_wh(dest_w, dest_h).ok_or(PaintError::Pixmap)?,
    )
    .ok_or(PaintError::Pixmap)
}

pub fn letterbox_pixmap(
    img: &DynamicImage,
    box_w: u32,
    box_h: u32,
) -> Result<(u32, u32, Pixmap), PaintError> {
    let (x, y, w, h) =
        letterbox_dest(img.width(), img.height(), box_w, box_h).ok_or(PaintError::Pixmap)?;
    let pixmap = raster_to_pixmap(img, w, h)?;
    Ok((x, y, pixmap))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_without_text_decodes() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><rect width="4" height="4" fill="#00f"/></svg>"##;
        decode_svg(svg).unwrap();
    }

    #[test]
    fn svg_with_text_fails_closed() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><text x="1" y="2">A</text></svg>"#;
        let err = decode_svg(svg).unwrap_err().to_string();
        assert!(err.contains("<text>"), "{err}");
        assert!(err.contains("<path>"), "{err}");
    }
}
