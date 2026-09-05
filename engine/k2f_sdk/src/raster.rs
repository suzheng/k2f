use crate::error::{AgentError, IMAGE_SIZE};
use image::{GenericImageView, ImageFormat};

pub fn raster_size(bytes: &[u8]) -> Result<(u32, u32, &'static str), AgentError> {
    match image::guess_format(bytes) {
        Ok(ImageFormat::Png) => size_from_raster(bytes, "png"),
        Ok(ImageFormat::WebP) => size_from_raster(bytes, "webp"),
        Ok(ImageFormat::Jpeg) => size_from_raster(bytes, "jpg"),
        Ok(other) => Err(AgentError::new(
            IMAGE_SIZE,
            format!("unsupported image format {other:?}; embed PNG, JPEG, WebP, or SVG"),
        )),
        Err(_) if looks_like_svg(bytes) => svg_size(bytes),
        Err(e) => Err(AgentError::new(
            IMAGE_SIZE,
            format!("cannot read image: {e}"),
        )),
    }
}

fn size_from_raster(
    bytes: &[u8],
    ext: &'static str,
) -> Result<(u32, u32, &'static str), AgentError> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| AgentError::new(IMAGE_SIZE, format!("cannot decode image: {e}")))?;
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return Err(AgentError::new(
            IMAGE_SIZE,
            "image pixel size must be positive",
        ));
    }
    Ok((w, h, ext))
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    let s = String::from_utf8_lossy(bytes);
    let trimmed = s.trim_start();
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("<svg") || (lower.starts_with("<?xml") && lower.contains("<svg"))
}

fn svg_size(bytes: &[u8]) -> Result<(u32, u32, &'static str), AgentError> {
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(bytes, &opt)
        .map_err(|e| AgentError::new(IMAGE_SIZE, format!("SVG parse failed: {e}")))?;
    let size = tree.size();
    let w = size.width().ceil().max(1.0) as u32;
    let h = size.height().ceil().max(1.0) as u32;
    Ok((w, h, "svg"))
}

pub fn asset_path(id: &str, ext: &str) -> String {
    let stem: String = id.chars().map(|c| if c == '.' { '_' } else { c }).collect();
    format!("assets/images/{stem}.{ext}")
}
