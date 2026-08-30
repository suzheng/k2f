use crate::theme::Theme;
use k2f_text::FontLibrary;
use std::collections::BTreeMap;

const GENERIC_FAMILIES: &[&str] = &[
    "serif",
    "sans-serif",
    "monospace",
    "cursive",
    "fantasy",
    "system-ui",
    "ui-serif",
    "ui-sans-serif",
    "ui-monospace",
    "ui-rounded",
    "emoji",
    "math",
    "fangsong",
];

pub fn is_generic_family(name: &str) -> bool {
    GENERIC_FAMILIES
        .iter()
        .any(|g| g.eq_ignore_ascii_case(name))
}

pub fn load_font_library(fonts: &BTreeMap<String, Vec<u8>>) -> Result<FontLibrary, String> {
    if fonts.is_empty() {
        return Err("FONT_MISSING: package has no embedded fonts under assets/fonts/".to_string());
    }
    let mut lib = FontLibrary::new();
    lib.set_path_order(fonts.keys().cloned().collect());
    for (path, bytes) in fonts {
        lib.add_embedded(path, bytes.clone());
    }
    // One embedded file may be called "default". Several files must be named in the theme;
    // picking BTreeMap::first would silently swap fonts when a second face is added.
    if lib.get_font("default").is_none() && fonts.len() == 1 {
        let (_, bytes) = fonts.iter().next().unwrap();
        lib.add_font("default", bytes.clone());
    }
    Ok(lib)
}

pub fn validate_theme_fonts(theme: &Theme, fonts: &FontLibrary) -> Result<(), String> {
    for (role, style) in &theme.roles {
        check_family(&style.font_family, theme, fonts, &format!("role '{role}'"))?;
        for (variant, vs) in &style.variants {
            if let Some(patch) = &vs.text_overrides {
                if let Some(ff) = &patch.font_family {
                    check_family(
                        ff,
                        theme,
                        fonts,
                        &format!("role '{role}' variant '{variant}'"),
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn check_family(
    family: &str,
    theme: &Theme,
    fonts: &FontLibrary,
    where_: &str,
) -> Result<(), String> {
    if is_generic_family(family) {
        return Err(format!(
            "FONT_MISSING: {where_} uses CSS generic family '{family}'; embed a TTF/OTF and name it in the theme"
        ));
    }
    let key = crate::style::resolve_font_family_key(family, theme);
    if is_generic_family(&key) {
        return Err(format!(
            "FONT_MISSING: {where_} alias '{family}' resolves to generic family '{key}'"
        ));
    }
    if fonts.get_font(&key).is_none() {
        return Err(format!(
            "FONT_MISSING: {where_} font '{family}' (key '{key}') is not embedded; key must match an assets/fonts/ file stem (single font also registers as 'default'; with two+ fonts set font_aliases)"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn rejects_sans_serif() {
        assert!(is_generic_family("sans-serif"));
        assert!(is_generic_family("Sans-Serif"));
        assert!(!is_generic_family("Roboto"));
    }

    #[test]
    fn single_font_is_also_default() {
        let mut fonts = BTreeMap::new();
        fonts.insert("assets/fonts/Roboto-Regular.ttf".into(), vec![0, 1, 2]);
        let lib = load_font_library(&fonts).unwrap();
        assert!(lib.get_font("default").is_some());
        assert!(lib.get_font("Roboto-Regular").is_some());
    }

    #[test]
    fn two_fonts_do_not_invent_a_default() {
        let mut fonts = BTreeMap::new();
        fonts.insert("assets/fonts/A.ttf".into(), vec![1]);
        fonts.insert("assets/fonts/B.ttf".into(), vec![2]);
        let lib = load_font_library(&fonts).unwrap();
        assert!(lib.get_font("default").is_none());
        assert!(lib.get_font("A").is_some());
        assert!(lib.get_font("B").is_some());
    }
}
