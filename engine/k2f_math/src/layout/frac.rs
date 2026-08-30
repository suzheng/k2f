use super::{layout_node, max_pt, merge, shape_text, MathBox};
use crate::atom::AtomClass;
use crate::constants;
use crate::error::MathError;
use crate::parse::MathNode;
use crate::style::MathStyle;
use k2f_core::{FillRect, Pt};
use k2f_text::Font;

pub(super) fn layout_frac(
    num: &MathNode,
    den: &MathNode,
    font: &Font,
    base_size: Pt,
    style: MathStyle,
) -> Result<MathBox, MathError> {
    let inner = style.script();
    let num_box = layout_node(num, font, base_size, inner)?;
    let den_box = layout_node(den, font, base_size, inner)?;
    let size = style.scale(base_size);
    let rule = constants::rule_thickness(size);
    let gap = constants::frac_gap(size);
    let pad = constants::frac_pad(size);
    let axis = constants::axis_height(size);
    let inner_w = max_pt(num_box.width, den_box.width);
    let width = inner_w + pad + pad;

    let num_x = pad + (inner_w - num_box.width) / 2;
    let den_x = pad + (inner_w - den_box.width) / 2;

    let bar_y = num_box.height() + gap;
    let den_y = bar_y + rule + gap;
    let ascent = bar_y + rule / 2 + axis;
    let descent = den_y + den_box.height() - ascent;

    let mut out = MathBox::empty(AtomClass::Inner);
    out.width = width;
    out.ascent = ascent;
    out.descent = max_pt(descent, Pt::ZERO);
    merge(&mut out, num_box, num_x, Pt::ZERO);
    out.fill_rects.push(FillRect {
        x: pad,
        y: bar_y,
        width: inner_w,
        height: rule,
    });
    merge(&mut out, den_box, den_x, den_y);
    Ok(out)
}

pub(super) fn layout_sqrt(
    body: &MathNode,
    font: &Font,
    base_size: Pt,
    style: MathStyle,
) -> Result<MathBox, MathError> {
    let inner = layout_node(body, font, base_size, style)?;
    let size = style.scale(base_size);
    let radical = shape_text("\u{221A}", font, size)?;
    let gap = constants::sqrt_gap(size);
    let pad = constants::sqrt_pad(size);
    let rule = constants::rule_thickness(size);
    let body_top = rule + gap;
    let content_ascent = body_top + inner.ascent;
    let ascent = max_pt(content_ascent, radical.ascent);
    let descent = max_pt(inner.descent, radical.descent);
    let rad_w = radical.width;
    let rad_ascent = radical.ascent;
    let inner_w = inner.width;

    let mut out = MathBox::empty(AtomClass::Open);
    out.width = rad_w + inner_w + pad;
    out.ascent = ascent;
    out.descent = descent;
    merge(&mut out, radical, Pt::ZERO, ascent - rad_ascent);
    let body_x = rad_w;
    merge(
        &mut out,
        inner,
        body_x,
        body_top + (ascent - content_ascent),
    );
    out.fill_rects.push(FillRect {
        x: body_x,
        y: Pt::ZERO,
        width: inner_w + pad,
        height: rule,
    });
    Ok(out)
}
