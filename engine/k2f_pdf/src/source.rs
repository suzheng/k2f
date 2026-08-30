use k2f_core::GlyphPosition;
use k2f_paint::PlacedGlyph;
use std::collections::HashMap;
use ttf_parser::{Face, GlyphId};

use crate::draw::PageDraw;
use crate::text::draw_glyph;

pub fn source_line(hash: &str) -> String {
    format!("Official source is K2F. appearance_hash={hash}")
}

pub fn caption_glyphs(
    page_h: f64,
    faces: &HashMap<String, Face<'_>>,
    hash: &str,
) -> Vec<(PlacedGlyph, String)> {
    let Some(face) = k2f_paint::face_for(faces, "default") else {
        return vec![];
    };
    let text = source_line(hash);
    let font_size = 6.0;
    let units = face.units_per_em() as f64;
    if units == 0.0 {
        return vec![];
    }
    let mut x = 8.0;
    let y = page_h - 8.0;
    let mut out = Vec::new();
    for ch in text.chars() {
        let Some(gid) = face.glyph_index(ch) else {
            continue;
        };
        let g = PlacedGlyph {
            origin_x_pt: x,
            origin_y_pt: y,
            font_size_pt: font_size,
            units_per_em: units,
            glyph_id: gid.0,
            rgba: [90, 90, 90, 255],
            font_family: "default".into(),
            bold: false,
            italic: false,
            cluster: GlyphPosition::CLUSTER_NOT_SOURCE,
        };
        out.push((g, ch.to_string()));
        let adv = face.glyph_hor_advance(GlyphId(gid.0)).unwrap_or(0) as f64;
        x += font_size * adv / units;
    }
    out
}

pub fn draw_source_caption(page: &mut PageDraw, faces: &HashMap<String, Face<'_>>, hash: &str) {
    for (g, _) in caption_glyphs(page.page_h, faces, hash) {
        draw_glyph_silent(page, faces, &g);
    }
}

fn draw_glyph_silent(page: &mut PageDraw, faces: &HashMap<String, Face<'_>>, g: &PlacedGlyph) {
    let len = page.notes.len();
    draw_glyph(page, faces, g);
    page.notes.truncate(len);
}
