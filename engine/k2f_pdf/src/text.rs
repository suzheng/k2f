use std::collections::HashMap;
use ttf_parser::{Face, GlyphId, OutlineBuilder};

use crate::coord::pdf_y;
use crate::draw::PageDraw;
use k2f_paint::PlacedGlyph;

pub fn draw_glyph(page: &mut PageDraw, faces: &HashMap<String, Face<'_>>, g: &PlacedGlyph) {
    let Some(face) = k2f_paint::face_for(faces, &g.font_family) else {
        return;
    };
    if g.font_size_pt <= 0.0 || g.units_per_em == 0.0 || g.rgba[3] == 0 {
        return;
    }
    page.note_glyph(g.origin_x_pt, g.origin_y_pt, g.glyph_id);
    let [r, green, blue, _] = g.rgba;
    page.content
        .set_fill_rgb(r as f32 / 255.0, green as f32 / 255.0, blue as f32 / 255.0);
    let scale = (g.font_size_pt / g.units_per_em) as f32;
    let origin_x = g.origin_x_pt as f32;
    let origin_y = pdf_y(page.page_h, g.origin_y_pt);
    let mut outline = PdfOutline {
        content: &mut page.content,
        scale,
        origin_x,
        origin_y,
        italic: g.italic,
        last_x: 0.0,
        last_y: 0.0,
    };
    let _ = face.outline_glyph(GlyphId(g.glyph_id), &mut outline);
    let stroke_pt = g.synthetic_bold_stroke_pt();
    if stroke_pt > 0.0 {
        page.content
            .set_stroke_rgb(r as f32 / 255.0, green as f32 / 255.0, blue as f32 / 255.0);
        page.content.set_line_width(stroke_pt as f32);
        page.content.fill_nonzero_and_stroke();
    } else {
        page.content.fill_nonzero();
    }
}

struct PdfOutline<'a> {
    content: &'a mut pdf_writer::Content,
    scale: f32,
    origin_x: f32,
    origin_y: f32,
    italic: bool,
    last_x: f32,
    last_y: f32,
}

impl PdfOutline<'_> {
    fn tx(&self, x: f32, y: f32) -> f32 {
        let slant_x = if self.italic { y * 0.2126 } else { 0.0 };
        self.origin_x + (x + slant_x) * self.scale
    }

    fn ty(&self, y: f32) -> f32 {
        self.origin_y + y * self.scale
    }
}

impl OutlineBuilder for PdfOutline<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        let x = self.tx(x, y);
        let y = self.ty(y);
        self.content.move_to(x, y);
        self.last_x = x;
        self.last_y = y;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let x = self.tx(x, y);
        let y = self.ty(y);
        self.content.line_to(x, y);
        self.last_x = x;
        self.last_y = y;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let x1 = self.tx(x1, y1);
        let y1 = self.ty(y1);
        let x = self.tx(x, y);
        let y = self.ty(y);
        let cx1 = self.last_x + (x1 - self.last_x) * (2.0 / 3.0);
        let cy1 = self.last_y + (y1 - self.last_y) * (2.0 / 3.0);
        let cx2 = x + (x1 - x) * (2.0 / 3.0);
        let cy2 = y + (y1 - y) * (2.0 / 3.0);
        self.content.cubic_to(cx1, cy1, cx2, cy2, x, y);
        self.last_x = x;
        self.last_y = y;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let x1 = self.tx(x1, y1);
        let y1 = self.ty(y1);
        let x2 = self.tx(x2, y2);
        let y2 = self.ty(y2);
        let x = self.tx(x, y);
        let y = self.ty(y);
        self.content
            .cubic_to(x1, y1, x2, y2, x, y);
        self.last_x = x;
        self.last_y = y;
    }

    fn close(&mut self) {
        self.content.close_path();
    }
}
