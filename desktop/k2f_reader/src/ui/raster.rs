use image::RgbaImage;

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

    pub fn get(&self, x: u32, y: u32) -> Option<u32> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.pixels[(y as usize) * (self.width as usize) + x as usize])
    }
}

pub fn decode_png(bytes: &[u8]) -> anyhow::Result<RgbaImage> {
    Ok(image::load_from_memory(bytes)?.to_rgba8())
}

pub fn xrgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}
