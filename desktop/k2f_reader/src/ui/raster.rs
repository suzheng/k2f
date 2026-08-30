use super::coords::scaled_png_size;
use image::{imageops::FilterType, Rgba, RgbaImage};

#[derive(Debug, Clone)]
pub struct Raster {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

impl Raster {
    pub fn from_rgba(img: &RgbaImage) -> Self {
        let raw = img.as_raw();
        let mut pixels = Vec::with_capacity(raw.len() / 4);
        for px in raw.chunks_exact(4) {
            pixels.push(xrgb(px[0], px[1], px[2]));
        }
        Self {
            width: img.width(),
            height: img.height(),
            pixels,
        }
    }
}

pub fn decode_png(bytes: &[u8]) -> anyhow::Result<RgbaImage> {
    Ok(image::load_from_memory(bytes)?.to_rgba8())
}

pub fn scale_rgba(page: &RgbaImage, zoom: f32) -> Raster {
    let (sw, sh) = scaled_png_size(page.width(), page.height(), zoom);
    if sw == page.width() && sh == page.height() {
        Raster::from_rgba(page)
    } else {
        let img = image::imageops::resize(page, sw, sh, FilterType::Triangle);
        Raster::from_rgba(&img)
    }
}

pub fn xrgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn rgba_xrgb(p: Rgba<u8>) -> u32 {
    xrgb(p[0], p[1], p[2])
}
