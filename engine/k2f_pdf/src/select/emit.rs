use k2f_paint::PlacedGlyph;
use pdf_writer::types::TextRenderingMode;
use pdf_writer::{Name, Str};
use std::collections::HashMap;

use crate::coord::pdf_y;
use crate::draw::PageDraw;

pub fn emit(
    page: &mut PageDraw,
    glyphs: &[(PlacedGlyph, String)],
    family_to_font: &HashMap<String, String>,
) {
    let visible: Vec<_> = glyphs.iter().filter(|(_, uni)| !uni.is_empty()).collect();
    if visible.is_empty() {
        return;
    }
    page.content.begin_text();
    page.content
        .set_text_rendering_mode(TextRenderingMode::Invisible);
    let mut last: Option<(&str, f32)> = None;
    for (g, _) in &visible {
        let Some(res) = family_to_font
            .get(g.font_family.as_str())
            .or_else(|| family_to_font.get("default"))
        else {
            continue;
        };
        let size = g.font_size_pt as f32;
        if last != Some((res.as_str(), size)) {
            page.content.set_font(Name(res.as_bytes()), size);
            last = Some((res.as_str(), size));
        }
        let x = g.origin_x_pt as f32;
        let y = pdf_y(page.page_h, g.origin_y_pt);
        page.content.set_text_matrix([1.0, 0.0, 0.0, 1.0, x, y]);
        let bytes = [(g.glyph_id >> 8) as u8, g.glyph_id as u8];
        page.content.show(Str(&bytes));
    }
    page.content.end_text();
}
