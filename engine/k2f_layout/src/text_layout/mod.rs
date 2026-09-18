//! Text layout for `NodeContent::Text`.
//!
//! Features:
//! - Minimal `TextLayout`/`TextLine`/`TextRun` types
//! - Deterministic text measurement using `k2f_text::TextShaper` advances
//! - Run splitting via modifiers (range-based styling) with deterministic precedence
//! - Deterministic wrapping (line breaking) using word-boundary opportunities + hard newlines

mod font_runs;
mod layout;
mod metrics;
mod preformatted;
mod run_split;
mod tracking;
mod types;
mod wrap;
mod wrap_fit;
mod wrap_tokenize;

pub use layout::layout_text;
pub(crate) use metrics::line_height_for_style;
pub use metrics::measure_text_run_width;
pub use preformatted::layout_code_block;
pub(crate) use tracking::apply_tracking;
pub use types::{TextLayout, TextLine, TextRun};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LayoutContext, SizeConstraint, Theme};
    use k2f_core::{Modifier, Pt};
    use std::collections::HashMap;

    #[test]
    fn layout_text_hello_is_positive_and_deterministic() {
        let fonts = crate::test_utils::test_fonts();
        let theme = Theme::default();
        let ctx = LayoutContext::new(&fonts, &theme);
        let constraint = SizeConstraint::infinite();

        let modifiers: Vec<Modifier> = vec![];

        let reference = layout_text("Hello", "body", None, &modifiers, constraint, &ctx).unwrap();
        assert!(reference.width > Pt::ZERO);
        assert!(reference.height > Pt::ZERO);
        assert_eq!(reference.lines.len(), 1);
        assert_eq!(reference.lines[0].runs.len(), 1);

        for _ in 0..25 {
            let current = layout_text("Hello", "body", None, &modifiers, constraint, &ctx).unwrap();
            assert_eq!(reference, current);
        }
    }

    #[test]
    fn layout_text_applies_variant_text_overrides() {
        use crate::style::StylePatch;
        use crate::theme::{RoleStyle, RoleVariant};

        let fonts = crate::test_utils::test_fonts();

        let mut roles = HashMap::new();
        roles.insert(
            "body".to_string(),
            RoleStyle {
                font_family: "default".to_string(),
                font_size: Pt(12000),
                line_height_mult: 1200,
                color: "black".to_string(),
                text_align: crate::style::TextAlign::Start,
                self_align: None,
                box_decoration: None,
                list_style: None,
                bold: false,
                italic: false,
                letter_spacing_pt: Pt::ZERO,
                first_line_indent_pt: Pt::ZERO,
                image_fit: None,
                variants: HashMap::from([(
                    "muted".to_string(),
                    RoleVariant {
                        box_decoration: None,
                        self_align: None,
                        text_overrides: Some(StylePatch {
                            color: Some("red".to_string()),
                            ..StylePatch::default()
                        }),
                        list_style: None,
                        image_fit: None,
                    },
                )]),
            },
        );

        let theme = Theme {
            palette: HashMap::new(),
            primitives: Default::default(),
            roles,
            modifiers: crate::ModifierTheme::default(),
            font_aliases: HashMap::new(),
            font_faces: HashMap::new(),
        };

        let ctx = LayoutContext::new(&fonts, &theme);
        let constraint = SizeConstraint::infinite();
        let modifiers: Vec<Modifier> = vec![];

        let layout =
            layout_text("Hello", "body", Some("muted"), &modifiers, constraint, &ctx).unwrap();
        assert_eq!(layout.lines.len(), 1);
        assert_eq!(layout.lines[0].runs.len(), 1);
        assert_eq!(layout.lines[0].runs[0].style.color, "red");
    }

    fn roboto_and_sc() -> crate::LayoutContext<'static> {
        use std::collections::BTreeMap;
        use std::path::PathBuf;
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
        let mut fonts_map = BTreeMap::new();
        fonts_map.insert(
            "assets/fonts/Roboto-Regular.ttf".into(),
            std::fs::read(dir.join("Roboto-Regular.ttf")).unwrap(),
        );
        fonts_map.insert(
            "assets/fonts/NotoSansSC-Regular.otf".into(),
            std::fs::read(dir.join("NotoSansSC-Regular.otf")).unwrap(),
        );
        let lib = crate::fonts::load_font_library(&fonts_map).unwrap();
        let theme: Theme = serde_json::from_str(
            r#"{"palette":{},"roles":{"code_block":{"font_family":"Roboto-Regular","font_size":12000,"line_height_mult":1400,"color":"ink"}}}"#,
        )
        .unwrap();
        let fonts = Box::leak(Box::new(lib));
        let theme = Box::leak(Box::new(theme));
        LayoutContext::new(fonts, theme)
    }

    #[test]
    fn layout_code_block_falls_back_for_cjk() {
        let ctx = roboto_and_sc();
        let layout = layout_code_block(
            "let x = 合;",
            "code_block",
            None,
            &[],
            SizeConstraint::infinite(),
            &ctx,
        )
        .unwrap();
        let families: Vec<&str> = layout.lines[0]
            .runs
            .iter()
            .map(|r| r.style.font_family.as_str())
            .collect();
        assert!(
            families.iter().any(|f| *f == "NotoSansSC-Regular"),
            "expected CJK run on NotoSansSC, got {families:?}"
        );
        assert!(
            families.iter().any(|f| *f == "Roboto-Regular"),
            "expected ASCII run on Roboto, got {families:?}"
        );
    }

    #[test]
    fn measure_dingbat_falls_back_to_embedded_face() {
        use crate::style::Style;
        use std::collections::BTreeMap;
        use std::path::PathBuf;
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
        let mut fonts_map = BTreeMap::new();
        fonts_map.insert(
            "assets/fonts/Roboto-Regular.ttf".into(),
            std::fs::read(dir.join("Roboto-Regular.ttf")).unwrap(),
        );
        fonts_map.insert(
            "assets/fonts/DejaVuSans.ttf".into(),
            std::fs::read(dir.join("DejaVuSans.ttf")).unwrap(),
        );
        let lib = crate::fonts::load_font_library(&fonts_map).unwrap();
        let theme: Theme = serde_json::from_str(
            r#"{"palette":{},"roles":{"body":{"font_family":"Roboto-Regular","font_size":12000,"line_height_mult":1200,"color":"ink"}}}"#,
        )
        .unwrap();
        let ctx = LayoutContext::new(&lib, &theme);
        let style = Style {
            font_family: "Roboto-Regular".into(),
            font_size: Pt(12000),
            line_height_mult: 1200,
            color: "ink".into(),
            ..Style::default()
        };
        let w = measure_text_run_width("□", &style, &ctx).unwrap();
        assert!(w > Pt::ZERO);
    }
}

#[cfg(test)]
mod wrap_tests;
