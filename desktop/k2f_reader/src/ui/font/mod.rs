//! Antialiased UI type for chrome only (not document paint).
//! Roboto → embedded NotoSansSC → OS UI faces for uncovered glyphs.

mod system;

use super::draw::blend_over;
use super::draw::Rect;
use fontdue::Font;
use std::sync::OnceLock;

const ROBOTO: &[u8] = include_bytes!("../../../../../assets/fonts/Roboto-Regular.ttf");
const NOTO_SC: &[u8] = include_bytes!("../../../../../assets/fonts/NotoSansSC-Regular.otf");

pub const TITLE_PX: f32 = 18.0;
pub const BODY_PX: f32 = 15.0;
pub const CAPTION_PX: f32 = 13.0;

struct Embedded {
    roboto: Font,
    noto_sc: Font,
}

static EMBEDDED: OnceLock<Embedded> = OnceLock::new();

fn embedded() -> &'static Embedded {
    EMBEDDED.get_or_init(|| Embedded {
        roboto: Font::from_bytes(ROBOTO, fontdue::FontSettings::default()).expect("Roboto-Regular"),
        noto_sc: Font::from_bytes(NOTO_SC, fontdue::FontSettings::default())
            .expect("NotoSansSC-Regular"),
    })
}

fn with_face<R>(ch: char, f: impl FnOnce(&Font) -> R) -> R {
    let emb = embedded();
    if emb.roboto.has_glyph(ch) {
        return f(&emb.roboto);
    }
    if emb.noto_sc.has_glyph(ch) {
        return f(&emb.noto_sc);
    }
    if let Ok(mut sys) = system::pool().lock() {
        if let Some(font) = sys.font_for(ch) {
            return f(font);
        }
    }
    f(&emb.roboto)
}

fn primary_font() -> &'static Font {
    &embedded().roboto
}

pub fn text_width_px(s: &str, px: f32) -> u32 {
    let mut w = 0.0f32;
    for ch in s.chars() {
        w += with_face(ch, |font| font.metrics(ch, px).advance_width);
    }
    w.ceil().max(0.0) as u32
}

pub fn em_height(px: f32) -> u32 {
    primary_font()
        .horizontal_line_metrics(px)
        .map(|m| (m.ascent - m.descent).ceil() as u32)
        .unwrap_or(px.ceil() as u32)
}

#[allow(dead_code)]
pub fn line_height(px: f32) -> u32 {
    primary_font()
        .horizontal_line_metrics(px)
        .map(|m| m.new_line_size.ceil() as u32)
        .unwrap_or(px.ceil() as u32)
}

/// `y` is the top of the em-box (ascent line). Glyphs share one baseline.
pub fn draw_text_px(
    buf: &mut [u32],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    text: &str,
    color: u32,
    px: f32,
) {
    let ascent = primary_font()
        .horizontal_line_metrics(px)
        .map(|m| m.ascent)
        .unwrap_or(px);
    let baseline = y as f32 + ascent;
    let mut pen_x = x as f32;
    for ch in text.chars() {
        let (metrics, bitmap) = with_face(ch, |font| font.rasterize(ch, px));
        let gx = pen_x.round() as i32 + metrics.xmin;
        let gy = baseline.round() as i32 - metrics.ymin - metrics.height as i32;
        blit_coverage(
            buf,
            width,
            height,
            gx,
            gy,
            metrics.width,
            metrics.height,
            &bitmap,
            color,
        );
        pen_x += metrics.advance_width;
    }
}

pub fn draw_text_centered(
    buf: &mut [u32],
    width: u32,
    height: u32,
    rect: Rect,
    text: &str,
    color: u32,
    px: f32,
) {
    let tw = text_width_px(text, px) as i32;
    let th = em_height(px) as i32;
    let x = rect.x + (rect.w as i32 - tw).max(0) / 2;
    let y = rect.y + (rect.h as i32 - th).max(0) / 2;
    draw_text_px(buf, width, height, x, y, text, color, px);
}

fn blit_coverage(
    buf: &mut [u32],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    gw: usize,
    gh: usize,
    bitmap: &[u8],
    color: u32,
) {
    let dw = width as i32;
    let dh = height as i32;
    for row in 0..gh {
        let dy = y + row as i32;
        if dy < 0 || dy >= dh {
            continue;
        }
        let dest_row = dy as u32 * width;
        for col in 0..gw {
            let a = bitmap[row * gw + col];
            if a == 0 {
                continue;
            }
            let dx = x + col as i32;
            if dx < 0 || dx >= dw {
                continue;
            }
            let i = (dest_row + dx as u32) as usize;
            buf[i] = blend_over(buf[i], color, a);
        }
    }
}

