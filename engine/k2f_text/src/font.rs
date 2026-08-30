use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, OnceLock};

pub struct FontLibrary {
    fonts: HashMap<String, Font>,
    /// Package font paths in deterministic (BTreeMap) order.
    path_order: Vec<String>,
}

impl Default for FontLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl FontLibrary {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            path_order: Vec::new(),
        }
    }

    pub fn set_path_order(&mut self, paths: Vec<String>) {
        self.path_order = paths;
    }

    pub fn path_order(&self) -> &[String] {
        &self.path_order
    }

    /// Primary family key first, then other embedded package paths (deterministic).
    pub fn resolve_font_key(&self, primary_key: &str, ch: char) -> Option<String> {
        if crate::coverage::is_default_ignorable(ch) {
            return Some(primary_key.to_string());
        }
        if self
            .get_font(primary_key)
            .is_some_and(|font| font.covers(ch))
        {
            return Some(primary_key.to_string());
        }
        for path in &self.path_order {
            let stem = match Path::new(path).file_stem() {
                Some(s) => s.to_string_lossy(),
                None => continue,
            };
            if stem.is_empty() || stem == primary_key {
                continue;
            }
            if self
                .get_font(stem.as_ref())
                .is_some_and(|font| font.covers(ch))
            {
                return Some(stem.into_owned());
            }
        }
        None
    }

    pub fn add_font(&mut self, name: &str, data: Vec<u8>) {
        let font = Font::new(data);
        self.fonts.insert(name.to_string(), font.clone());
    }

    /// Register a package font under its path and file stem. Bytes are shared.
    pub fn add_embedded(&mut self, path: &str, data: Vec<u8>) {
        let font = Font::new(data);
        self.fonts.insert(path.to_string(), font.clone());
        if let Some(stem) = std::path::Path::new(path).file_stem() {
            let stem = stem.to_string_lossy();
            if !stem.is_empty() && !self.fonts.contains_key(stem.as_ref()) {
                self.fonts.insert(stem.into_owned(), font.clone());
            }
        }
    }

    pub fn get_font(&self, name: &str) -> Option<&Font> {
        self.fonts.get(name)
    }

    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.fonts.keys()
    }
}

#[derive(Clone)]
pub struct Font {
    pub data: Arc<Vec<u8>>,
    pub(crate) cmap_cache: Arc<OnceLock<HashSet<u32>>>,
}

impl Font {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: Arc::new(data),
            cmap_cache: Arc::new(OnceLock::new()),
        }
    }

    /// Typographic ascender at `font_size`, millipt. Matches paint's
    /// `face.ascender() * font_size / units_per_em` (integer division).
    pub fn typographic_ascender(&self, font_size: k2f_core::Pt) -> k2f_core::Pt {
        let Ok(face) = ttf_parser::Face::parse(self.data.as_ref(), 0) else {
            return k2f_core::Pt::ZERO;
        };
        let upem = face.units_per_em() as i128;
        if upem == 0 {
            return k2f_core::Pt::ZERO;
        }
        k2f_core::Pt((face.ascender() as i128) * font_size.0 / upem)
    }

    /// Glyph ink rectangle at `font_size`, millipt, relative to (left, baseline).
    /// Not the OpenType MATH table — just the outline bbox.
    pub fn glyph_ink(&self, ch: char, font_size: k2f_core::Pt) -> Option<GlyphInk> {
        let face = ttf_parser::Face::parse(self.data.as_ref(), 0).ok()?;
        let upem = face.units_per_em() as i128;
        if upem == 0 {
            return None;
        }
        let gid = face.glyph_index(ch)?;
        let r = face.glyph_bounding_box(gid)?;
        let s = |v: i16| k2f_core::Pt((v as i128) * font_size.0 / upem);
        Some(GlyphInk {
            x_min: s(r.x_min),
            y_min: s(r.y_min),
            x_max: s(r.x_max),
            y_max: s(r.y_max),
        })
    }
}

/// Outline bounding box of a single glyph, millipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphInk {
    pub x_min: k2f_core::Pt,
    pub y_min: k2f_core::Pt,
    pub x_max: k2f_core::Pt,
    pub y_max: k2f_core::Pt,
}

impl GlyphInk {
    pub fn width(self) -> k2f_core::Pt {
        self.x_max - self.x_min
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::Pt;

    #[test]
    fn roboto_paren_ink_is_positive_and_deterministic() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts/Roboto-Regular.ttf");
        let font = Font::new(std::fs::read(&path).unwrap());
        let a = font.glyph_ink('(', Pt(12_000)).expect("Roboto has '('");
        let b = font.glyph_ink('(', Pt(12_000)).expect("Roboto has '('");
        assert_eq!(a, b);
        assert!(a.width().0 > 0);
        assert!(a.y_max.0 > a.y_min.0);
    }
}
