use crate::theme::Theme;
use k2f_text::FontLibrary;
use std::collections::{BTreeMap, HashSet};

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
            check_family(stem, theme, fonts, &format!("font_faces '{family}' {slot}"))?;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FontSyntheticWarning {
    pub family: String,
    pub unused_face: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FaceStyle {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

/// Bold/italic is still synthetic while a same-family Bold/Italic file sits
/// unused. No warning when that file is absent (starter Roboto-Regular) or
/// belongs to another family (a CJK Bold next to Roboto).
pub(crate) fn font_synthetic_warnings(
    theme: &Theme,
    fonts: &FontLibrary,
) -> Vec<FontSyntheticWarning> {
    let faces = embedded_faces(fonts);
    if faces.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let default_role = theme.roles.get("default");
    for style in theme.roles.values() {
        let family = role_family(style, default_role);
        if family.is_empty() {
            continue;
        }
        consider_synthetic(
            family,
            style.bold,
            style.italic,
            theme,
            &faces,
            &mut seen,
            &mut out,
        );
        for variant in style.variants.values() {
            let Some(patch) = &variant.text_overrides else {
                continue;
            };
            let bold = patch.bold.unwrap_or(style.bold);
            let italic = patch.italic.unwrap_or(style.italic);
            let patched = patch
                .font_family
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(family);
            consider_synthetic(patched, bold, italic, theme, &faces, &mut seen, &mut out);
        }
    }
    let default_family = default_role.map(|d| d.font_family.as_str()).unwrap_or("");
    for styles in theme.modifiers.styles.values() {
        for patch in styles.values() {
            let bold = patch.bold.unwrap_or(false);
            let italic = patch.italic.unwrap_or(false);
            if !bold && !italic {
                continue;
            }
            let family = patch
                .font_family
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(default_family);
            consider_synthetic(family, bold, italic, theme, &faces, &mut seen, &mut out);
        }
    }
    out.sort_by(|a, b| (&a.family, &a.unused_face).cmp(&(&b.family, &b.unused_face)));
    out
}

fn role_family<'a>(
    style: &'a crate::theme::RoleStyle,
    default: Option<&'a crate::theme::RoleStyle>,
) -> &'a str {
    if !style.font_family.is_empty() {
        style.font_family.as_str()
    } else {
        default.map(|d| d.font_family.as_str()).unwrap_or("")
    }
}

fn consider_synthetic(
    family: &str,
    bold: bool,
    italic: bool,
    theme: &Theme,
    faces: &[NamedFace],
    seen: &mut HashSet<(String, String)>,
    out: &mut Vec<FontSyntheticWarning>,
) {
    if !bold && !italic {
        return;
    }
    let resolved = crate::style::resolve_face_key(family, bold, italic, theme);
    if face_satisfies(&resolved, bold, italic) {
        return;
    }
    let fam = face_family(&resolved);
    for wanted in wanted_styles(bold, italic) {
        let Some(unused) = faces
            .iter()
            .find(|face| face.family == fam && face.style == wanted)
        else {
            continue;
        };
        if unused.stem == resolved {
            continue;
        }
        if seen.insert((fam.clone(), unused.stem.clone())) {
            out.push(FontSyntheticWarning {
                family: fam,
                unused_face: unused.stem.clone(),
            });
        }
        break;
    }
}

fn wanted_styles(bold: bool, italic: bool) -> Vec<FaceStyle> {
    match (bold, italic) {
        (true, true) => vec![FaceStyle::BoldItalic, FaceStyle::Bold, FaceStyle::Italic],
        (true, false) => vec![FaceStyle::Bold, FaceStyle::BoldItalic],
        (false, true) => vec![FaceStyle::Italic, FaceStyle::BoldItalic],
        (false, false) => Vec::new(),
    }
}

fn face_satisfies(stem: &str, bold: bool, italic: bool) -> bool {
    match classify_face(stem).map(|(_, style)| style) {
        Some(FaceStyle::BoldItalic) => bold || italic,
        Some(FaceStyle::Bold) => bold && !italic,
        Some(FaceStyle::Italic) => italic && !bold,
        _ => false,
    }
}

fn face_family(stem: &str) -> String {
    classify_face(stem)
        .map(|(family, _)| family)
        .unwrap_or_else(|| stem.to_string())
}

struct NamedFace {
    stem: String,
    family: String,
    style: FaceStyle,
}

fn embedded_faces(fonts: &FontLibrary) -> Vec<NamedFace> {
    let mut out = Vec::new();
    for path in fonts.path_order() {
        if !k2f_core::is_font_face_path(path) {
            continue;
        }
        let Some(stem) = std::path::Path::new(path).file_stem() else {
            continue;
        };
        let stem = stem.to_string_lossy();
        if stem.is_empty() || stem.eq_ignore_ascii_case("default") {
            continue;
        }
        let Some((family, style)) = classify_face(&stem) else {
            continue;
        };
        if style == FaceStyle::Regular {
            continue;
        }
        out.push(NamedFace {
            stem: stem.into_owned(),
            family,
            style,
        });
    }
    out
}

