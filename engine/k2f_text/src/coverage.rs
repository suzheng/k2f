use std::collections::HashSet;

use crate::font::Font;

/// Unicode default-ignorable formatting characters (ZWJ, VS, keycap, etc.).
pub fn is_default_ignorable(ch: char) -> bool {
    matches!(
        ch,
        '\u{200C}' | '\u{200D}' | '\u{FE0E}' | '\u{FE0F}' | '\u{20E3}'
    ) || ch as u32 == 0xE0020
        || (0xE0021..=0xE007F).contains(&(ch as u32))
}

/// Line/tab breaks embedded in text nodes (no glyph required).
pub fn is_layout_whitespace(ch: char) -> bool {
    matches!(ch, '\n' | '\r' | '\t')
}

pub fn is_significant(ch: char) -> bool {
    !is_default_ignorable(ch) && !is_layout_whitespace(ch)
}

impl Font {
    pub(crate) fn cmap(&self) -> &HashSet<u32> {
        self.cmap_cache.get_or_init(|| build_cmap(&self.data))
    }

    pub fn covers(&self, ch: char) -> bool {
        if is_default_ignorable(ch) || is_layout_whitespace(ch) {
            return true;
        }
        self.cmap().contains(&(ch as u32))
    }
}

fn build_cmap(data: &[u8]) -> HashSet<u32> {
    let face = match ttf_parser::Face::parse(data, 0) {
        Ok(f) => f,
        Err(_) => return HashSet::new(),
    };
    let mut out = HashSet::new();
    for cp in 0x20..=0x10FFFF_u32 {
        if let Some(ch) = char::from_u32(cp) {
            if face.glyph_index(ch).is_some() {
                out.insert(cp);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ignorable_variation_selector() {
        assert!(is_default_ignorable('\u{FE0F}'));
        assert!(!is_default_ignorable('A'));
    }
}
