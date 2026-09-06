use super::font::FontCtx;
use crate::coord::millipt_to_twips;
use k2f_core::{GlyphPosition, TextPaintStyle};

pub(crate) fn tracking_twips(
    glyphs: &[&GlyphPosition],
    style: &TextPaintStyle,
    fonts: &FontCtx,
) -> i32 {
    if glyphs.len() < 2 {
        return 0;
    }
    let Some(data) = fonts.bytes_for(&style.font_family) else {
        return 0;
    };
    // Large CJK faces: skip rather than parse on every run (lock glyphs already include tracking).
    if data.len() > 512_000 {
        return 0;
    }
    let Ok(face) = ttf_parser::Face::parse(data, 0) else {
        return 0;
    };
    let upem = i128::from(face.units_per_em());
    if upem == 0 {
        return 0;
    }
    let mut extras = Vec::new();
    for g in glyphs.iter().take(glyphs.len() - 1) {
        let Ok(gid) = u16::try_from(g.glyph_id) else {
            return 0;
        };
        let Some(adv) = face.glyph_hor_advance(ttf_parser::GlyphId(gid)) else {
            return 0;
        };
        let font_adv = i128::from(adv) * style.font_size.0 / upem;
        extras.push(g.x_advance.0 - font_adv);
    }
    if extras.is_empty() {
        return 0;
    }
    extras.sort_unstable();
    let extra = extras[extras.len() / 2];
    i32::try_from(millipt_to_twips(i64::try_from(extra).unwrap_or(0))).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::Pt;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    fn tracking_twips_from_lock_extra_advance_on_alias_stem() {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/Roboto-Regular.ttf");
        let bytes = std::fs::read(path).unwrap();
        let mut fonts = BTreeMap::new();
        fonts.insert("assets/fonts/Roboto-Regular.ttf".into(), bytes.clone());
        let ctx = FontCtx::new(&fonts);
        let face = ttf_parser::Face::parse(&bytes, 0).unwrap();
        let gid = face.glyph_index('A').unwrap();
        let native = i128::from(face.glyph_hor_advance(gid).unwrap()) * 12_000
            / i128::from(face.units_per_em());
        let extra = 3_500i128;
        let glyphs = [
            GlyphPosition {
                glyph_id: u32::from(gid.0),
                cluster: 0,
                x_offset: Pt(0),
                y_offset: Pt(0),
                x_advance: Pt(native + extra),
                y_advance: Pt(0),
            },
            GlyphPosition {
                glyph_id: u32::from(gid.0),
                cluster: 1,
                x_offset: Pt(native + extra),
                y_offset: Pt(0),
                x_advance: Pt(native),
                y_advance: Pt(0),
            },
        ];
        let refs: Vec<&GlyphPosition> = glyphs.iter().collect();
        let style = TextPaintStyle {
            font_family: "Roboto-Regular".into(),
            font_size: Pt(12_000),
            color: "#000000".into(),
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
        };
        assert_eq!(tracking_twips(&refs, &style, &ctx), 70);
    }
}
