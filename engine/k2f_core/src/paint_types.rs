use crate::Pt;
use serde::{Deserialize, Serialize};

/// A concrete, resolved text style for paint.
///
/// This is intentionally a subset of theme/style authoring. It exists so a viewer can
/// execute a lock file without re-running style resolution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextPaintStyle {
    pub font_family: String,
    pub font_size: Pt,
    pub color: String,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub strikethrough: bool,
    #[serde(default)]
    pub underline: bool,
}

/// A contiguous glyph slice (within `GeometryNode.glyphs`) that shares one paint style.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextGlyphRun {
    pub glyph_range: [usize; 2],
    pub style: TextPaintStyle,
}
