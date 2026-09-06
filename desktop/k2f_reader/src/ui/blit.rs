use super::coords::PageView;
use super::raster::Raster;

/// Stage color behind a letterboxed page (desktop canvas, slightly softer than web).
pub const LETTERBOX: u32 = 0xFBFBFD;

/// True when the page dest rect intersects the window (including the 2px drop shadow).
pub fn page_in_window(view: &PageView, dest_w: u32, dest_h: u32) -> bool {
    let (sw, sh) = view.scaled_size();
    let ox = view.origin_x.round() as i32;
    let oy = view.origin_y.round() as i32;
    rect_intersects(ox, oy, sw, sh, dest_w, dest_h)
        || rect_intersects(
            ox.saturating_add(2),
            oy.saturating_add(2),
            sw,
            sh,
            dest_w,
            dest_h,
        )
}

fn rect_intersects(ox: i32, oy: i32, sw: u32, sh: u32, dest_w: u32, dest_h: u32) -> bool {
    ox < dest_w as i32
        && oy < dest_h as i32
        && ox.saturating_add(sw as i32) > 0
        && oy.saturating_add(sh as i32) > 0
}

/// Paint the official raster into `dest`, scaled to `view` (blit-time, viewport only).
/// `zoom == 1` is a 1:1 copy so tests can match lock PNG pixels.
pub fn blit_raster(dest: &mut [u32], dest_w: u32, dest_h: u32, src: &Raster, view: &PageView) {
    debug_assert_eq!(dest.len(), dest_w as usize * dest_h as usize);
    if src.width == 0 || src.height == 0 || !page_in_window(view, dest_w, dest_h) {
        return;
    }
    let (sw, sh) = view.scaled_size();
    if sw == src.width && sh == src.height {
        blit_1to1(dest, dest_w, dest_h, src, view);
    } else {
        blit_bilinear(dest, dest_w, dest_h, src, view, sw, sh);
    }
}

fn blit_1to1(dest: &mut [u32], dest_w: u32, dest_h: u32, src: &Raster, view: &PageView) {
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

fn blit_bilinear(
    dest: &mut [u32],
    dest_w: u32,
    dest_h: u32,
    src: &Raster,
    view: &PageView,
    sw: u32,
    sh: u32,
) {
    let ox = view.origin_x.round() as i32;
    let oy = view.origin_y.round() as i32;
    let x0 = ox.max(0);
    let y0 = oy.max(0);
    let x1 = (ox + sw as i32).min(dest_w as i32);
    let y1 = (oy + sh as i32).min(dest_h as i32);
    if x0 >= x1 || y0 >= y1 {
        return;
    }
    let scale_x = src.width as f32 / sw as f32;
    let scale_y = src.height as f32 / sh as f32;
    for dy in y0..y1 {
        let sy = ((dy - oy) as f32 + 0.5) * scale_y - 0.5;
        let row = dy as usize * dest_w as usize;
        for dx in x0..x1 {
            let sx = ((dx - ox) as f32 + 0.5) * scale_x - 0.5;
            dest[row + dx as usize] = sample_bilinear(src, sx, sy);
        }
    }
}

fn sample_bilinear(src: &Raster, x: f32, y: f32) -> u32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let fx = (x - x0 as f32).clamp(0.0, 1.0);
    let fy = (y - y0 as f32).clamp(0.0, 1.0);
    let c00 = pixel_clamped(src, x0, y0);
    let c10 = pixel_clamped(src, x0 + 1, y0);
    let c01 = pixel_clamped(src, x0, y0 + 1);
    let c11 = pixel_clamped(src, x0 + 1, y0 + 1);
    lerp_xrgb(lerp_xrgb(c00, c10, fx), lerp_xrgb(c01, c11, fx), fy)
}

fn pixel_clamped(src: &Raster, x: i32, y: i32) -> u32 {
    let max_x = src.width.saturating_sub(1) as i32;
    let max_y = src.height.saturating_sub(1) as i32;
    let x = x.clamp(0, max_x) as u32;
    let y = y.clamp(0, max_y) as u32;
    src.pixels[(y as usize) * (src.width as usize) + x as usize]
}

fn lerp_xrgb(a: u32, b: u32, t: f32) -> u32 {
    let r = lerp_chan((a >> 16) & 0xFF, (b >> 16) & 0xFF, t);
    let g = lerp_chan((a >> 8) & 0xFF, (b >> 8) & 0xFF, t);
    let bch = lerp_chan(a & 0xFF, b & 0xFF, t);
    (r << 16) | (g << 8) | bch
}

fn lerp_chan(a: u32, b: u32, t: f32) -> u32 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u32
}

/// Web `.k2f-text-layer span::selection { background: rgba(0, 122, 255, 0.28) }`.
pub const SELECTION_FILL: u32 = 0x007AFF;
pub const SELECTION_ALPHA: u8 = 71;

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
            dest[i] = blend(dest[i], SELECTION_FILL, SELECTION_ALPHA);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::coords::PageView;

    fn view(ox: f64, oy: f64, zoom: f32, png_w: u32, png_h: u32) -> PageView {
        PageView {
            origin_x: ox,
            origin_y: oy,
            zoom,
            png_w,
            png_h,
        }
    }

    #[test]
    fn bilinear_upscale_keeps_solid_fill() {
        let src = Raster {
            width: 2,
            height: 2,
            pixels: vec![0xCC3333; 4],
        };
        let mut dest = vec![LETTERBOX; 16];
        blit_raster(&mut dest, 4, 4, &src, &view(0.0, 0.0, 2.0, 2, 2));
        assert!(
            dest.iter().all(|&p| p == 0xCC3333),
            "solid source must stay solid after blit-time scale, got {dest:?}"
        );
    }

    #[test]
    fn zoom_one_is_memcpy() {
        let src = Raster {
            width: 2,
            height: 2,
            pixels: vec![0x010101, 0x020202, 0x030303, 0x040404],
        };
        let mut dest = vec![0; 4];
        blit_raster(&mut dest, 2, 2, &src, &view(0.0, 0.0, 1.0, 2, 2));
        assert_eq!(dest, src.pixels);
    }

    #[test]
    fn offscreen_page_is_skipped() {
        let src = Raster {
            width: 2,
            height: 2,
            pixels: vec![0xFFFFFF; 4],
        };
        let mut dest = vec![LETTERBOX; 100];
        blit_raster(&mut dest, 10, 10, &src, &view(0.0, 80.0, 1.0, 2, 2));
        assert!(
            dest.iter().all(|&p| p == LETTERBOX),
            "fully off-screen pages must not touch the window"
        );
    }
}
