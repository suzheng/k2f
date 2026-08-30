use super::coords::PageView;
use super::raster::Raster;

/// Stage color behind a letterboxed page (`sdk/js/viewer/styles.js` `.k2f-root`).
pub const LETTERBOX: u32 = 0xC8C8C8;

/// Paint a pre-scaled official raster into `dest`. Caller fills the letterbox.
pub fn blit_raster(dest: &mut [u32], dest_w: u32, dest_h: u32, src: &Raster, view: &PageView) {
    debug_assert_eq!(dest.len(), dest_w as usize * dest_h as usize);
    let ox = view.origin_x.round() as i32;
    let oy = view.origin_y.round() as i32;
    let dw = dest_w as i32;
    let dh = dest_h as i32;
    for y in 0..src.height as i32 {
        let dy = oy + y;
        if dy < 0 || dy >= dh {
            continue;
        }
        let mut x0 = 0i32;
        let mut x1 = src.width as i32;
        if ox + x0 < 0 {
            x0 = -ox;
        }
        if ox + x1 > dw {
            x1 = dw - ox;
        }
        if x0 >= x1 {
            continue;
        }
        let n = (x1 - x0) as usize;
        let dest_i = dy as usize * dest_w as usize + (ox + x0) as usize;
        let src_i = y as usize * src.width as usize + x0 as usize;
        dest[dest_i..dest_i + n].copy_from_slice(&src.pixels[src_i..src_i + n]);
    }
}

pub fn blend_rect(dest: &mut [u32], dest_w: u32, dest_h: u32, x0: f64, y0: f64, x1: f64, y1: f64) {
    let min_x = x0.min(x1).floor().max(0.0) as u32;
    let min_y = y0.min(y1).floor().max(0.0) as u32;
    let max_x = x0.max(x1).ceil().min(dest_w as f64) as u32;
    let max_y = y0.max(y1).ceil().min(dest_h as f64) as u32;
    if min_x >= max_x || min_y >= max_y {
        return;
    }
    for y in min_y..max_y {
        let row = y * dest_w;
        for x in min_x..max_x {
            let i = (row + x) as usize;
            dest[i] = blend(dest[i], 0x2563EB, 80);
        }
    }
}

fn blend(dst: u32, src: u32, src_a: u8) -> u32 {
    let a = src_a as u32;
    let inv = 255 - a;
    let br = (dst >> 16) & 0xFF;
    let bg = (dst >> 8) & 0xFF;
    let bb = dst & 0xFF;
    let r = (((src >> 16) & 0xFF) * a + br * inv) / 255;
    let g = (((src >> 8) & 0xFF) * a + bg * inv) / 255;
    let b = ((src & 0xFF) * a + bb * inv) / 255;
    (r << 16) | (g << 8) | b
}
