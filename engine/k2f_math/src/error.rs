use std::fmt;

/// Every math failure is fatal: the compiler must never silently drop a formula.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathError {
    /// Malformed TeX: unbalanced braces, missing arguments, dangling scripts.
    Parse(String),
    /// Well-formed TeX outside the v1 whitelist.
    Unsupported(String),
    /// The math font has no glyph for a required code point.
    MissingGlyph(String),
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MathError::Parse(msg) => write!(f, "MATH_PARSE: {msg}"),
            MathError::Unsupported(msg) => write!(f, "MATH_UNSUPPORTED: {msg}"),
            MathError::MissingGlyph(msg) => write!(f, "MATH_MISSING_GLYPH: {msg}"),
        }
    }
}

impl std::error::Error for MathError {}

/// `TextShaper` reports failures as strings; keep the missing-glyph case distinct.
pub(crate) fn from_shape_error(err: String) -> MathError {
    if err.contains("MISSING_GLYPH") {
        MathError::MissingGlyph(err)
    } else {
        MathError::Unsupported(format!("font shaping failed: {err}"))
    }
}
