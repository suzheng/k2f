use crate::style::{
    apply_patch, apply_script_geometry, modifier_precedence, patch_for_modifier,
};
use crate::Theme;
use k2f_core::Modifier;

use super::types::{push_or_merge_run, TextRun};

/// Split `text` into deterministic styled runs, based on range modifiers.
///
/// - Ranges are **byte offsets** into UTF-8 text (validated elsewhere to be on char boundaries).
/// - Style is computed as: base(role) + all patches whose range covers the run.
/// - Overlaps are resolved deterministically by:
///   1) modifier type precedence (`Theme.modifiers.precedence` or default),
///   2) modifier type name (lexicographic) for unknown types not in precedence list,
///   3) larger ranges apply before smaller ranges (more specific ranges win),
///   4) range start/end, then intent (lexicographic).
pub(crate) fn split_runs(
    text: &str,
    role: &str,
    variant: Option<&str>,
    modifiers: &[Modifier],
    theme: &Theme,
) -> Result<Vec<TextRun>, String> {
    if modifiers.is_empty() {
        let style = crate::resolved_style::resolve_text_style(role, variant, theme);
        return Ok(vec![TextRun {
            start: 0,
            end: text.len(),
            style,
            text: text.to_string(),
            math_tex: None,
        }]);
    }

    // Defensive validation (callers typically run `validate_semantic_tree` before layout).
    let len = text.len();
    for m in modifiers {
        let [s, e] = m.range;
        if s >= e || e > len {
            return Err(format!(
                "Invalid modifier range {:?} for text length {}",
                m.range, len
            ));
        }
        if !text.is_char_boundary(s) || !text.is_char_boundary(e) {
            return Err(format!(
                "Modifier range {:?} is not on UTF-8 char boundary",
                m.range
            ));
        }
    }

    // Collect deterministic breakpoints.
    let mut points: Vec<usize> = Vec::with_capacity(2 + modifiers.len() * 2);
    points.push(0);
    points.push(len);
    for m in modifiers {
        points.push(m.range[0]);
        points.push(m.range[1]);
    }
    points.sort_unstable();
    points.dedup();

    let precedence = modifier_precedence(theme);
    let base_style = crate::resolved_style::resolve_text_style(role, variant, theme);

    let mut out: Vec<TextRun> = Vec::new();
    for w in points.windows(2) {
        let start = w[0];
        let end = w[1];
        if start == end {
            continue;
        }

        let mut applicable: Vec<&Modifier> = modifiers
            .iter()
            .filter(|m| m.range[0] <= start && m.range[1] >= end)
            .collect();

        applicable.sort_by(|a, b| modifier_cmp(a, b, &precedence));

        let mut style = base_style.clone();
        // Highest-precedence script type wins when both somehow overlap.
        let mut script_kind: Option<&str> = None;
        for m in &applicable {
            let mut patch = patch_for_modifier(m, theme)?;
            if m.mod_type == "superscript" || m.mod_type == "subscript" {
                // Engine owns script size; ignore theme font_size on these types.
                patch.font_size = None;
                script_kind = Some(m.mod_type.as_str());
            }
            style = apply_patch(style, &patch);
        }
        if let Some(kind) = script_kind {
            apply_script_geometry(&mut style, kind);
        }

        let run_text = text[start..end].to_string();
        let math_tex = applicable
            .iter()
            .find(|m| m.mod_type == "math")
            .map(|m| m.intent.clone());
        push_or_merge_run(
            &mut out,
            TextRun {
                start,
                end,
                style,
                text: run_text,
                math_tex,
            },
        );
    }

    // Ensure we always return at least one run (should be true given points include 0/len).
    if out.is_empty() {
        let style = crate::resolved_style::resolve_text_style(role, variant, theme);
        return Ok(vec![TextRun {
            start: 0,
            end: text.len(),
            style,
            text: text.to_string(),
            math_tex: None,
        }]);
    }

    Ok(out)
}

fn precedence_index(mod_type: &str, precedence: &[String]) -> usize {
    precedence
        .iter()
        .position(|t| t == mod_type)
        .unwrap_or(precedence.len())
}

