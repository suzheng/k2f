use k2f_core::{GeometryNode, GlyphPosition, Rect, TextGlyphRun};
use std::collections::HashMap;
use ttf_parser::{name_id, Face, Language};

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

/// TTF subfamily (`Bold`, `Italic`, `Regular`, …). Same table IDML reads.
pub fn face_subfamily(face: &Face<'_>) -> String {
    name_english(face, name_id::TYPOGRAPHIC_SUBFAMILY)
        .or_else(|| name_english(face, name_id::SUBFAMILY))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Regular".into())
}

pub fn face_style_is_italic(style: &str) -> bool {
    let l = style.to_ascii_lowercase();
    l.contains("italic") || l.contains("oblique")
}

pub fn face_style_is_bold(style: &str) -> bool {
    style.to_ascii_lowercase().contains("bold")
}

fn name_english(face: &Face<'_>, id: u16) -> Option<String> {
    let mut fallback = None;
    for name in face.names() {
        if name.name_id != id || !name.is_unicode() {
            continue;
        }
        let Some(s) = name.to_string() else {
            continue;
        };
        if name.language() == Language::English_UnitedStates {
            return Some(s);
        }
        if fallback.is_none() {
            fallback = Some(s);
        }
    }
    fallback
}

impl PlacedGlyph {
    /// Stroke width in pt when paint asked for bold on a non-bold face (1/30 em).
    pub fn synthetic_bold_stroke_pt(&self, face: &Face<'_>) -> f64 {
        if self.bold && !face_style_is_bold(&face_subfamily(face)) {
            self.font_size_pt / 30.0
        } else {
            0.0
        }
    }

    /// Shear when paint asked for italic on a non-italic face.
    pub fn synthetic_italic(&self, face: &Face<'_>) -> bool {
        self.italic && !face_style_is_italic(&face_subfamily(face))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subfamily_helpers_match_idml() {
        assert!(face_style_is_bold("Bold"));
        assert!(face_style_is_bold("Bold Italic"));
        assert!(!face_style_is_bold("Regular"));
        assert!(!face_style_is_bold("Medium"));
        assert!(face_style_is_italic("Italic"));
        assert!(face_style_is_italic("Oblique"));
        assert!(!face_style_is_italic("Regular"));
    }

    #[test]
    fn roboto_regular_still_synthesizes_bold() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts/Roboto-Regular.ttf");
        let bytes = std::fs::read(&path).expect("Roboto-Regular.ttf");
        let face = Face::parse(&bytes, 0).unwrap();
        assert_eq!(face_subfamily(&face), "Regular");
        let g = PlacedGlyph {
            origin_x_pt: 0.0,
            origin_y_pt: 0.0,
            font_size_pt: 12.0,
            units_per_em: 1000.0,
            glyph_id: 1,
            rgba: [0, 0, 0, 255],
            font_family: "Roboto-Regular".into(),
            bold: true,
            italic: true,
            cluster: 0,
        };
        assert!((g.synthetic_bold_stroke_pt(&face) - 0.4).abs() < 1e-9);
        assert!(g.synthetic_italic(&face));
        let regular = PlacedGlyph {
            bold: false,
            italic: false,
            ..g.clone()
        };
        assert_eq!(regular.synthetic_bold_stroke_pt(&face), 0.0);
        assert!(!regular.synthetic_italic(&face));
    }

    #[test]
    fn bold_name_table_skips_synthetic_stroke() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts/Roboto-Regular.ttf");
        let mut bytes = std::fs::read(&path).expect("Roboto-Regular.ttf");
        let regular: &[u8] = &[
            0x00, 0x52, 0x00, 0x65, 0x00, 0x67, 0x00, 0x75, 0x00, 0x6c, 0x00, 0x61, 0x00, 0x72,
        ];
        let boldxxx: &[u8] = &[
            0x00, 0x42, 0x00, 0x6f, 0x00, 0x6c, 0x00, 0x64, 0x00, 0x78, 0x00, 0x78, 0x00, 0x78,
        ];
        let mut hits = 0usize;
        let mut i = 0;
        while i + regular.len() <= bytes.len() {
            if bytes[i..i + regular.len()] == *regular {
                bytes[i..i + boldxxx.len()].copy_from_slice(boldxxx);
                hits += 1;
                i += regular.len();
            } else {
                i += 1;
            }
        }
        assert!(hits > 0, "Roboto name table should contain UTF-16 Regular");
        let face = Face::parse(&bytes, 0).unwrap();
        assert!(
            face_style_is_bold(&face_subfamily(&face)),
            "patched subfamily {}",
            face_subfamily(&face)
        );
        let g = PlacedGlyph {
            origin_x_pt: 0.0,
            origin_y_pt: 0.0,
            font_size_pt: 12.0,
            units_per_em: 1000.0,
            glyph_id: 1,
            rgba: [0, 0, 0, 255],
            font_family: "Roboto-Bold".into(),
            bold: true,
            italic: false,
            cluster: 0,
        };
        assert_eq!(g.synthetic_bold_stroke_pt(&face), 0.0);
        assert!(!g.synthetic_italic(&face));
    }
}
