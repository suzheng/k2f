mod draw;
mod lines;

use k2f_paint::OpenedDocument;
use std::collections::{BTreeMap, HashMap};
use ttf_parser::Face;

use crate::draw::PageDraw;
use crate::error::PdfError;
use crate::select::{self, FontSet};

pub fn build_page(
    doc: &OpenedDocument,
    faces: &HashMap<String, Face<'_>>,
    fonts: &FontSet,
    gid_maps: &mut [BTreeMap<u16, String>],
) -> Result<(f64, f64, Vec<u8>), PdfError> {
    let lock = doc.lock().ok_or(PdfError::Unlocked)?;
    let first = lock
        .geometry
        .pages
        .first()
        .ok_or(PdfError::Write("lock has no pages".into()))?;
    let w = first.width.as_f64_pt();
    let h = first.height.as_f64_pt();
    let text_lines = lines::verification_lines(doc);
    let mut page_draw = PageDraw::new(w, h);
    for line in &text_lines {
        if line.starts_with("status:")
            || line.starts_with("content_hash:")
            || line.starts_with("fingerprint:")
        {
            page_draw.notes.push_str(&format!("% k2f.verify {line}\n"));
        }
    }
    let glyphs = draw::line_glyphs(h, faces, &text_lines);
    draw::draw_lines(&mut page_draw, faces, &text_lines);
    select::merge_gid_maps(fonts, &glyphs, gid_maps);
    select::emit_page(&mut page_draw, &glyphs, fonts);
    Ok((w, h, page_draw.finish()))
}
