//! Native TeX-subset math layout for K2F. Millipt only; no floats in geometry.

mod atom;
mod constants;
mod error;
mod layout;
mod parse;
mod style;
mod token;

#[cfg(test)]
mod v2_tests;

pub use error::MathError;
pub use style::MathStyle;

use k2f_core::{FillRect, Pt};
use k2f_text::Font;
use layout::{flatten, layout_node};
use parse::parse_tex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathLayout {
    pub width: Pt,
    pub height: Pt,
    pub ascent: Pt,
    pub descent: Pt,
    pub glyphs: Vec<PlacedMathGlyph>,
    pub fill_rects: Vec<FillRect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedMathGlyph {
    pub glyph_id: u32,
    /// Distance from the math box's left edge.
    pub x: Pt,
    /// Distance from the math box's top edge to this glyph's baseline.
    pub y: Pt,
    pub font_size: Pt,
    pub cluster: u32,
}

pub fn layout_tex(
    tex: &str,
    font: &Font,
    font_size: Pt,
    style: MathStyle,
) -> Result<MathLayout, MathError> {
    let ast = parse_tex(tex)?;
    let boxed = layout_node(&ast, font, font_size, style)?;
    Ok(flatten(boxed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_text::Font;

    fn font() -> Font {
        Font::new(include_bytes!("../../../assets/fonts/NotoSansMath-Regular.ttf").to_vec())
    }

    fn layout(tex: &str) -> MathLayout {
        layout_tex(tex, &font(), Pt(12000), MathStyle::Display).expect(tex)
    }

    #[test]
    fn emc2_is_deterministic() {
        let a = layout("E=mc^2");
        let b = layout("E=mc^2");
        assert_eq!(a, b);
        assert!(a.width.0 > 0);
        assert!(!a.glyphs.is_empty());
    }

    #[test]
    fn frac_has_rule() {
        let m = layout(r"\frac{1}{2}");
        assert!(
            !m.fill_rects.is_empty(),
            "expected a fraction bar, got {:?}",
            m.fill_rects
        );
        assert!(m.glyphs.len() >= 2);
    }

    #[test]
    fn sqrt_has_vinculum() {
        let m = layout(r"\sqrt{a+b}");
        assert!(!m.fill_rects.is_empty());
        assert!(m.glyphs.len() >= 3);
    }

    #[test]
    fn sum_with_limits() {
        let m = layout(r"\sum_{i=1}^n i");
        assert!(m.width.0 > 0);
        assert!(m.glyphs.len() >= 4);
    }

    #[test]
    fn unknown_command_is_unsupported() {
        let err = layout_tex(r"\unknown", &font(), Pt(12000), MathStyle::Display).unwrap_err();
        assert!(matches!(err, MathError::Unsupported(_)), "{err}");
        assert!(err.to_string().starts_with("MATH_UNSUPPORTED:"));
    }

    #[test]
    fn unmatched_brace_is_parse() {
        let err = layout_tex("{x", &font(), Pt(12000), MathStyle::Display).unwrap_err();
        assert!(matches!(err, MathError::Parse(_)), "{err}");
        assert!(err.to_string().starts_with("MATH_PARSE:"));
    }

    #[test]
    fn simple_x_works() {
        let m = layout("x");
        assert_eq!(m.glyphs.len(), 1);
    }
}
