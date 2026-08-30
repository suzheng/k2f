use crate::theme::Theme;
use k2f_core::{Modifier, Pt};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextAlign {
    Start,
    Center,
    End,
    Justify,
}

impl Default for TextAlign {
    fn default() -> Self {
        TextAlign::Start
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub font_family: String,
    pub font_size: Pt,
    pub line_height_mult: i128,
    pub color: String,
    pub text_align: TextAlign,
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub letter_spacing: Pt,
    pub first_line_indent: Pt,
    /// Raise amount in millipt (positive = up). Applied as `y_offset -= baseline_shift`
    /// when shaping. Engine-only; not part of theme JSON or lock paint style.
    pub baseline_shift: Pt,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct StylePatch {
    #[serde(default)]
    pub font_family: Option<String>,
    #[serde(default)]
    pub font_size: Option<Pt>,
    #[serde(default)]
    pub line_height_mult: Option<i128>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub text_align: Option<TextAlign>,
    #[serde(default)]
    pub bold: Option<bool>,
    #[serde(default)]
    pub italic: Option<bool>,
    #[serde(default)]
    pub strikethrough: Option<bool>,
    #[serde(default)]
    pub underline: Option<bool>,
    #[serde(default)]
    pub letter_spacing_pt: Option<Pt>,
    #[serde(default)]
    pub first_line_indent_pt: Option<Pt>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            font_family: "default".to_string(),
            font_size: Pt(12000),
            line_height_mult: 1200,
            color: "black".to_string(),
            text_align: TextAlign::Start,
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
            letter_spacing: Pt::ZERO,
            first_line_indent: Pt::ZERO,
            baseline_shift: Pt::ZERO,
        }
    }
}

/// Script geometry for `superscript` / `subscript` (matches math script ratios).
/// Uses the style's current (parent) font size, then shrinks to 7/10 and sets shift.
pub fn apply_script_geometry(style: &mut Style, kind: &str) {
    let parent = style.font_size;
    style.font_size = Pt(parent.0 * 7 / 10);
    style.baseline_shift = match kind {
        // y grows downward: positive shift raises the glyph.
        "superscript" => Pt(parent.0 * 450 / 1000),
        "subscript" => Pt(-(parent.0 * 250 / 1000)),
        _ => Pt::ZERO,
    };
}

/// Resolve a theme-provided font family to a loaded font key.
///
/// This is **not** a fallback mechanism. If the resolved key is not loaded, shaping/measurement must error.
pub fn resolve_font_family_key(font_family: &str, theme: &Theme) -> String {
    theme
        .font_aliases
        .get(font_family)
        .cloned()
        .unwrap_or_else(|| font_family.to_string())
}

/// Base style from role. No modifiers applied here.
pub fn resolve_base_style(role: &str, theme: &Theme) -> Style {
    let base = theme.roles.get(role).or_else(|| theme.roles.get("default"));

    if let Some(rs) = base {
        Style {
            font_family: resolve_font_family_key(&rs.font_family, theme),
            font_size: rs.font_size,
            line_height_mult: rs.line_height_mult,
            color: rs.color.clone(),
            text_align: rs.text_align,
            bold: rs.bold,
            italic: rs.italic,
            letter_spacing: rs.letter_spacing_pt,
            first_line_indent: rs.first_line_indent_pt,
            ..Style::default()
        }
    } else {
        Style::default()
    }
}

pub fn apply_patch(mut style: Style, patch: &StylePatch) -> Style {
    if let Some(v) = &patch.font_family {
        style.font_family = v.clone();
    }
    if let Some(v) = patch.font_size {
        style.font_size = v;
    }
    if let Some(v) = patch.line_height_mult {
        style.line_height_mult = v;
    }
    if let Some(v) = &patch.color {
        style.color = v.clone();
    }
    if let Some(v) = patch.text_align {
        style.text_align = v;
    }
    if let Some(v) = patch.bold {
        style.bold = v;
    }
    if let Some(v) = patch.italic {
        style.italic = v;
    }
    if let Some(v) = patch.strikethrough {
        style.strikethrough = v;
    }
    if let Some(v) = patch.underline {
        style.underline = v;
    }
    if let Some(v) = patch.letter_spacing_pt {
        style.letter_spacing = v;
    }
    if let Some(v) = patch.first_line_indent_pt {
        style.first_line_indent = v;
    }
    style
}

/// Deterministic default precedence list (lowest -> highest).
/// Higher precedence types apply later and therefore override lower precedence when overlapping.
pub fn default_modifier_precedence() -> Vec<String> {
    vec![
        "syntax_highlight".to_string(),
        "emphasis".to_string(),
        "underline".to_string(),
        "strikethrough".to_string(),
        "link".to_string(),
        "math".to_string(),
        "subscript".to_string(),
        "superscript".to_string(),
    ]
}

pub fn modifier_precedence(theme: &Theme) -> Vec<String> {
    if theme.modifiers.precedence.is_empty() {
        default_modifier_precedence()
    } else {
        theme.modifiers.precedence.clone()
    }
}

