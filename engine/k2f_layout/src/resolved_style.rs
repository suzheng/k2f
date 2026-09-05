use crate::list_style::ListStyle;
use crate::style::{apply_patch, resolve_base_style, Style};
use crate::theme::{Theme, ThemeDecoration};
use crate::visual_primitives::EdgeInsetsPt;
use k2f_core::{Align, Pt};

/// Insets in fixed-point pt units (1/1000 pt).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeInsets {
    pub top: Pt,
    pub right: Pt,
    pub bottom: Pt,
    pub left: Pt,
}

impl EdgeInsets {
    pub const ZERO: EdgeInsets = EdgeInsets {
        top: Pt::ZERO,
        right: Pt::ZERO,
        bottom: Pt::ZERO,
        left: Pt::ZERO,
    };

    pub fn horizontal(self) -> Pt {
        self.left + self.right
    }

    pub fn vertical(self) -> Pt {
        self.top + self.bottom
    }
}

/// Resolve the base text style for a role, then apply any role-variant text overrides.
///
/// Modifiers are applied later by the text run splitting logic.
pub fn resolve_text_style(role: &str, variant: Option<&str>, theme: &Theme) -> Style {
    let mut style = resolve_base_style(role, theme);

    let Some(variant_name) = variant else {
        return style;
    };

    let Some(role_style) = theme.roles.get(role).or_else(|| theme.roles.get("default")) else {
        return style;
    };

    let Some(variant_def) = role_style.variants.get(variant_name) else {
        return style;
    };

    if let Some(patch) = &variant_def.text_overrides {
        style = apply_patch(style, patch);
    }

    style
}

/// Resolve the self-alignment override for a role/variant.
///
/// Returns Some(Align) if the theme explicitly sets it, otherwise None.
pub fn resolve_self_align(role: &str, variant: Option<&str>, theme: &Theme) -> Option<Align> {
    let role_style = theme
        .roles
        .get(role)
        .or_else(|| theme.roles.get("default"))?;

    if let Some(variant_name) = variant {
        if let Some(variant_def) = role_style.variants.get(variant_name) {
            if let Some(sa) = variant_def.self_align {
                return Some(sa);
            }
        }
    }

    role_style.self_align
}

/// Resolve the box decoration for a role, then apply any role-variant overrides.
///
/// The output is representation-level (it preserves primitive references). Consumers decide
/// when/where to resolve named primitives into concrete paint operations.
pub fn resolve_box_decoration(
    role: &str,
    variant: Option<&str>,
    theme: &Theme,
) -> Option<ThemeDecoration> {
    let role_style = theme
        .roles
        .get(role)
        .or_else(|| theme.roles.get("default"))?;

    let mut decoration = role_style.box_decoration.clone();

    let Some(variant_name) = variant else {
        return decoration;
    };
    let Some(variant_def) = role_style.variants.get(variant_name) else {
        return decoration;
    };
    let Some(variant_decoration) = &variant_def.box_decoration else {
        return decoration;
    };

    match decoration.as_mut() {
        Some(base) => base.merge_from(variant_decoration),
        None => decoration = Some(variant_decoration.clone()),
    }

    decoration
}

/// Resolve list layout tokens for a role, then deterministically apply any role-variant overrides.
pub fn resolve_list_style(role: &str, variant: Option<&str>, theme: &Theme) -> Option<ListStyle> {
    let role_style = theme
        .roles
        .get(role)
        .or_else(|| theme.roles.get("default"))?;

    let base = role_style.list_style.as_ref();
    let default_for_role = role == "list_item" && base.is_none();

    let Some(variant_name) = variant else {
        if default_for_role {
            return Some(ListStyle::list_item_defaults());
        }
        return base.cloned();
    };
    let Some(variant_def) = role_style.variants.get(variant_name) else {
        if default_for_role {
            return Some(ListStyle::list_item_defaults());
        }
        return base.cloned();
    };

    let merged = ListStyle::merged(base, variant_def.list_style.as_ref());
    if merged.is_none() && role == "list_item" {
        return Some(ListStyle::list_item_defaults());
    }
    merged
}

/// Extract padding insets for a node-like (role, variant) pair.
pub fn padding_for_role_variant(
    role: &str,
    variant: Option<&str>,
    theme: &Theme,
) -> Result<EdgeInsets, String> {
    let decoration = resolve_box_decoration(role, variant, theme);
    let Some(decoration) = decoration else {
        return Ok(EdgeInsets::ZERO);
    };
    padding_from_decoration(&decoration)
}