pub fn wrap_text(s: &str, max_px: u32, px: f32) -> Vec<String> {
    if s.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in s.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if text_width_px(&candidate, px) <= max_px {
            current = candidate;
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        if text_width_px(word, px) <= max_px {
            current = word.to_string();
            continue;
        }
        for ch in word.chars() {
            let mut next = current.clone();
            next.push(ch);
            if !current.is_empty() && text_width_px(&next, px) > max_px {
                lines.push(std::mem::take(&mut current));
                current.push(ch);
            } else {
                current = next;
            }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        vec![String::new()]
    } else {
        lines
    }
}

pub fn ellipsize_to_width(s: &str, max_px: u32, px: f32) -> String {
    if text_width_px(s, px) <= max_px {
        return s.to_string();
    }
    let ell = "...";
    let ell_w = text_width_px(ell, px);
    if max_px <= ell_w + 8 {
        return ell.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut best = ell.to_string();
    for keep in 1..n {
        let left = keep / 2 + keep % 2;
        let right = keep / 2;
        if left + right >= n {
            break;
        }
        let mut out: String = chars[..left].iter().collect();
        out.push_str(ell);
        out.extend(chars[n - right..].iter());
        if text_width_px(&out, px) <= max_px {
            best = out;
        } else {
            break;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_ink(buf: &[u32]) -> bool {
        buf.iter().any(|&p| p != 0)
    }

    fn ink_for_char(ch: char, px: f32) -> bool {
        let w = 96u32;
        let h = 64u32;
        let mut buf = vec![0u32; (w * h) as usize];
        draw_text_px(&mut buf, w, h, 8, 8, &ch.to_string(), 0xFFFFFF, px);
        has_ink(&buf)
    }

    fn last_ink_row(ch: char, px: f32) -> i32 {
        let w = 96u32;
        let h = 64u32;
        let mut buf = vec![0u32; (w * h) as usize];
        draw_text_px(&mut buf, w, h, 8, 8, &ch.to_string(), 0xFFFFFF, px);
        let mut last = -1;
        for y in 0..h as i32 {
            for x in 0..w {
                if buf[(y as u32 * w + x) as usize] != 0 {
                    last = y;
                }
            }
        }
        last
    }

    #[test]
    fn cjk_char_paints_ink() {
        assert!(ink_for_char('对', 18.0), "NotoSansSC fallback must paint CJK");
    }

    #[test]
    fn latin_still_paints_ink() {
        let w = 160u32;
        let h = 64u32;
        let mut buf = vec![0u32; (w * h) as usize];
        draw_text_px(&mut buf, w, h, 8, 8, "INVOICE", 0xFFFFFF, 18.0);
        assert!(has_ink(&buf), "Roboto must still paint Latin chrome text");
    }

    #[test]
    fn deep_ocean_title_paints_every_glyph() {
        // Real manifest title from deep-ocean-calm-gradient.K2F.
        // 谧 is outside the tiny embedded NotoSansSC subset and needs OS fallback.
        const TITLE: &str = "深海静谧";
        for ch in TITLE.chars() {
            if !ink_for_char(ch, 18.0) {
                // Linux CI may lack CJK system faces for rare Han.
                if ch == '谧' && !cfg!(any(target_os = "macos", target_os = "windows")) {
                    eprintln!("skip 谧: no OS CJK face on this host");
                    continue;
                }
                panic!("chrome must paint {ch:?} (U+{:04X})", ch as u32);
            }
        }
    }

    #[test]
    fn letters_share_a_baseline() {
        let e = last_ink_row('E', 32.0);
        let o = last_ink_row('o', 32.0);
        let x = last_ink_row('x', 32.0);
        assert!(
            e > 8 && o > 8 && x > 8,
            "glyphs must paint, E={e} o={o} x={x}"
        );
        assert!(
            (e - o).abs() <= 3,
            "E and o must sit on one baseline, E={e} o={o}"
        );
        assert!(
            (e - x).abs() <= 3,
            "E and x must sit on one baseline, E={e} x={x}"
        );
    }

    #[test]
    fn wrap_text_breaks_on_words() {
        let lines = wrap_text("Content and lock do not match", 80, 15.0);
        assert!(
            lines.len() >= 2,
            "narrow width must wrap the sentence, got {lines:?}"
        );
        assert!(lines.iter().all(|l| !l.is_empty()), "{lines:?}");
    }
}