/// Resolve a modifier into a style patch.
/// Exact theme intent, then theme `default` for that type, then built-ins.
pub fn patch_for_modifier(modifier: &Modifier, theme: &Theme) -> Result<StylePatch, String> {
    if let Some(by_intent) = theme.modifiers.styles.get(&modifier.mod_type) {
        if let Some(patch) = by_intent.get(&modifier.intent) {
            return Ok(patch.clone());
        }
        if modifier.intent != "default" {
            if let Some(patch) = by_intent.get("default") {
                return Ok(patch.clone());
            }
        }
    }

    match modifier.mod_type.as_str() {
        "emphasis" => {
            // Spec example: intent: "critical" (engine decides exact visual)
            // Deterministic default: "critical" => bold=true, "highlight" => underline=true
            let intent = modifier.intent.as_str();
            let mut patch = StylePatch::default();
            match intent {
                "critical" | "strong" | "bold" => patch.bold = Some(true),
                "italic" | "emphasis" => patch.italic = Some(true),
                _ => {}
            }
            Ok(patch)
        }
        "link" => Ok(StylePatch {
            underline: Some(true),
            color: Some("accent".to_string()),
            ..StylePatch::default()
        }),
        "underline" => Ok(StylePatch {
            underline: Some(true),
            ..StylePatch::default()
        }),
        "strikethrough" => Ok(StylePatch {
            strikethrough: Some(true),
            ..StylePatch::default()
        }),
        "superscript" | "subscript" => Ok(StylePatch {
            ..StylePatch::default()
        }),
        "math" | "syntax_highlight" => Ok(StylePatch::default()),
        other => Err(format!("Unknown modifier type '{other}'")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ModifierTheme, RoleStyle, Theme};
    use std::collections::HashMap;

    fn sample_theme() -> Theme {
        let mut roles = HashMap::new();
        roles.insert(
            "body".to_string(),
            RoleStyle {
                font_family: "default".to_string(),
                font_size: Pt(12000),
                line_height_mult: 1400,
                color: "black".to_string(),
                text_align: TextAlign::Start,
                self_align: None,
                box_decoration: None,
                list_style: None,
                bold: false,
                italic: false,
                letter_spacing_pt: Pt::ZERO,
            first_line_indent_pt: Pt::ZERO,
                variants: HashMap::new(),
            },
        );
        roles.insert(
            "header".to_string(),
            RoleStyle {
                font_family: "default".to_string(),
                font_size: Pt(24000),
                line_height_mult: 1100,
                color: "black".to_string(),
                text_align: TextAlign::Start,
                self_align: None,
                box_decoration: None,
                list_style: None,
                bold: false,
                italic: false,
                letter_spacing_pt: Pt::ZERO,
            first_line_indent_pt: Pt::ZERO,
                variants: HashMap::new(),
            },
        );

        Theme {
            palette: HashMap::new(),
            primitives: Default::default(),
            roles,
            modifiers: ModifierTheme::default(),
            font_aliases: HashMap::new(),
        }
    }

    #[test]
    fn test_resolve_basic_role() {
        let theme = sample_theme();
        let style = resolve_base_style("body", &theme);
        assert_eq!(style.font_size, Pt(12000));
    }

    #[test]
    fn font_size_modifier_is_rejected() {
        let theme = sample_theme();
        let err = patch_for_modifier(
            &Modifier {
                range: [0, 4],
                mod_type: "font-size".into(),
                intent: "12pt".into(),
            },
            &theme,
        )
        .unwrap_err();
        assert!(err.contains("Unknown modifier type"), "{err}");
    }

    #[test]
    fn test_modifier_override() {
        let theme = sample_theme();
        let modifiers = vec![Modifier {
            range: [0, 10],
            mod_type: "emphasis".to_string(),
            intent: "strong".to_string(),
        }];

        let base = resolve_base_style("body", &theme);
        let patch = patch_for_modifier(&modifiers[0], &theme).unwrap();
        let style = apply_patch(base, &patch);
        assert!(style.bold);
        assert_eq!(style.font_size, Pt(12000));
    }

    #[test]
    fn link_url_intent_uses_theme_default() {
        let mut theme = sample_theme();
        let mut link = HashMap::new();
        link.insert(
            "default".to_string(),
            StylePatch {
                underline: Some(true),
                color: Some("accent".into()),
                ..StylePatch::default()
            },
        );
        theme.modifiers.styles.insert("link".into(), link);
        let patch = patch_for_modifier(
            &Modifier {
                range: [0, 4],
                mod_type: "link".into(),
                intent: "https://example.com".into(),
            },
            &theme,
        )
        .unwrap();
        assert_eq!(patch.underline, Some(true));
        assert_eq!(patch.color.as_deref(), Some("accent"));
    }

    #[test]
    fn test_boolean_modifiers() {
        let theme = sample_theme();
        let modifiers = vec![Modifier {
            range: [0, 5],
            mod_type: "emphasis".to_string(),
            intent: "bold".to_string(),
        }];

        let base = resolve_base_style("body", &theme);
        let patch = patch_for_modifier(&modifiers[0], &theme).unwrap();
        let style = apply_patch(base, &patch);
        assert!(style.bold);
    }

    #[test]
    fn test_unknown_role_fallback() {
        let theme = sample_theme();
        let style = resolve_base_style("unknown_role", &theme);
        // Defaults
        assert_eq!(style.font_size, Pt(12000));
    }

    #[test]
    fn resolve_base_style_applies_role_bold() {
        let theme: Theme = serde_json::from_str(
            r#"{
      "palette": {},
      "roles": { "h1": {
        "font_family": "default", "font_size": 24000,
        "line_height_mult": 1250, "color": "black", "bold": true
      }}
    }"#,
        )
        .unwrap();
        let s = resolve_base_style("h1", &theme);
        assert!(s.bold);
        assert!(!s.italic);
    }

    #[test]
    fn resolve_base_style_applies_role_italic() {
        let theme: Theme = serde_json::from_str(
            r#"{
      "palette": {},
      "roles": { "body": {
        "font_family": "default", "font_size": 12000,
        "line_height_mult": 1600, "color": "black", "italic": true
      }}
    }"#,
        )
        .unwrap();
        let s = resolve_base_style("body", &theme);
        assert!(s.italic);
        assert!(!s.bold);
    }
}
