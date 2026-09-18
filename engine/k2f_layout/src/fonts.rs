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
    let faces: Vec<(&String, &Vec<u8>)> = fonts
        .iter()
        .filter(|(path, _)| k2f_core::is_font_face_path(path))
        .collect();
    if faces.is_empty() {
        return Err("FONT_MISSING: package has no embedded fonts under assets/fonts/".to_string());
    }
    let mut lib = FontLibrary::new();
    lib.set_path_order(faces.iter().map(|(path, _)| (*path).clone()).collect());
    for (path, bytes) in &faces {
        lib.add_embedded(path, (*bytes).clone());
    }
    // One embedded face may be called "default". Several files must be named in the theme;
    // picking BTreeMap::first would silently swap fonts when a second face is added.
    // License sidecars under assets/fonts/ do not count toward this.
    if lib.get_font("default").is_none() && faces.len() == 1 {
        let (_, bytes) = faces[0];
        lib.add_font("default", bytes.clone());
    }
    Ok(lib)
}

pub fn validate_theme_fonts(theme: &Theme, fonts: &FontLibrary) -> Result<(), String> {
    crate::style::validate_role_text_fields(theme)?;
    for (family, slots) in &theme.font_faces {
        for (slot, stem) in [
            ("regular", slots.regular.as_deref()),
            ("bold", slots.bold.as_deref()),
            ("italic", slots.italic.as_deref()),
            ("bold_italic", slots.bold_italic.as_deref()),
        ] {
            let Some(stem) = stem.map(str::trim).filter(|s| !s.is_empty()) else {
                continue;
            };
            check_family(
                stem,
                theme,
                fonts,
                &format!("font_faces '{family}' {slot}"),
            )?;
        }
    }
    let default = theme.roles.get("default");
    for (role, style) in &theme.roles {
        let family = if style.font_family.is_empty() {
            default.map(|d| d.font_family.as_str()).unwrap_or("")
        } else {
            style.font_family.as_str()
        };
        if family.is_empty() {
            continue;
        }
        check_family(family, theme, fonts, &format!("role '{role}'"))?;
        check_face(
            family,
            style.bold,
            style.italic,
            theme,
            fonts,
            &format!("role '{role}'"),
        )?;
        for (variant, vs) in &style.variants {
            if let Some(patch) = &vs.text_overrides {
                let ff = patch.font_family.as_deref().unwrap_or(family);
                let bold = patch.bold.unwrap_or(style.bold);
                let italic = patch.italic.unwrap_or(style.italic);
                if patch.font_family.is_some() {
                    check_family(
                        ff,
                        theme,
                        fonts,
                        &format!("role '{role}' variant '{variant}'"),
                    )?;
                }
                check_face(
                    ff,
                    bold,
                    italic,
                    theme,
                    fonts,
                    &format!("role '{role}' variant '{variant}'"),
                )?;
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
    // A font_faces group key is not itself a file stem. Slots are checked above.
    if theme.font_faces.contains_key(family) || theme.font_faces.contains_key(&key) {
        return Ok(());
    }
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

fn check_face(
    family: &str,
    bold: bool,
    italic: bool,
    theme: &Theme,
    fonts: &FontLibrary,
    where_: &str,
) -> Result<(), String> {
    let key = crate::style::resolve_face_key(family, bold, italic, theme);
    if is_generic_family(&key) {
        return Err(format!(
            "FONT_MISSING: {where_} face '{family}' resolves to generic family '{key}'"
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

    #[test]
    fn license_sidecar_does_not_block_single_font_default() {
        let mut fonts = BTreeMap::new();
        fonts.insert("assets/fonts/Roboto-Regular.ttf".into(), vec![0, 1, 2]);
        fonts.insert(
            "assets/fonts/licenses/Roboto-Apache.txt".into(),
            b"Apache".to_vec(),
        );
        let lib = load_font_library(&fonts).unwrap();
        assert!(lib.get_font("default").is_some());
        assert!(lib.get_font("Roboto-Regular").is_some());
        assert!(lib.get_font("Roboto-Apache").is_none());
    }

    #[test]
    fn licenses_only_is_font_missing() {
        let mut fonts = BTreeMap::new();
        fonts.insert(
            "assets/fonts/licenses/Roboto-Apache.txt".into(),
            b"Apache".to_vec(),
        );
        let err = match load_font_library(&fonts) {
            Ok(_) => panic!("expected FONT_MISSING"),
            Err(e) => e,
        };
        assert!(err.contains("FONT_MISSING"), "{err}");
    }

    #[test]
    fn declared_bold_slot_must_be_embedded() {
        let mut fonts = BTreeMap::new();
        fonts.insert("assets/fonts/Roboto-Regular.ttf".into(), vec![0, 1, 2]);
        let lib = load_font_library(&fonts).unwrap();
        let theme: Theme = serde_json::from_str(
            r#"{
      "palette": {},
      "font_aliases": {
        "Roboto-Regular": "Roboto-Regular",
        "Roboto-Bold": "Roboto-Bold"
      },
      "font_faces": {
        "Roboto-Regular": { "regular": "Roboto-Regular", "bold": "Roboto-Bold" }
      },
      "roles": {
        "default": {
          "font_family": "Roboto-Regular",
          "font_size": 12000,
          "line_height_mult": 1400,
          "color": "black"
        }
      }
    }"#,
        )
        .unwrap();
        let err = validate_theme_fonts(&theme, &lib).unwrap_err();
        assert!(err.contains("FONT_MISSING"), "{err}");
        assert!(err.contains("Roboto-Bold"), "{err}");
    }

    #[test]
    fn bold_role_lock_uses_bold_stem() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts/Roboto-Regular.ttf");
        let bytes = std::fs::read(&path).expect("Roboto-Regular.ttf");
        let mut fonts = BTreeMap::new();
        fonts.insert("assets/fonts/Roboto-Regular.ttf".into(), bytes.clone());
        fonts.insert("assets/fonts/Roboto-Bold.ttf".into(), bytes);
        let theme = r##"{
            "palette": { "ink": "#000000" },
            "font_aliases": {
                "Roboto": "Roboto-Regular",
                "Roboto-Regular": "Roboto-Regular",
                "Roboto-Bold": "Roboto-Bold"
            },
            "font_faces": {
                "Roboto": { "regular": "Roboto-Regular", "bold": "Roboto-Bold" }
            },
            "roles": {
                "default": {
                    "font_family": "Roboto",
                    "font_size": 12000,
                    "line_height_mult": 1400,
                    "color": "ink"
                },
                "h1": { "font_family": "Roboto", "font_size": 24000, "bold": true },
                "body": { "font_family": "Roboto" }
            }
        }"##;
        let content = r#"{
            "id": "root",
            "role": "body",
            "content": {
                "type": "container",
                "value": {
                    "children": [
                        { "id": "root.title", "role": "h1", "content": { "type": "text", "value": "Hello" } },
                        { "id": "root.p", "role": "body", "content": { "type": "text", "value": "World" } }
                    ]
                }
            }
        }"#;
        let json = crate::compile_chunk_with_fonts(content, theme, &fonts, None)
            .expect("compile");
        let lock: k2f_core::LockFile = serde_json::from_str(&json).unwrap();
        let mut families = std::collections::BTreeMap::<String, (String, bool)>::new();
        for page in &lock.render_plan.pages {
            for op in &page.ops {
                if let k2f_core::PaintOp::DrawText { node_id, runs, .. } = op {
                    if let Some(run) = runs.first() {
                        families.insert(
                            node_id.clone(),
                            (run.style.font_family.clone(), run.style.bold),
                        );
                    }
                }
            }
        }
        assert_eq!(
            families.get("root.title"),
            Some(&("Roboto-Bold".into(), true))
        );
        assert_eq!(
            families.get("root.p"),
            Some(&("Roboto-Regular".into(), false))
        );
    }

    #[test]
    fn emphasis_modifier_uses_bold_stem() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts/Roboto-Regular.ttf");
        let bytes = std::fs::read(&path).expect("Roboto-Regular.ttf");
        let mut fonts = BTreeMap::new();
        fonts.insert("assets/fonts/Roboto-Regular.ttf".into(), bytes.clone());
        fonts.insert("assets/fonts/Roboto-Bold.ttf".into(), bytes);
        let theme = r##"{
            "palette": { "ink": "#000000" },
            "font_aliases": {
                "Roboto": "Roboto-Regular",
                "Roboto-Regular": "Roboto-Regular",
                "Roboto-Bold": "Roboto-Bold"
            },
            "font_faces": {
                "Roboto": { "regular": "Roboto-Regular", "bold": "Roboto-Bold" }
            },
            "modifiers": {
                "styles": { "emphasis": { "strong": { "bold": true } } }
            },
            "roles": {
                "default": {
                    "font_family": "Roboto",
                    "font_size": 12000,
                    "line_height_mult": 1400,
                    "color": "ink"
                },
                "body": { "font_family": "Roboto" }
            }
        }"##;
        let content = r#"{
            "id": "root",
            "role": "body",
            "content": { "type": "text", "value": "ab" },
            "modifiers": [{ "range": [0, 1], "type": "emphasis", "intent": "strong" }]
        }"#;
        let json = crate::compile_chunk_with_fonts(content, theme, &fonts, None)
            .expect("compile");
        let lock: k2f_core::LockFile = serde_json::from_str(&json).unwrap();
        let mut run_faces: Vec<(String, bool)> = Vec::new();
        for page in &lock.render_plan.pages {
            for op in &page.ops {
                if let k2f_core::PaintOp::DrawText { runs, .. } = op {
                    for run in runs {
                        run_faces.push((run.style.font_family.clone(), run.style.bold));
                    }
                }
            }
        }
        assert!(
            run_faces.iter().any(|(f, b)| f == "Roboto-Bold" && *b),
            "expected a bold stem run, got {run_faces:?}"
        );
        assert!(
            run_faces.iter().any(|(f, b)| f == "Roboto-Regular" && !*b),
            "expected a regular stem run, got {run_faces:?}"
        );
    }

    #[test]
    fn family_name_without_alias_lock_uses_bold_stem() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts/Roboto-Regular.ttf");
        let bytes = std::fs::read(&path).expect("Roboto-Regular.ttf");
        let mut fonts = BTreeMap::new();
        fonts.insert("assets/fonts/Roboto-Regular.ttf".into(), bytes.clone());
        fonts.insert("assets/fonts/Roboto-Bold.ttf".into(), bytes);
        let theme = r##"{
            "palette": { "ink": "#000000" },
            "font_faces": {
                "Roboto": { "regular": "Roboto-Regular", "bold": "Roboto-Bold" }
            },
            "roles": {
                "default": {
                    "font_family": "Roboto",
                    "font_size": 12000,
                    "line_height_mult": 1400,
                    "color": "ink"
                },
                "h1": { "font_family": "Roboto", "font_size": 24000, "bold": true }
            }
        }"##;
        let content = r#"{
            "id": "root",
            "role": "h1",
            "content": { "type": "text", "value": "Hello" }
        }"#;
        let json = crate::compile_chunk_with_fonts(content, theme, &fonts, None)
            .expect("compile");
        let lock: k2f_core::LockFile = serde_json::from_str(&json).unwrap();
        let mut found = false;
        for page in &lock.render_plan.pages {
            for op in &page.ops {
                if let k2f_core::PaintOp::DrawText { runs, .. } = op {
                    if let Some(run) = runs.first() {
                        assert_eq!(run.style.font_family, "Roboto-Bold");
                        assert!(run.style.bold);
                        found = true;
                    }
                }
            }
        }
        assert!(found, "expected a text run");
    }
}
