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
}

#[cfg(test)]
mod wrap_tests;