/// Filename suffix only. Medium / Light / SemiBold are not guessed.
fn classify_face(stem: &str) -> Option<(String, FaceStyle)> {
    const SUFFIXES: &[(&str, FaceStyle)] = &[
        ("-BoldItalic", FaceStyle::BoldItalic),
        ("-Bold-Italic", FaceStyle::BoldItalic),
        ("-BoldOblique", FaceStyle::BoldItalic),
        ("-Bold-Oblique", FaceStyle::BoldItalic),
        ("-Italic", FaceStyle::Italic),
        ("-Oblique", FaceStyle::Italic),
        ("-Bold", FaceStyle::Bold),
        ("-Regular", FaceStyle::Regular),
        ("-Book", FaceStyle::Regular),
        ("-Roman", FaceStyle::Regular),
    ];
    let lower = stem.to_ascii_lowercase();
    for (suffix, style) in SUFFIXES {
        let suffix_lower = suffix.to_ascii_lowercase();
        if lower.ends_with(&suffix_lower) && stem.len() > suffix.len() {
            let family = stem[..stem.len() - suffix.len()].to_string();
            if !family.is_empty() {
                return Some((family, *style));
            }
        }
    }
    None
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
        let json = crate::compile_chunk_with_fonts(content, theme, &fonts, None).expect("compile");
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
        let json = crate::compile_chunk_with_fonts(content, theme, &fonts, None).expect("compile");
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
        let json = crate::compile_chunk_with_fonts(content, theme, &fonts, None).expect("compile");
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

    fn roboto_bytes() -> Vec<u8> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/fonts/Roboto-Regular.ttf");
        std::fs::read(&path).expect("Roboto-Regular.ttf")
    }

    fn theme_with(roles_and_rest: &str) -> Theme {
        serde_json::from_str(roles_and_rest).unwrap()
    }

    #[test]
    fn synthetic_bold_warns_when_same_family_bold_is_unused() {
        let bytes = roboto_bytes();
        let mut fonts = BTreeMap::new();
        fonts.insert(
            "assets/fonts/OpenSauceSans-Regular.ttf".into(),
            bytes.clone(),
        );
        fonts.insert("assets/fonts/OpenSauceSans-Bold.ttf".into(), bytes);
        let lib = load_font_library(&fonts).unwrap();
        let theme = theme_with(
            r#"{
              "palette": {},
              "roles": {
                "h1": {
                  "font_family": "OpenSauceSans-Regular",
                  "font_size": 24000,
                  "line_height_mult": 1200,
                  "color": "black",
                  "bold": true
                }
              }
            }"#,
        );
        let warnings = font_synthetic_warnings(&theme, &lib);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert_eq!(warnings[0].family, "OpenSauceSans");
        assert_eq!(warnings[0].unused_face, "OpenSauceSans-Bold");
    }

    #[test]
    fn synthetic_bold_silent_for_starter_and_other_family() {
        let bytes = roboto_bytes();
        let mut only = BTreeMap::new();
        only.insert("assets/fonts/Roboto-Regular.ttf".into(), bytes.clone());
        let theme = theme_with(
            r#"{
              "palette": {},
              "modifiers": { "styles": { "emphasis": { "strong": { "bold": true } } } },
              "roles": {
                "default": {
                  "font_family": "Roboto-Regular",
                  "font_size": 12000,
                  "line_height_mult": 1400,
                  "color": "black"
                },
                "h1": {
                  "font_family": "Roboto-Regular",
                  "font_size": 24000,
                  "line_height_mult": 1200,
                  "color": "black",
                  "bold": true
                }
              }
            }"#,
        );
        assert!(font_synthetic_warnings(&theme, &load_font_library(&only).unwrap()).is_empty());

        let mut mixed = BTreeMap::new();
        mixed.insert("assets/fonts/Roboto-Regular.ttf".into(), bytes.clone());
        mixed.insert("assets/fonts/NotoSansJP-Bold.otf".into(), bytes);
        assert!(
            font_synthetic_warnings(&theme, &load_font_library(&mixed).unwrap()).is_empty(),
            "a CJK Bold face is not Roboto's bold"
        );
    }

    #[test]
    fn synthetic_bold_silent_when_font_faces_selects_it() {
        let bytes = roboto_bytes();
        let mut fonts = BTreeMap::new();
        fonts.insert(
            "assets/fonts/OpenSauceSans-Regular.ttf".into(),
            bytes.clone(),
        );
        fonts.insert("assets/fonts/OpenSauceSans-Bold.ttf".into(), bytes);
        let lib = load_font_library(&fonts).unwrap();
        let theme = theme_with(
            r#"{
              "palette": {},
              "font_faces": {
                "OpenSauceSans": {
                  "regular": "OpenSauceSans-Regular",
                  "bold": "OpenSauceSans-Bold"
                }
              },
              "roles": {
                "h1": {
                  "font_family": "OpenSauceSans-Regular",
                  "font_size": 24000,
                  "line_height_mult": 1200,
                  "color": "black",
                  "bold": true
                }
              }
            }"#,
        );
        assert!(font_synthetic_warnings(&theme, &lib).is_empty());
    }
}
