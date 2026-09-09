use crate::coverage::{is_default_ignorable, is_layout_whitespace, is_significant};
use crate::font::FontLibrary;

/// One substring shaped with a single embedded font key (family stem).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontRun {
    pub font_key: String,
    pub text: String,
}

/// Split `text` into runs by embedded-font coverage. Primary family is tried first.
pub fn split_by_coverage(
    text: &str,
    primary_key: &str,
    fonts: &FontLibrary,
) -> Result<Vec<FontRun>, String> {
    if text.is_empty() {
        return Ok(vec![]);
    }

    let mut out: Vec<FontRun> = Vec::new();
    let mut missing: Vec<(u32, char)> = Vec::new();
    let mut seen_missing = std::collections::BTreeSet::new();

    let mut current_key: Option<String> = None;
    let mut current_text = String::new();

    let flush = |key: &str, buf: &mut String, out: &mut Vec<FontRun>| {
        if buf.is_empty() {
            return;
        }
        out.push(FontRun {
            font_key: key.to_string(),
            text: std::mem::take(buf),
        });
    };

    for ch in text.chars() {
        if is_significant(ch) && fonts.resolve_font_key(primary_key, ch).is_none() {
            if seen_missing.insert(ch as u32) {
                missing.push((ch as u32, ch));
            }
        }

        let key = if is_default_ignorable(ch) || is_layout_whitespace(ch) {
            current_key
                .clone()
                .unwrap_or_else(|| primary_key.to_string())
        } else if let Some(k) = fonts.resolve_font_key(primary_key, ch) {
            k
        } else {
            current_key
                .clone()
                .unwrap_or_else(|| primary_key.to_string())
        };

        if current_key.as_deref() != Some(key.as_str()) {
            if let Some(prev) = current_key.take() {
                flush(&prev, &mut current_text, &mut out);
            }
            current_key = Some(key);
        }
        current_text.push(ch);
    }

    if let Some(key) = current_key {
        flush(&key, &mut current_text, &mut out);
    }

    if !missing.is_empty() {
        return Err(missing_glyph_error(text, &missing));
    }

    Ok(out)
}

pub(crate) fn missing_glyph_error(text: &str, missing: &[(u32, char)]) -> String {
    let list = missing
        .iter()
        .map(|(cp, ch)| format!("U+{cp:04X} {ch:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "FONT_MISSING_GLYPH: no embedded face covers {list} (text {:?}); add a covering TTF/OTF (--add-font); no OS fallback",
        text.chars().take(16).collect::<String>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::Font;
    use std::path::PathBuf;

    fn load(name: &str) -> Font {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts")
            .join(name);
        Font::new(std::fs::read(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}")))
    }

    fn two_font_lib(primary_path: &str, primary: Font, emoji: Font) -> FontLibrary {
        let mut lib = FontLibrary::new();
        lib.set_path_order(vec![
            primary_path.to_string(),
            "assets/fonts/NotoEmoji-Regular.ttf".to_string(),
        ]);
        lib.add_embedded(primary_path, (*primary.data).clone());
        lib.add_embedded("assets/fonts/NotoEmoji-Regular.ttf", (*emoji.data).clone());
        lib
    }

    #[test]
    fn roboto_alone_rejects_checkmark_emoji() {
        let roboto = load("Roboto-Regular.ttf");
        let mut lib = FontLibrary::new();
        lib.add_embedded("assets/fonts/Roboto-Regular.ttf", (*roboto.data).clone());
        let err = split_by_coverage("OK ✅", "Roboto-Regular", &lib).unwrap_err();
        assert!(err.contains("FONT_MISSING_GLYPH"), "{err}");
        assert!(err.contains("U+2705"), "{err}");
        assert!(err.contains("--add-font"), "{err}");
        assert!(err.contains("no OS fallback"), "{err}");
    }

    #[test]
    fn roboto_with_emoji_splits_runs() {
        let roboto = load("Roboto-Regular.ttf");
        let emoji = load("NotoEmoji-Regular.ttf");
        let lib = two_font_lib("assets/fonts/Roboto-Regular.ttf", roboto, emoji);
        let runs = split_by_coverage("OK ✅", "Roboto-Regular", &lib).unwrap();
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].font_key, "Roboto-Regular");
        assert_eq!(runs[0].text, "OK ");
        assert_eq!(runs[1].font_key, "NotoEmoji-Regular");
        assert_eq!(runs[1].text, "✅");
    }

    #[test]
    fn cjk_still_fails_without_sc_font() {
        let roboto = load("Roboto-Regular.ttf");
        let emoji = load("NotoEmoji-Regular.ttf");
        let lib = two_font_lib("assets/fonts/Roboto-Regular.ttf", roboto, emoji);
        let err = split_by_coverage("Hi合", "Roboto-Regular", &lib).unwrap_err();
        assert!(err.contains("U+5408"), "{err}");
    }
}
