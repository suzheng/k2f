//! v2: stretchy `\left\right` and `matrix`/`align`/`cases`.

use super::*;
use k2f_core::Pt;
use k2f_text::Font;

fn font() -> Font {
    Font::new(include_bytes!("../../../assets/fonts/NotoSansMath-Regular.ttf").to_vec())
}

fn layout(tex: &str) -> MathLayout {
    layout_tex(tex, &font(), Pt(12_000), MathStyle::Display).expect(tex)
}

fn err(tex: &str) -> MathError {
    layout_tex(tex, &font(), Pt(12_000), MathStyle::Display).unwrap_err()
}

#[test]
fn left_right_paren_around_atom() {
    let m = layout(r"\left(x\right)");
    assert!(m.glyphs.len() >= 3, "got {} glyphs", m.glyphs.len());
    assert!(m.width.0 > 0);
    let again = layout(r"\left(x\right)");
    assert_eq!(m, again);
}

#[test]
fn left_right_grows_for_fraction() {
    let atom = layout(r"\left(x\right)");
    let frac = layout(r"\left(\frac{1}{2}\right)");
    assert!(
        frac.height.0 > atom.height.0,
        "frac={:?} atom={:?}",
        frac.height,
        atom.height
    );
    assert!(
        frac.glyphs.len() > atom.glyphs.len(),
        "assembled parens should add pieces: frac={} atom={}",
        frac.glyphs.len(),
        atom.glyphs.len()
    );
}

#[test]
fn left_right_bar_uses_fill_rect() {
    let m = layout(r"\left|\frac{1}{2}\right|");
    assert!(
        m.fill_rects.len() >= 3,
        "fraction rule + two stretchy bars, got {:?}",
        m.fill_rects
    );
}

#[test]
fn left_dot_is_null_delimiter() {
    let inner = layout(r"x");
    let m = layout(r"\left.x\right)");
    assert!(m.width.0 > inner.width.0);
    assert!(m.glyphs.len() >= 2);
}

#[test]
fn unmatched_left_is_parse() {
    let e = err(r"\left(x");
    assert!(matches!(e, MathError::Parse(_)), "{e}");
}

#[test]
fn matrix_two_by_two() {
    let m = layout(r"\begin{matrix} a & b \\ c & d \end{matrix}");
    assert!(m.glyphs.len() >= 4, "got {} glyphs", m.glyphs.len());
    let row = layout("ab");
    assert!(
        m.width.0 > row.width.0,
        "matrix should be wider than a single row of ab"
    );
    assert_eq!(m, layout(r"\begin{matrix} a & b \\ c & d \end{matrix}"));
}

#[test]
fn pmatrix_wraps_with_parens() {
    let inner = layout(r"\begin{matrix} a \\ b \end{matrix}");
    let wrapped = layout(r"\begin{pmatrix} a \\ b \end{pmatrix}");
    assert!(wrapped.width.0 > inner.width.0);
    assert!(wrapped.glyphs.len() > inner.glyphs.len());
}

#[test]
fn align_has_two_sides() {
    let m = layout(r"\begin{align} a&=b \\ c&=d \end{align}");
    assert!(m.glyphs.len() >= 4);
}

#[test]
fn cases_has_left_brace() {
    let inner = layout(r"\begin{matrix} x \\ y \end{matrix}");
    let cases = layout(r"\begin{cases} x \\ y \end{cases}");
    assert!(cases.width.0 > inner.width.0);
    assert!(cases.glyphs.len() > inner.glyphs.len());
}

#[test]
fn unknown_environment_is_unsupported() {
    let e = err(r"\begin{foo} a \end{foo}");
    assert!(e.to_string().starts_with("MATH_UNSUPPORTED:"), "{e}");
}

#[test]
fn amp_outside_env_is_parse() {
    let e = err("a&b");
    assert!(e.to_string().starts_with("MATH_PARSE:"), "{e}");
}
