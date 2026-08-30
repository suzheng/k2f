use k2f_core::Rect;
use pdf_writer::Content;

/// Content stream plus lock-geometry notes parsed back by tests.
pub struct PageDraw {
    pub page_w: f64,
    pub page_h: f64,
    pub content: Content,
    pub notes: String,
}

impl PageDraw {
    pub fn new(page_w: f64, page_h: f64) -> Self {
        Self {
            page_w,
            page_h,
            content: Content::new(),
            notes: String::new(),
        }
    }

    pub fn note_box(&mut self, rect: &Rect) {
        self.notes.push_str(&format!(
            "% k2f.b {:.3} {:.3} {:.3} {:.3}\n",
            rect.x.as_f64_pt(),
            rect.y.as_f64_pt(),
            rect.width.as_f64_pt(),
            rect.height.as_f64_pt()
        ));
    }

    pub fn note_glyph(&mut self, x: f64, y: f64, gid: u16) {
        self.notes
            .push_str(&format!("% k2f.g {x:.3} {y:.3} {gid}\n"));
    }

    pub fn note_image(&mut self, rect: &Rect) {
        self.notes.push_str(&format!(
            "% k2f.i {:.3} {:.3} {:.3} {:.3}\n",
            rect.x.as_f64_pt(),
            rect.y.as_f64_pt(),
            rect.width.as_f64_pt(),
            rect.height.as_f64_pt()
        ));
    }

    pub fn note_image_full(&mut self, w_px: u32, h_px: u32) {
        self.notes.push_str(&format!(
            "% k2f.i 0.000 0.000 {:.3} {:.3}\n",
            self.page_w, self.page_h
        ));
        let _ = (w_px, h_px);
    }

    pub fn finish(self) -> Vec<u8> {
        let mut out = self.notes.into_bytes();
        out.extend_from_slice(self.content.finish().as_ref());
        out
    }
}

pub fn parse_notes(content: &[u8]) -> (Vec<[f64; 4]>, Vec<(f64, f64, u16)>, Vec<[f64; 4]>) {
    let text = String::from_utf8_lossy(content);
    let mut boxes = Vec::new();
    let mut glyphs = Vec::new();
    let mut images = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("% k2f.b ") {
            if let Some(v) = four(rest) {
                boxes.push(v);
            }
        } else if let Some(rest) = line.strip_prefix("% k2f.g ") {
            let p: Vec<&str> = rest.split_whitespace().collect();
            if p.len() == 3 {
                if let (Ok(x), Ok(y), Ok(g)) = (p[0].parse(), p[1].parse(), p[2].parse()) {
                    glyphs.push((x, y, g));
                }
            }
        } else if let Some(rest) = line.strip_prefix("% k2f.i ") {
            if let Some(v) = four(rest) {
                images.push(v);
            }
        }
    }
    (boxes, glyphs, images)
}

fn four(s: &str) -> Option<[f64; 4]> {
    let p: Vec<&str> = s.split_whitespace().collect();
    if p.len() != 4 {
        return None;
    }
    Some([
        p[0].parse().ok()?,
        p[1].parse().ok()?,
        p[2].parse().ok()?,
        p[3].parse().ok()?,
    ])
}