pub fn padding_from_decoration(decoration: &ThemeDecoration) -> Result<EdgeInsets, String> {
    let Some(padding) = &decoration.padding_pt else {
        return Ok(EdgeInsets::ZERO);
    };
    edge_insets_from_pt(padding)
}

fn edge_insets_from_pt(insets: &EdgeInsetsPt) -> Result<EdgeInsets, String> {
    let (top, right, bottom, left) = match insets {
        EdgeInsetsPt::Uniform(v) => (*v, *v, *v, *v),
        EdgeInsetsPt::PerEdge {
            top,
            right,
            bottom,
            left,
        } => (*top, *right, *bottom, *left),
    };

    if top < 0 || right < 0 || bottom < 0 || left < 0 {
        return Err("Padding insets must be non-negative".to_string());
    }

    Ok(EdgeInsets {
        top: Pt(top as i128),
        right: Pt(right as i128),
        bottom: Pt(bottom as i128),
        left: Pt(left as i128),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{RoleStyle, RoleVariant};
    use crate::visual_primitives::EdgeInsetsPt;
    use std::collections::HashMap;

    #[test]
    fn resolves_and_merges_box_decoration_from_role_and_variant() {
        let mut roles = HashMap::new();
        roles.insert(
            "card".to_string(),
            RoleStyle {
                font_family: "default".to_string(),
                font_size: Pt(12_000),
                line_height_mult: 1_200,
                color: "black".to_string(),
                text_align: crate::style::TextAlign::Start,
                self_align: None,
                box_decoration: Some(ThemeDecoration {
                    padding_pt: Some(EdgeInsetsPt::Uniform(4_000)),
                    ..ThemeDecoration::default()
                }),
                list_style: None,
                bold: false,
                italic: false,
                letter_spacing_pt: Pt::ZERO,
            first_line_indent_pt: Pt::ZERO,
                variants: HashMap::from([(
                    "spacious".to_string(),
                    RoleVariant {
                        box_decoration: Some(ThemeDecoration {
                            padding_pt: Some(EdgeInsetsPt::Uniform(10_000)),
                            ..ThemeDecoration::default()
                        }),
                        self_align: None,
                        text_overrides: None,
                        list_style: None,
                    },
                )]),
            },
        );

        let theme = Theme {
            palette: HashMap::new(),
            primitives: Default::default(),
            roles,
            modifiers: Default::default(),
            font_aliases: HashMap::new(),
        };

        let decoration = resolve_box_decoration("card", Some("spacious"), &theme).unwrap();
        assert_eq!(
            padding_from_decoration(&decoration).unwrap(),
            EdgeInsets {
                top: Pt(10_000),
                right: Pt(10_000),
                bottom: Pt(10_000),
                left: Pt(10_000),
            }
        );
    }

    #[test]
    fn resolves_and_merges_list_style_from_role_and_variant() {
        use crate::list_style::ListStyle;

        let mut roles = HashMap::new();
        roles.insert(
            "list_item".to_string(),
            RoleStyle {
                font_family: "default".to_string(),
                font_size: Pt(12_000),
                line_height_mult: 1_200,
                color: "black".to_string(),
                text_align: crate::style::TextAlign::Start,
                self_align: None,
                box_decoration: None,
                list_style: Some(ListStyle {
                    marker_box_width_pt: Some(Pt(18_000)),
                    marker_gap_pt: Some(Pt(4_000)),
                    ..ListStyle::default()
                }),
                bold: false,
                italic: false,
                letter_spacing_pt: Pt::ZERO,
            first_line_indent_pt: Pt::ZERO,
                variants: HashMap::from([(
                    "compact".to_string(),
                    RoleVariant {
                        box_decoration: None,
                        self_align: None,
                        text_overrides: None,
                        list_style: Some(ListStyle {
                            marker_gap_pt: Some(Pt(2_000)),
                            bullet_glyph: Some("*".to_string()),
                            ..ListStyle::default()
                        }),
                    },
                )]),
            },
        );

        let theme = Theme {
            palette: HashMap::new(),
            primitives: Default::default(),
            roles,
            modifiers: Default::default(),
            font_aliases: HashMap::new(),
        };

        let resolved = resolve_list_style("list_item", Some("compact"), &theme).unwrap();
        assert_eq!(resolved.marker_box_width_pt, Some(Pt(18_000)));
        assert_eq!(resolved.marker_gap_pt, Some(Pt(2_000)));
        assert_eq!(resolved.bullet_glyph.as_deref(), Some("*"));
    }
}
