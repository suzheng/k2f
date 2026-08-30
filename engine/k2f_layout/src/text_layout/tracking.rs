use k2f_core::{GlyphPosition, Pt};

/// Extra advance between glyphs (not after the last glyph in the run).
///
/// Shaped `x_offset` values are run-relative origins; later glyphs shift by the
/// accumulated tracking so measure and paint stay aligned.
pub fn apply_tracking(glyphs: &mut [GlyphPosition], letter_spacing: Pt) {
    if glyphs.len() < 2 || letter_spacing == Pt::ZERO {
        return;
    }
    let last = glyphs.len() - 1;
    let mut extra = Pt::ZERO;
    for (i, g) in glyphs.iter_mut().enumerate() {
        g.x_offset = g.x_offset + extra;
        if i < last {
            g.x_advance = g.x_advance + letter_spacing;
            extra = extra + letter_spacing;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glyph(x_offset: i128, x_advance: i128) -> GlyphPosition {
        GlyphPosition {
            glyph_id: 1,
            cluster: 0,
            x_offset: Pt(x_offset),
            y_offset: Pt::ZERO,
            x_advance: Pt(x_advance),
            y_advance: Pt::ZERO,
        }
    }

    #[test]
    fn tracking_spaces_all_but_last_and_shifts_later_offsets() {
        let mut glyphs = vec![glyph(0, 1000), glyph(1000, 1100), glyph(2100, 900)];
        apply_tracking(&mut glyphs, Pt(500));
        assert_eq!(glyphs[0].x_advance, Pt(1500));
        assert_eq!(glyphs[0].x_offset, Pt(0));
        assert_eq!(glyphs[1].x_advance, Pt(1600));
        assert_eq!(glyphs[1].x_offset, Pt(1500));
        assert_eq!(glyphs[2].x_advance, Pt(900));
        assert_eq!(glyphs[2].x_offset, Pt(3100));
    }

    #[test]
    fn tracking_zero_and_single_glyph_are_noops() {
        let mut one = vec![glyph(0, 1000)];
        apply_tracking(&mut one, Pt(500));
        assert_eq!(one[0].x_advance, Pt(1000));

        let mut two = vec![glyph(0, 1000), glyph(1000, 1000)];
        apply_tracking(&mut two, Pt::ZERO);
        assert_eq!(two[1].x_offset, Pt(1000));
    }
}
