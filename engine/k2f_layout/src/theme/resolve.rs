use super::decoration::{unknown_primitive, ThemeDecoration};
use super::Theme;
use k2f_core::{BlurRef, BoxDecoration, Fill, FillRef, LinearGradient, ShadowRef};

/// Compile a named theme decoration into an inlined lock `BoxDecoration`.
pub fn resolve_theme_decoration(
    deco: &ThemeDecoration,
    theme: &Theme,
) -> Result<BoxDecoration, String> {
    let background = match deco.background.as_deref() {
        None => None,
        Some(name) => Some(FillRef::Inline(resolve_named_fill(name, theme)?)),
    };
    let border = match deco.border.as_deref() {
        None => None,
        Some(name) => {
            let mut b = theme
                .primitives
                .borders
                .get(name)
                .cloned()
                .ok_or_else(|| unknown_primitive("border", name))?;
            b.color = resolve_palette_color(&b.color, theme);
            Some(b)
        }
    };
    let corner_radius_pt = match deco.corner_radius.as_deref() {
        None => None,
        Some(name) => Some(
            *theme
                .primitives
                .corners
                .get(name)
                .ok_or_else(|| unknown_primitive("corner", name))?,
        ),
    };
    let shadow = match deco.shadow.as_deref() {
        None => None,
        Some(name) => {
            let mut s = theme
                .primitives
                .shadows
                .get(name)
                .cloned()
                .ok_or_else(|| unknown_primitive("shadow", name))?;
            for layer in &mut s.layers {
                layer.color = resolve_palette_color(&layer.color, theme);
            }
            Some(ShadowRef::Inline(s))
        }
    };
    let blur = match deco.blur.as_deref() {
        None => None,
        Some(name) => {
            let b = theme
                .primitives
                .blurs
                .get(name)
                .cloned()
                .ok_or_else(|| unknown_primitive("blur", name))?;
            Some(BlurRef::Inline(b))
        }
    };

    Ok(BoxDecoration {
        background,
        border,
        corner_radius_pt,
        padding_pt: deco.padding_pt.clone(),
        shadow,
        blur,
    })
}

fn resolve_named_fill(name: &str, theme: &Theme) -> Result<Fill, String> {
    if let Some(fill) = theme.primitives.surfaces.get(name) {
        return Ok(resolve_fill_colors(fill.clone(), theme));
    }
    if let Some(grad) = theme.primitives.gradients.get(name) {
        return Ok(Fill::LinearGradient {
            value: resolve_gradient_colors(grad.clone(), theme),
        });
    }
    Err(unknown_primitive("fill", name))
}

fn resolve_fill_colors(fill: Fill, theme: &Theme) -> Fill {
    match fill {
        Fill::Solid { color } => Fill::Solid {
            color: resolve_palette_color(&color, theme),
        },
        Fill::LinearGradient { value } => Fill::LinearGradient {
            value: resolve_gradient_colors(value, theme),
        },
    }
}

fn resolve_gradient_colors(grad: LinearGradient, theme: &Theme) -> LinearGradient {
    match grad {
        LinearGradient::Linear {
            angle_degrees,
            stops,
        } => LinearGradient::Linear {
            angle_degrees,
            stops: stops
                .into_iter()
                .map(|s| k2f_core::GradientStop {
                    pos: s.pos,
                    color: resolve_palette_color(&s.color, theme),
                })
                .collect(),
        },
    }
}

pub fn resolve_palette_color(color: &str, theme: &Theme) -> String {
    if color.trim_start().starts_with('#') {
        return color.to_string();
    }
    let mut cur = color.to_string();
    for _ in 0..8 {
        if cur.trim_start().starts_with('#') {
            return cur;
        }
        match theme.palette.get(&cur) {
            Some(next) => cur = next.clone(),
            None => break,
        }
    }
    cur
}
