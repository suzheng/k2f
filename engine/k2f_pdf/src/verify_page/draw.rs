use k2f_core::GlyphPosition;
use k2f_paint::PlacedGlyph;
use std::collections::HashMap;
use ttf_parser::{Face, GlyphId};

use crate::draw::PageDraw;
use crate::text::draw_glyph;

const FONT_SIZE: f64 = 10.0;
const MARGIN: f64 = 48.0;
const LINE_HEIGHT: f64 = 14.0;

pub fn line_glyphs(
    page_h: f64,
    faces: &HashMap<String, Face<'_>>,
    lines: &[String],
) -> Vec<(PlacedGlyph, String)> {
    let Some(face) = k2f_paint::face_for(faces, "default") else {
        return vec![];
    };
    let units = face.units_per_em() as f64;
    if units == 0.0 {
        return vec![];
    }
    let mut out = Vec::new();
    let mut y = page_h - MARGIN;
    for line in lines {
        let mut x = MARGIN;
        for ch in line.chars() {
            let Some(gid) = face.glyph_index(ch) else {
                continue;
            };
            let g = PlacedGlyph {
                origin_x_pt: x,
                origin_y_pt: y,
                font_size_pt: FONT_SIZE,
                units_per_em: units,
                glyph_id: gid.0,
                rgba: [20, 20, 20, 255],
                font_family: "default".into(),
                bold: false,
                italic: false,
                cluster: GlyphPosition::CLUSTER_NOT_SOURCE,
            };
            out.push((g, ch.to_string()));
            let adv = face.glyph_hor_advance(GlyphId(gid.0)).unwrap_or(0) as f64;
            x += FONT_SIZE * adv / units;
        }
        y -= LINE_HEIGHT;
    }
    out
}

pub fn draw_lines(page: &mut PageDraw, faces: &HashMap<String, Face<'_>>, lines: &[String]) {
    for (g, _) in line_glyphs(page.page_h, faces, lines) {
        draw_glyph_silent(page, faces, &g);
    }
}

fn draw_glyph_silent(page: &mut PageDraw, faces: &HashMap<String, Face<'_>>, g: &PlacedGlyph) {
    let len = page.notes.len();
    draw_glyph(page, faces, g);
    page.notes.truncate(len);
}
