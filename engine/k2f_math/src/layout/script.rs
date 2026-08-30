use super::{layout_node, max_pt, merge, shape_text, MathBox};
use crate::atom::AtomClass;
use crate::constants;
use crate::error::MathError;
use crate::parse::MathNode;
use crate::style::MathStyle;
use k2f_core::Pt;
use k2f_text::Font;

pub(super) fn layout_big_op(
    ch: char,
    limits_above: bool,
    font: &Font,
    base_size: Pt,
    style: MathStyle,
) -> Result<MathBox, MathError> {
    let size = if style.is_display() {
        constants::big_op_size(style.scale(base_size))
    } else {
        style.scale(base_size)
    };
    let mut b = shape_text(&ch.to_string(), font, size)?;
    b.class = AtomClass::Op;
    b.limits_above = limits_above && style.is_display();
    Ok(b)
}

pub(super) fn layout_script(
    base_node: &MathNode,
    sup: Option<&MathNode>,
    sub: Option<&MathNode>,
    font: &Font,
    base_size: Pt,
    style: MathStyle,
) -> Result<MathBox, MathError> {
    let base = layout_node(base_node, font, base_size, style)?;
    let script_style = style.script();
    let size = style.scale(base_size);
    let sup_box = match sup {
        Some(n) => Some(layout_node(n, font, base_size, script_style)?),
        None => None,
    };
    let sub_box = match sub {
        Some(n) => Some(layout_node(n, font, base_size, script_style)?),
        None => None,
    };

    if base.limits_above && style.is_display() {
        return layout_op_limits(base, sup_box, sub_box, size);
    }

    let sup_shift = constants::sup_shift(size);
    let sub_shift = constants::sub_shift(size);
    let gap = constants::script_gap(size);
    let script_x = base.width;

    let mut ascent = base.ascent;
    let mut descent = base.descent;
    if let Some(supb) = &sup_box {
        ascent = max_pt(ascent, sup_shift + supb.ascent);
    }
    if let Some(subb) = &sub_box {
        descent = max_pt(descent, sub_shift + subb.descent);
    }
    if let (Some(supb), Some(subb)) = (&sup_box, &sub_box) {
        let sup_bottom = (ascent - (sup_shift + supb.ascent)) + supb.height();
        let sub_top = ascent + sub_shift - subb.ascent;
        if sub_top.0 < sup_bottom.0 + gap.0 {
            descent = descent + (sup_bottom + gap - sub_top);
        }
    }

    let mut width = base.width;
    if let Some(s) = &sup_box {
        width = max_pt(width, script_x + s.width);
    }
    if let Some(s) = &sub_box {
        width = max_pt(width, script_x + s.width);
    }

    let class = base.class;
    let mut out = MathBox::empty(class);
    out.width = width;
    out.ascent = ascent;
    out.descent = descent;
    let base_dy = ascent - base.ascent;
    merge(&mut out, base, Pt::ZERO, base_dy);
    if let Some(supb) = sup_box {
        let dy = max_pt(Pt::ZERO, ascent - (sup_shift + supb.ascent));
        merge(&mut out, supb, script_x, dy);
    }
    if let Some(subb) = sub_box {
        let dy = ascent + sub_shift - subb.ascent;
        merge(&mut out, subb, script_x, dy);
    }
    Ok(out)
}

fn layout_op_limits(
    base: MathBox,
    sup: Option<MathBox>,
    sub: Option<MathBox>,
    size: Pt,
) -> Result<MathBox, MathError> {
    let gap = constants::limit_gap(size);
    let mut width = base.width;
    if let Some(s) = &sup {
        width = max_pt(width, s.width);
    }
    if let Some(s) = &sub {
        width = max_pt(width, s.width);
    }
    let sup_h = sup.as_ref().map(|s| s.height() + gap).unwrap_or(Pt::ZERO);
    let sub_h = sub.as_ref().map(|s| s.height() + gap).unwrap_or(Pt::ZERO);
    let base_h = base.height();
    let base_w = base.width;
    let mut out = MathBox::empty(AtomClass::Op);
    out.width = width;
    out.ascent = base.ascent + sup_h;
    out.descent = base.descent + sub_h;
    if let Some(s) = sup {
        let sw = s.width;
        merge(&mut out, s, (width - sw) / 2, Pt::ZERO);
    }
    merge(&mut out, base, (width - base_w) / 2, sup_h);
    if let Some(s) = sub {
        let sw = s.width;
        merge(&mut out, s, (width - sw) / 2, sup_h + base_h + gap);
    }
    Ok(out)
}
