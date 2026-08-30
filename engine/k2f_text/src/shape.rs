use crate::bidi::visual_runs;
use crate::coverage::{is_default_ignorable, is_layout_whitespace};
use crate::font::Font;
use k2f_core::{GlyphPosition, Pt};
use rustybuzz::{Direction, Face, UnicodeBuffer};

pub struct TextShaper;

impl TextShaper {
    /// Bidirectional resolution, then HarfBuzz. Fails if the font has no glyph for a character.
    pub fn shape_text(
        text: &str,
        font: &Font,
        font_size: Pt,
    ) -> Result<Vec<GlyphPosition>, String> {
        if text.is_empty() {
            return Ok(vec![]);
        }
        let face = Face::from_slice(&font.data, 0)
            .ok_or_else(|| "Failed to parse font face".to_string())?;
        let units_per_em = face.units_per_em() as i128;
        if units_per_em == 0 {
            return Err("Font has 0 units_per_em".to_string());
        }

        let mut output = Vec::new();
        let mut origin_x = Pt::ZERO;
        let mut origin_y = Pt::ZERO;
        for run in visual_runs(text) {
            let slice = &text[run.start..run.end];
            let origin_cluster = byte_to_char_index(text, run.start);
            let glyphs = shape_run(slice, &face, font_size, units_per_em, run.rtl)?;
            let mut run_advance_x = Pt::ZERO;
            let mut run_advance_y = Pt::ZERO;
            for mut g in glyphs {
                g.cluster += origin_cluster;
                g.x_offset = origin_x + g.x_offset;
                g.y_offset = origin_y + g.y_offset;
                run_advance_x += g.x_advance;
                run_advance_y += g.y_advance;
                output.push(g);
            }
            origin_x += run_advance_x;
            origin_y += run_advance_y;
        }
        Ok(output)
    }

    pub fn shape_run(
        text: &str,
        font: &Font,
        font_size: Pt,
        rtl: bool,
    ) -> Result<Vec<GlyphPosition>, String> {
        if text.is_empty() {
            return Ok(vec![]);
        }
        let face = Face::from_slice(&font.data, 0)
            .ok_or_else(|| "Failed to parse font face".to_string())?;
        let units_per_em = face.units_per_em() as i128;
        if units_per_em == 0 {
            return Err("Font has 0 units_per_em".to_string());
        }
        shape_run(text, &face, font_size, units_per_em, rtl)
    }
}

fn shape_run(
    text: &str,
    face: &Face<'_>,
    font_size: Pt,
    units_per_em: i128,
    rtl: bool,
) -> Result<Vec<GlyphPosition>, String> {
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(if rtl {
        Direction::RightToLeft
    } else {
        Direction::LeftToRight
    });

    let glyph_buffer = rustybuzz::shape(face, &[], buffer);
    let glyph_infos = glyph_buffer.glyph_infos();
    let glyph_positions = glyph_buffer.glyph_positions();

    let mut missing: Vec<(u32, char)> = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for info in glyph_infos {
        if info.glyph_id != 0 {
            continue;
        }
        let cluster = info.cluster as usize;
        if let Some(ch) = text.get(cluster..).and_then(|s| s.chars().next()) {
            if is_default_ignorable(ch) || is_layout_whitespace(ch) {
                continue;
            }
            if seen.insert(ch as u32) {
                missing.push((ch as u32, ch));
            }
        }
    }
    if !missing.is_empty() {
        return Err(missing_glyph_error(text, &missing));
    }

    let mut output = Vec::with_capacity(glyph_infos.len());
    let mut cursor_x = Pt::ZERO;
    let mut cursor_y = Pt::ZERO;
    for (info, pos) in glyph_infos.iter().zip(glyph_positions.iter()) {
        let x_advance = scale(pos.x_advance, font_size, units_per_em);
        let y_advance = scale(pos.y_advance, font_size, units_per_em);
        let x_offset = scale(pos.x_offset, font_size, units_per_em);
        let y_offset = scale(pos.y_offset, font_size, units_per_em);
        output.push(GlyphPosition {
            glyph_id: info.glyph_id,
            cluster: byte_to_char_index(text, info.cluster as usize),
            x_offset: cursor_x + x_offset,
            y_offset: cursor_y + y_offset,
            x_advance,
            y_advance,
        });
        cursor_x += x_advance;
        cursor_y += y_advance;
    }
    Ok(output)
}

