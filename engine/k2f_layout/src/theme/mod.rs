mod decoration;
mod primitives;
mod resolve;

pub use decoration::{unknown_primitive, ThemeDecoration};
pub use primitives::ThemePrimitives;
pub use resolve::{resolve_palette_color, resolve_theme_decoration};

use crate::list_style::ListStyle;
use crate::style::{StylePatch, TextAlign};
use k2f_core::{Align, Pt};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Theme {
    pub palette: HashMap<String, String>,
    #[serde(default)]
    pub primitives: ThemePrimitives,
    pub roles: HashMap<String, RoleStyle>,
    #[serde(default)]
    pub modifiers: ModifierTheme,
    #[serde(default)]
    pub font_aliases: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ModifierTheme {
    #[serde(default)]
    pub precedence: Vec<String>,
    #[serde(default)]
    pub styles: HashMap<String, HashMap<String, StylePatch>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RoleStyle {
    pub font_family: String,
    pub font_size: Pt,
    pub line_height_mult: i128,
    pub color: String,
    #[serde(default)]
    pub text_align: TextAlign,
    #[serde(default)]
    pub self_align: Option<Align>,
    #[serde(default)]
    pub box_decoration: Option<ThemeDecoration>,
    #[serde(default)]
    pub list_style: Option<ListStyle>,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub letter_spacing_pt: Pt,
    #[serde(default)]
    pub first_line_indent_pt: Pt,
    #[serde(default)]
    pub variants: HashMap<String, RoleVariant>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RoleVariant {
    #[serde(default)]
    pub box_decoration: Option<ThemeDecoration>,
    #[serde(default)]
    pub self_align: Option<Align>,
    #[serde(default)]
    pub text_overrides: Option<StylePatch>,
    #[serde(default)]
    pub list_style: Option<ListStyle>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_parses_named_decoration_refs() {
        let theme: Theme = serde_json::from_str(
            r##"
{
  "palette": { "black": "#000000", "white": "#FFFFFF" },
  "primitives": {
    "surfaces": { "glass": { "type": "solid", "color": "#FFFFFFCC" } },
    "shadows": { "elevation.1": { "layers": [ { "offset_x_pt": 0, "offset_y_pt": 2000, "blur_radius_pt": 4000, "spread_radius_pt": 0, "color": "#0000001A" } ] } },
    "blurs": { "background": { "radius_pt": 20000 } },
    "corners": { "medium": 12000 },
    "borders": { "subtle": { "width_pt": 500, "color": "#0000001A" } }
  },
  "roles": {
    "card": {
      "font_family": "default",
      "font_size": 12000,
      "line_height_mult": 1200,
      "color": "black",
      "variants": {
        "glass": {
          "box_decoration": {
            "background": "glass",
            "shadow": "elevation.1",
            "blur": "background",
            "corner_radius": "medium",
            "border": "subtle"
          }
        }
      }
    }
  }
}
"##,
        )
        .unwrap();

        assert!(theme.primitives.surfaces.contains_key("glass"));
        let glass = &theme.roles["card"].variants["glass"].box_decoration;
        assert_eq!(glass.as_ref().unwrap().background.as_deref(), Some("glass"));
        assert_eq!(
            glass.as_ref().unwrap().corner_radius.as_deref(),
            Some("medium")
        );
    }

    #[test]
    fn unknown_fill_name_is_compile_error() {
        let theme: Theme = serde_json::from_str(
            r##"
{
  "palette": { "black": "#000000" },
  "roles": {
    "body": {
      "font_family": "default",
      "font_size": 12000,
      "line_height_mult": 1200,
      "color": "black",
      "box_decoration": { "background": "nope" }
    }
  }
}
"##,
        )
        .unwrap();
        let deco = theme.roles["body"].box_decoration.as_ref().unwrap();
        let err = resolve_theme_decoration(deco, &theme).unwrap_err();
        assert!(err.contains("UNKNOWN_PRIMITIVE"), "{err}");
        assert!(err.contains("fill"), "{err}");
        assert!(err.contains("nope"), "{err}");
    }
}
