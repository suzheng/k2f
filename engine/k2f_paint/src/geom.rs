use k2f_core::{Pt, Rect};
use tiny_skia::{Color, Path, PathBuilder};

#[derive(Debug, Clone, Copy)]
pub struct RectPx {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

pub fn pt_to_px_u32(pt: Pt, scale: f32) -> u32 {
    let v = pt.as_f64_pt() * (scale as f64);
    v.round().max(0.0) as u32
}

pub fn pt_i64_to_px_i32(pt_fixed: i64, scale: f32) -> i32 {
    let pt = (pt_fixed as f64) / 1000.0;
    (pt * (scale as f64)).round() as i32
}

pub fn rect_to_px(rect: &Rect, scale: f32) -> RectPx {
    let left = (rect.x.as_f64_pt() * scale as f64).round() as i32;
    let top = (rect.y.as_f64_pt() * scale as f64).round() as i32;
    let right = ((rect.x + rect.width).as_f64_pt() * scale as f64).round() as i32;
    let bottom = ((rect.y + rect.height).as_f64_pt() * scale as f64).round() as i32;
    RectPx {
        left,
        top,
        right: right.max(left + 1),
        bottom: bottom.max(top + 1),
    }
}

pub fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<Path> {
    let mut pb = PathBuilder::new();
    let r = r.max(0.0).min(w * 0.5).min(h * 0.5);
    if r == 0.0 {
        let rect = tiny_skia::Rect::from_xywh(x, y, w, h)?;
        pb.push_rect(rect);
        return pb.finish();
    }

    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    pb.finish()
}

pub fn parse_hex_rgba(s: &str) -> Option<[u8; 4]> {
    let s = s.trim();
    if !s.starts_with('#') {
        return None;
    }
    let hex = &s[1..];
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some([r, g, b, 255])
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some([r, g, b, a])
        }
        _ => None,
    }
}

pub fn parse_hex_color_rgba8(s: &str) -> Option<Color> {
    let [r, g, b, a] = parse_hex_rgba(s)?;
    Some(Color::from_rgba8(r, g, b, a))
}