fn scale(units: i32, font_size: Pt, upem: i128) -> Pt {
    Pt((units as i128) * font_size.0 / upem)
}

/// Byte offset in `text` → UTF-8 character index. rustybuzz clusters are bytes;
/// selection and `GlyphPosition.cluster` use character indexes.
pub fn byte_to_char_index(text: &str, byte: usize) -> u32 {
    let mut b = byte.min(text.len());
    while b > 0 && !text.is_char_boundary(b) {
        b -= 1;
    }
    text[..b].chars().count() as u32
}

fn missing_glyph_error(text: &str, missing: &[(u32, char)]) -> String {
    let list = missing
        .iter()
        .map(|(cp, ch)| format!("U+{cp:04X} {ch:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "FONT_MISSING_GLYPH: font has no glyph for {list} (text {:?})",
        text.chars().take(16).collect::<String>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::Font;

    fn roboto() -> Font {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts/Roboto-Regular.ttf");
        Font::new(std::fs::read(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}")))
    }

    #[test]
    fn hello_has_positive_width() {
        let glyphs = TextShaper::shape_text("Hello", &roboto(), Pt(12000)).unwrap();
        assert!(!glyphs.is_empty());
        let width: Pt = glyphs.iter().fold(Pt::ZERO, |a, g| a + g.x_advance);
        assert!(width > Pt::ZERO);
    }

    #[test]
    fn shaped_clusters_are_character_indexes() {
        let glyphs = TextShaper::shape_text("Hi", &roboto(), Pt(12000)).unwrap();
        assert_eq!(glyphs[0].cluster, 0);
        assert_eq!(glyphs[1].cluster, 1);
    }

    #[test]
    fn byte_offset_of_cjk_char_maps_to_char_index() {
        assert_eq!(byte_to_char_index("你好", 0), 0);
        assert_eq!(byte_to_char_index("你好", 3), 1);
        assert_eq!(byte_to_char_index("你好", 6), 2);
        assert_eq!(byte_to_char_index("Hi合", 2), 2);
    }

    #[test]
    fn shaping_is_deterministic() {
        let font = roboto();
        let size = Pt(12000);
        let reference = TextShaper::shape_text("Test string 123", &font, size).unwrap();
        for _ in 0..50 {
            assert_eq!(
                reference,
                TextShaper::shape_text("Test string 123", &font, size).unwrap()
            );
        }
    }

    #[test]
    fn missing_cjk_glyph_in_roboto_fails() {
        let err = TextShaper::shape_text("合同", &roboto(), Pt(12000)).unwrap_err();
        assert!(
            err.contains("FONT_MISSING_GLYPH"),
            "expected missing glyph, got {err}"
        );
    }

    #[test]
    fn missing_glyph_names_the_cluster_not_the_first_letter() {
        let err = TextShaper::shape_text("Hi合", &roboto(), Pt(12000)).unwrap_err();
        assert!(err.contains("U+5408"), "expected CJK code point, got {err}");
        assert!(!err.contains("U+0048"), "must not blame Latin H, got {err}");
    }

    #[test]
    fn missing_glyph_lists_every_uncovered_codepoint() {
        let err = TextShaper::shape_text("Hi合韩", &roboto(), Pt(12000)).unwrap_err();
        assert!(err.contains("FONT_MISSING_GLYPH"), "got {err}");
        assert!(err.contains("U+5408"), "expected 合, got {err}");
        assert!(err.contains("U+97E9"), "expected 韩, got {err}");
        assert!(!err.contains("U+0048"), "must not blame Latin H, got {err}");
    }

    fn dejavu() -> Font {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts/DejaVuSans.ttf");
        Font::new(std::fs::read(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}")))
    }

    #[test]
    fn mixed_bidi_shapes_with_dejavu() {
        let glyphs = TextShaper::shape_text("Hi שלום", &dejavu(), Pt(12000)).unwrap();
        assert!(glyphs.len() >= 4);
        assert!(glyphs.iter().all(|g| g.glyph_id != 0));
        let width: Pt = glyphs.iter().fold(Pt::ZERO, |a, g| a + g.x_advance);
        assert!(width > Pt::ZERO);
    }
}
