use k2f_core::{GeometryNode, GlyphPosition, Rect, TextGlyphRun};
use std::collections::HashMap;
use ttf_parser::Face;

use crate::error::PaintError;

/// A glyph to draw at lock coordinates. Shared by PNG and PDF executors.
#[derive(Debug, Clone)]
pub struct PlacedGlyph {
    pub origin_x_pt: f64,
    pub origin_y_pt: f64,
    pub font_size_pt: f64,
    pub units_per_em: f64,
    pub glyph_id: u16,
    pub rgba: [u8; 4],
    pub font_family: String,
    pub bold: bool,
    pub italic: bool,
    pub cluster: u32,
}

impl PlacedGlyph {
    /// Stroke width in pt for Regular-only fonts (1/30 em). Zero when not bold.
    pub fn synthetic_bold_stroke_pt(&self) -> f64 {
        if self.bold {
            self.font_size_pt / 30.0
        } else {
            0.0
        }
    }
}

pub fn face_for<'a>(faces: &'a HashMap<String, Face<'a>>, name: &str) -> Option<&'a Face<'a>> {
    faces.get(name).or_else(|| faces.get("default"))
}

pub fn baseline_y_pt(faces: &HashMap<String, Face<'_>>, rect: &Rect, runs: &[TextGlyphRun]) -> f64 {
    let max_asc_pt = runs
        .iter()
        .map(|r| {
            let face = face_for(faces, &r.style.font_family);
            let units = face.map(|f| f.units_per_em() as f64).unwrap_or(1000.0);
            let asc = face.map(|f| f.ascender() as f64).unwrap_or(800.0);
            if units == 0.0 {
                0.0
            } else {
                (r.style.font_size.as_f64_pt() * asc) / units
            }
        })
        .fold(0.0f64, f64::max);
    rect.y.as_f64_pt() + max_asc_pt
}

pub fn placed_glyphs(
    faces: &HashMap<String, Face<'_>>,
    geo: &GeometryNode,
    rect: &Rect,
    runs: &[TextGlyphRun],
) -> Result<Vec<PlacedGlyph>, PaintError> {
    if geo.glyphs.is_empty() || runs.is_empty() {
        return Ok(Vec::new());
    }
    let baseline_y_pt = baseline_y_pt(faces, rect, runs);
    let mut out = Vec::new();
    for run in runs {
        let face = face_for(faces, &run.style.font_family).ok_or_else(|| {
            PaintError::Font(format!("font '{}' not embedded", run.style.font_family))
        })?;
        let rgba = crate::geom::parse_hex_rgba(&run.style.color).unwrap_or([0, 0, 0, 255]);
        if rgba[3] == 0 {
            continue;
        }
        let font_size_pt = run.style.font_size.as_f64_pt();
        if font_size_pt <= 0.0 {
            continue;
        }
        let units_per_em = face.units_per_em() as f64;
        if units_per_em == 0.0 {
            continue;
        }
        let [start, end] = run.glyph_range;
        let start = start.min(geo.glyphs.len());
        let end = end.min(geo.glyphs.len());
        for glyph in &geo.glyphs[start..end] {
            if let Some(placed) = place_glyph(
                rect,
                baseline_y_pt,
                glyph,
                font_size_pt,
                units_per_em,
                rgba,
                &run.style.font_family,
                run.style.bold,
                run.style.italic,
            ) {
                out.push(placed);
            }
        }
    }
    Ok(out)
}

pub fn glyph_origin_pt(rect: &Rect, baseline_y_pt: f64, glyph: &GlyphPosition) -> (f64, f64) {
    (
        rect.x.as_f64_pt() + glyph.x_offset.as_f64_pt(),
        baseline_y_pt + glyph.y_offset.as_f64_pt(),
    )
}

fn place_glyph(
    rect: &Rect,
    baseline_y_pt: f64,
    glyph: &GlyphPosition,
    font_size_pt: f64,
    units_per_em: f64,
    rgba: [u8; 4],
    font_family: &str,
    bold: bool,
    italic: bool,
) -> Option<PlacedGlyph> {
    let glyph_id = u16::try_from(glyph.glyph_id).ok()?;
    let (origin_x_pt, origin_y_pt) = glyph_origin_pt(rect, baseline_y_pt, glyph);
    Some(PlacedGlyph {
        origin_x_pt,
        origin_y_pt,
        font_size_pt,
        units_per_em,
        glyph_id,
        rgba,
        font_family: font_family.to_string(),
        bold,
        italic,
        cluster: glyph.cluster,
    })
}
