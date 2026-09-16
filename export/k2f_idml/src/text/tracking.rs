use super::font::FontCtx;
use k2f_core::{GlyphPosition, TextPaintStyle};

/// Extra lock advance vs the face's native advance, as IDML Tracking (1/1000 em).
/// Omit when unmeasurable — never fake `Tracking="0"`.
pub(crate) fn tracking_em(
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
    if data.len() > 512_000 {
        return 0;
    }
    let Ok(face) = ttf_parser::Face::parse(data, 0) else {
        return 0;
    };
    let upem = i128::from(face.units_per_em());
    if upem == 0 || style.font_size.0 == 0 {
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
    i32::try_from(extra.saturating_mul(1000) / style.font_size.0).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::Pt;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    fn tracking_em_from_lock_extra_advance() {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/Roboto-Regular.ttf");
        let bytes = std::fs::read(&path).unwrap();
        let mut fonts = BTreeMap::new();
        fonts.insert("assets/fonts/Roboto-Regular.ttf".into(), bytes.clone());
        let ctx = FontCtx::new(&fonts);
        let face = ttf_parser::Face::parse(&bytes, 0).unwrap();
        let gid = face.glyph_index('A').unwrap();
        let size = 12_000i128;
        let native = i128::from(face.glyph_hor_advance(gid).unwrap()) * size
            / i128::from(face.units_per_em());
        let extra = -1_500i128;
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
        let style = TextPaintStyle {
            font_family: "default".into(),
            font_size: Pt(size),
            color: "#000000".into(),
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
        };
        let refs: Vec<&GlyphPosition> = glyphs.iter().collect();
        assert_eq!(tracking_em(&refs, &style, &ctx), -125);
    }
}
