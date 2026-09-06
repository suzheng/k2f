//! CPU primitives for the desktop chrome (no GPU / CSS).

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(self, px: f64, py: f64) -> bool {
        if self.w == 0 || self.h == 0 {
            return false;
        }
        let x1 = f64::from(self.x);
        let y1 = f64::from(self.y);
        let x2 = x1 + f64::from(self.w);
        let y2 = y1 + f64::from(self.h);
        px >= x1 && px < x2 && py >= y1 && py < y2
    }

    #[allow(dead_code)]
    pub fn right(self) -> i32 {
        self.x.saturating_add(self.w as i32)
    }
}

pub fn fill_rect(buf: &mut [u32], width: u32, height: u32, rect: Rect, color: u32) {
    if rect.w == 0 || rect.h == 0 {
        return;
    }
    let x0 = rect.x.max(0) as u32;
    let y0 = rect.y.max(0) as u32;
    let x1 = (rect.x.saturating_add(rect.w as i32)).max(0) as u32;
    let y1 = (rect.y.saturating_add(rect.h as i32)).max(0) as u32;
    let x1 = x1.min(width);
    let y1 = y1.min(height);
    if x0 >= x1 || y0 >= y1 {
        return;
    }
    for y in y0..y1 {
        let row = y * width;
        for x in x0..x1 {
            buf[(row + x) as usize] = color;
        }
    }
}

pub fn hline(buf: &mut [u32], width: u32, height: u32, y: i32, x0: u32, x1: u32, color: u32) {
    if y < 0 || y >= height as i32 {
        return;
    }
    let y = y as u32;
    let lo = x0.min(width);
    let hi = x1.min(width);
    if lo >= hi {
        return;
    }
    let row = y * width;
    for x in lo..hi {
        buf[(row + x) as usize] = color;
    }
}

pub fn fill_round_rect(
    buf: &mut [u32],
    width: u32,
    height: u32,
    rect: Rect,
    radius: u32,
    color: u32,
) {
    if rect.w == 0 || rect.h == 0 {
        return;
    }
    let r = radius.min(rect.w / 2).min(rect.h / 2) as i32;
    let x0 = rect.x;
    let y0 = rect.y;
    let x1 = rect.x.saturating_add(rect.w as i32);
    let y1 = rect.y.saturating_add(rect.h as i32);
    let dw = width as i32;
    let dh = height as i32;
    let r2 = r * r;
    for y in y0.max(0)..y1.min(dh) {
        let row = y as u32 * width;
        for x in x0.max(0)..x1.min(dw) {
            if !in_round_rect(x, y, x0, y0, x1, y1, r, r2) {
                continue;
            }
            buf[(row + x as u32) as usize] = color;
        }
    }
}

pub fn blend_over(dst: u32, src: u32, src_a: u8) -> u32 {
    if src_a == 0 {
        return dst;
    }
    if src_a == 255 {
        return src;
    }
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

pub fn blend_pixel(
    buf: &mut [u32],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    color: u32,
    alpha: u8,
) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 || alpha == 0 {
        return;
    }
    let i = (y as u32 * width + x as u32) as usize;
    buf[i] = blend_over(buf[i], color, alpha);
}

pub fn stroke_line(
    buf: &mut [u32],
    width: u32,
    height: u32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    thickness: f32,
    color: u32,
) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let nx = dx / len;
    let ny = dy / len;
    let half = thickness * 0.5 + 0.75;
    let min_x = x0.min(x1) - half;
    let max_x = x0.max(x1) + half;
    let min_y = y0.min(y1) - half;
    let max_y = y0.max(y1) + half;
    let x_start = min_x.floor().max(0.0) as i32;
    let y_start = min_y.floor().max(0.0) as i32;
    let x_end = max_x.ceil().min(width as f32) as i32;
    let y_end = max_y.ceil().min(height as f32) as i32;
    let radius = thickness * 0.5;
    for y in y_start..y_end {
        for x in x_start..x_end {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let t = ((px - x0) * nx + (py - y0) * ny).clamp(0.0, len);
            let cx = x0 + nx * t;
            let cy = y0 + ny * t;
            let d = (px - cx).hypot(py - cy);
            let a = (radius + 0.55 - d).clamp(0.0, 1.0);
            if a > 0.0 {
                blend_pixel(buf, width, height, x, y, color, (a * 255.0).round() as u8);
            }
        }
    }
}

#[allow(dead_code)]
pub fn stroke_circle(
    buf: &mut [u32],
    width: u32,
    height: u32,
    cx: f32,
    cy: f32,
    radius: f32,
    thickness: f32,
    color: u32,
) {
    let half = thickness * 0.5 + 0.75;
    let x0 = (cx - radius - half).floor().max(0.0) as i32;
    let y0 = (cy - radius - half).floor().max(0.0) as i32;
    let x1 = (cx + radius + half).ceil().min(width as f32) as i32;
    let y1 = (cy + radius + half).ceil().min(height as f32) as i32;
    for y in y0..y1 {
        for x in x0..x1 {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let dist = ((px - cx).hypot(py - cy) - radius).abs() - thickness * 0.5;
            let a = (1.0 - dist).clamp(0.0, 1.0);
            if a > 0.0 {
                blend_pixel(buf, width, height, x, y, color, (a * 255.0).round() as u8);
            }
        }
    }
}

fn in_round_rect(x: i32, y: i32, x0: i32, y0: i32, x1: i32, y1: i32, r: i32, r2: i32) -> bool {
    if r <= 0 {
        return true;
    }
    let cx = if x < x0 + r {
        x0 + r
    } else if x >= x1 - r {
        x1 - 1 - r
    } else {
        return true;
    };
    let cy = if y < y0 + r {
        y0 + r
    } else if y >= y1 - r {
        y1 - 1 - r
    } else {
        return true;
    };
    if (x < x0 + r || x >= x1 - r) && (y < y0 + r || y >= y1 - r) {
        let dx = x - cx;
        let dy = y - cy;
        dx * dx + dy * dy <= r2
    } else {
        true
    }
}