fn modifier_cmp(a: &Modifier, b: &Modifier, precedence: &[String]) -> std::cmp::Ordering {
    use std::cmp::Reverse;

    let a_prec = precedence_index(&a.mod_type, precedence);
    let b_prec = precedence_index(&b.mod_type, precedence);

    // Larger ranges apply first (so smaller, more specific ranges override later).
    let a_len = a.range[1] - a.range[0];
    let b_len = b.range[1] - b.range[0];

    (
        a_prec,
        a.mod_type.as_str(),
        Reverse(a_len),
        a.range[0],
        Reverse(a.range[1]),
        a.intent.as_str(),
    )
        .cmp(&(
            b_prec,
            b.mod_type.as_str(),
            Reverse(b_len),
            b.range[0],
            Reverse(b.range[1]),
            b.intent.as_str(),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::StylePatch;
    use k2f_core::Pt;
    use std::collections::HashMap;

    fn color_emphasis_theme() -> Theme {
        let mut theme = Theme::default();
        let mut by_intent = HashMap::new();
        by_intent.insert(
            "red".to_string(),
            StylePatch {
                color: Some("red".to_string()),
                ..StylePatch::default()
            },
        );
        by_intent.insert(
            "blue".to_string(),
            StylePatch {
                color: Some("blue".to_string()),
                ..StylePatch::default()
            },
        );
        theme
            .modifiers
            .styles
            .insert("emphasis".to_string(), by_intent);
        theme
    }

    #[test]
    fn adjacent_modifiers_create_adjacent_runs() {
        let theme = color_emphasis_theme();
        let text = "HelloWorld";
        let modifiers = vec![
            Modifier {
                range: [0, 5],
                mod_type: "emphasis".to_string(),
                intent: "red".to_string(),
            },
            Modifier {
                range: [5, 10],
                mod_type: "emphasis".to_string(),
                intent: "blue".to_string(),
            },
        ];

        let runs = split_runs(text, "body", None, &modifiers, &theme).unwrap();
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].start, 0);
        assert_eq!(runs[0].end, 5);
        assert_eq!(runs[0].text, "Hello");
        assert_eq!(runs[0].style.color, "red");
        assert_eq!(runs[1].start, 5);
        assert_eq!(runs[1].end, 10);
        assert_eq!(runs[1].text, "World");
        assert_eq!(runs[1].style.color, "blue");
    }

    #[test]
    fn overlapping_same_type_more_specific_range_wins() {
        let theme = color_emphasis_theme();
        let text = "abcdef";
        let modifiers = vec![
            Modifier {
                range: [0, 6],
                mod_type: "emphasis".to_string(),
                intent: "red".to_string(),
            },
            Modifier {
                range: [1, 3],
                mod_type: "emphasis".to_string(),
                intent: "blue".to_string(),
            },
        ];

        let runs = split_runs(text, "body", None, &modifiers, &theme).unwrap();
        assert_eq!(runs.len(), 3);
        assert_eq!(
            (
                runs[0].start,
                runs[0].end,
                runs[0].text.as_str(),
                runs[0].style.color.as_str()
            ),
            (0, 1, "a", "red")
        );
        assert_eq!(
            (
                runs[1].start,
                runs[1].end,
                runs[1].text.as_str(),
                runs[1].style.color.as_str()
            ),
            (1, 3, "bc", "blue")
        );
        assert_eq!(
            (
                runs[2].start,
                runs[2].end,
                runs[2].text.as_str(),
                runs[2].style.color.as_str()
            ),
            (3, 6, "def", "red")
        );
    }

    #[test]
    fn overlapping_different_types_follow_precedence_deterministically() {
        let mut theme = Theme::default();

        // Set explicit precedence: "a" then "b" (so "b" applies later and wins).
        theme.modifiers.precedence = vec!["emphasis".to_string(), "underline".to_string()];

        // Provide theme styles for both types, both writing `color`.
        let mut styles: HashMap<String, HashMap<String, StylePatch>> = HashMap::new();
        styles.insert(
            "emphasis".to_string(),
            HashMap::from([(
                "x".to_string(),
                StylePatch {
                    color: Some("red".to_string()),
                    ..StylePatch::default()
                },
            )]),
        );
        styles.insert(
            "underline".to_string(),
            HashMap::from([(
                "x".to_string(),
                StylePatch {
                    color: Some("blue".to_string()),
                    ..StylePatch::default()
                },
            )]),
        );
        theme.modifiers.styles = styles;

        let text = "abc";
        // Provide modifiers in "winning-last" *reverse* order to ensure we don't depend on input ordering.
        let modifiers = vec![
            Modifier {
                range: [0, 3],
                mod_type: "underline".to_string(),
                intent: "x".to_string(),
            },
            Modifier {
                range: [0, 3],
                mod_type: "emphasis".to_string(),
                intent: "x".to_string(),
            },
        ];

        let runs = split_runs(text, "body", None, &modifiers, &theme).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "abc");
        assert_eq!(runs[0].style.color, "blue");
    }

    #[test]
    fn utf8_byte_ranges_split_on_char_boundaries() {
        let theme = color_emphasis_theme();
        let text = "a\u{1F642}b"; // U+1F642 is 4 bytes in UTF-8

        // Byte layout: "a" [0..1], "🙂" [1..5], "b" [5..6]
        assert_eq!(text.len(), 6);
        assert!(text.is_char_boundary(0));
        assert!(text.is_char_boundary(1));
        assert!(text.is_char_boundary(5));
        assert!(text.is_char_boundary(6));

        let modifiers = vec![Modifier {
            range: [1, 5],
            mod_type: "emphasis".to_string(),
            intent: "red".to_string(),
        }];

        let runs = split_runs(text, "body", None, &modifiers, &theme).unwrap();
        assert_eq!(runs.len(), 3);

        assert_eq!(
            (runs[0].start, runs[0].end, runs[0].text.as_str()),
            (0, 1, "a")
        );
        assert_eq!(runs[0].style.color, "black"); // base style default

        assert_eq!(
            (runs[1].start, runs[1].end, runs[1].text.as_str()),
            (1, 5, "\u{1F642}")
        );
        assert_eq!(runs[1].style.color, "red");

        assert_eq!(
            (runs[2].start, runs[2].end, runs[2].text.as_str()),
            (5, 6, "b")
        );
        assert_eq!(runs[2].style.color, "black"); // base style default
    }

    #[test]
    fn superscript_shrinks_and_raises() {
        let theme = Theme::default();
        let text = "x2";
        let modifiers = vec![Modifier {
            range: [1, 2],
            mod_type: "superscript".to_string(),
            intent: "default".to_string(),
        }];
        let runs = split_runs(text, "body", None, &modifiers, &theme).unwrap();
        assert_eq!(runs.len(), 2);
        let parent = runs[0].style.font_size;
        assert_eq!(runs[1].style.font_size, Pt(parent.0 * 7 / 10));
        assert_eq!(runs[1].style.baseline_shift, Pt(parent.0 * 450 / 1000));
        assert_eq!(runs[0].style.baseline_shift, Pt::ZERO);
    }

    #[test]
    fn subscript_shrinks_and_lowers() {
        let theme = Theme::default();
        let text = "H2O";
        let modifiers = vec![Modifier {
            range: [1, 2],
            mod_type: "subscript".to_string(),
            intent: "default".to_string(),
        }];
        let runs = split_runs(text, "body", None, &modifiers, &theme).unwrap();
        assert_eq!(runs.len(), 3);
        let parent = runs[0].style.font_size;
        assert_eq!(runs[1].text, "2");
        assert_eq!(runs[1].style.font_size, Pt(parent.0 * 7 / 10));
        assert_eq!(runs[1].style.baseline_shift, Pt(-(parent.0 * 250 / 1000)));
    }

    #[test]
    fn script_theme_font_size_is_ignored() {
        use crate::style::StylePatch;
        use std::collections::HashMap;

        let mut theme = Theme::default();
        let mut by_intent = HashMap::new();
        by_intent.insert(
            "default".to_string(),
            StylePatch {
                font_size: Some(Pt(99_000)),
                color: Some("red".to_string()),
                ..StylePatch::default()
            },
        );
        theme
            .modifiers
            .styles
            .insert("superscript".to_string(), by_intent);

        let text = "a1";
        let modifiers = vec![Modifier {
            range: [1, 2],
            mod_type: "superscript".to_string(),
            intent: "default".to_string(),
        }];
        let runs = split_runs(text, "body", None, &modifiers, &theme).unwrap();
        let parent = runs[0].style.font_size;
        assert_eq!(runs[1].style.font_size, Pt(parent.0 * 7 / 10));
        assert_ne!(runs[1].style.font_size, Pt(99_000));
        assert_eq!(runs[1].style.color, "red");
    }

    #[test]
    fn utf8_non_boundary_range_is_rejected() {
        let theme = Theme::default();
        let text = "a\u{1F642}b";

        // [1,2] falls inside the multi-byte emoji and is not a char boundary.
        let modifiers = vec![Modifier {
            range: [1, 2],
            mod_type: "emphasis".to_string(),
            intent: "red".to_string(),
        }];

        let err = split_runs(text, "body", None, &modifiers, &theme).unwrap_err();
        assert!(err.contains("not on UTF-8 char boundary"));
    }
}
