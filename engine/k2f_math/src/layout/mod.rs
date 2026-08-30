//! TeX-subset box layout. Every coordinate is millipt (`Pt`); no floats.

mod env;
mod frac;
mod script;
mod stretchy;

use crate::atom::{demote_binaries, space_mu, AtomClass};
use crate::constants;
use crate::error::{from_shape_error, MathError};
use crate::parse::MathNode;
use crate::style::MathStyle;
use crate::{MathLayout, PlacedMathGlyph};
use k2f_core::{FillRect, GlyphPosition, Pt};
use k2f_text::{Font, TextShaper};

pub fn layout_node(
    node: &MathNode,
    font: &Font,
    base_size: Pt,
    style: MathStyle,
) -> Result<MathBox, MathError> {
    match node {
        MathNode::Row(items) => layout_row(items, font, base_size, style),
        MathNode::Atom { ch, class } => {
            let mut b = shape_text(&ch.to_string(), font, style.scale(base_size))?;
            b.class = *class;
            Ok(b)
        }
        MathNode::Text(s) => {
            if s.is_empty() {
                return Ok(MathBox::empty(AtomClass::Ord));
            }
            let mut b = shape_text(s, font, style.scale(base_size))?;
            b.class = AtomClass::Ord;
            Ok(b)
        }
        MathNode::Space(mu) => Ok(MathBox::space(constants::mu(style.scale(base_size), *mu))),
        MathNode::BigOp { ch, limits_above } => {
            script::layout_big_op(*ch, *limits_above, font, base_size, style)
        }
        MathNode::OperatorName { text, limits_under } => {
            let mut b = shape_text(text, font, style.scale(base_size))?;
            b.class = AtomClass::Op;
            b.limits_above = *limits_under && style.is_display();
            b.width = b.width + constants::mu(style.scale(base_size), 3);
            Ok(b)
        }
        MathNode::Frac { num, den } => frac::layout_frac(num, den, font, base_size, style),
        MathNode::Sqrt { body } => frac::layout_sqrt(body, font, base_size, style),
        MathNode::Script { base, sup, sub } => {
            script::layout_script(base, sup.as_deref(), sub.as_deref(), font, base_size, style)
        }
        MathNode::LeftRight { left, inner, right } => {
            stretchy::layout_left_right(*left, inner, *right, font, base_size, style)
        }
        MathNode::Env { kind, rows } => env::layout_env(*kind, rows, font, base_size, style),
    }
}

#[derive(Debug, Clone)]
pub struct MathBox {
    pub width: Pt,
    pub ascent: Pt,
    pub descent: Pt,
    pub class: AtomClass,
    pub limits_above: bool,
    glyphs: Vec<PlacedMathGlyph>,
    fill_rects: Vec<FillRect>,
}

impl MathBox {
    pub(crate) fn empty(class: AtomClass) -> Self {
        Self {
            width: Pt::ZERO,
            ascent: Pt::ZERO,
            descent: Pt::ZERO,
            class,
            limits_above: false,
            glyphs: vec![],
            fill_rects: vec![],
        }
    }

    fn space(width: Pt) -> Self {
        Self {
            width,
            ascent: Pt::ZERO,
            descent: Pt::ZERO,
            class: AtomClass::Space,
            limits_above: false,
            glyphs: vec![],
            fill_rects: vec![],
        }
    }

    pub(crate) fn height(&self) -> Pt {
        self.ascent + self.descent
    }
}

pub fn flatten(b: MathBox) -> MathLayout {
    MathLayout {
        width: b.width,
        height: b.height(),
        ascent: b.ascent,
        descent: b.descent,
        glyphs: b.glyphs,
        fill_rects: b.fill_rects,
    }
}

pub(crate) fn max_pt(a: Pt, b: Pt) -> Pt {
    if a.0 >= b.0 {
        a
    } else {
        b
    }
}

pub(crate) fn shape_text(text: &str, font: &Font, size: Pt) -> Result<MathBox, MathError> {
    let shaped = TextShaper::shape_text(text, font, size).map_err(from_shape_error)?;
    let mut width = Pt::ZERO;
    let ascent = constants::ascent(size);
    let mut glyphs = Vec::with_capacity(shaped.len());
    for g in shaped {
        width = width + g.x_advance;
        glyphs.push(PlacedMathGlyph {
            glyph_id: g.glyph_id,
            x: g.x_offset,
            y: ascent + g.y_offset,
            font_size: size,
            cluster: GlyphPosition::CLUSTER_NOT_SOURCE,
        });
    }
    Ok(MathBox {
        width,
        ascent,
        descent: constants::descent(size),
        class: AtomClass::Ord,
        limits_above: false,
        glyphs,
        fill_rects: vec![],
    })
}

fn layout_row(
    items: &[MathNode],
    font: &Font,
    base_size: Pt,
    style: MathStyle,
) -> Result<MathBox, MathError> {
    if items.is_empty() {
        return Ok(MathBox::empty(AtomClass::Ord));
    }
    let mut boxes: Vec<MathBox> = Vec::with_capacity(items.len());
    for item in items {
        boxes.push(layout_node(item, font, base_size, style)?);
    }
    let mut classes: Vec<AtomClass> = boxes.iter().map(|b| b.class).collect();
    demote_binaries(&mut classes);
    for (b, c) in boxes.iter_mut().zip(classes.iter()) {
        b.class = *c;
    }

    let size = style.scale(base_size);
    let mut xs = Vec::with_capacity(boxes.len());
    let mut x = Pt::ZERO;
    let mut max_ascent = Pt::ZERO;
    let mut max_descent = Pt::ZERO;
    for i in 0..boxes.len() {
        if i > 0 {
            let mu = space_mu(boxes[i - 1].class, boxes[i].class, style);
            x = x + constants::mu(size, mu);
        }
        max_ascent = max_pt(max_ascent, boxes[i].ascent);
        max_descent = max_pt(max_descent, boxes[i].descent);
        xs.push(x);
        x = x + boxes[i].width;
    }

    let class = if boxes.len() == 1 {
        boxes[0].class
    } else {
        AtomClass::Ord
    };
    let mut out = MathBox::empty(class);
    out.width = x;
    out.ascent = max_ascent;
    out.descent = max_descent;
    for (i, child) in boxes.into_iter().enumerate() {
        let dy = max_ascent - child.ascent;
        merge(&mut out, child, xs[i], dy);
    }
    Ok(out)
}

pub(crate) fn merge(parent: &mut MathBox, child: MathBox, dx: Pt, dy: Pt) {
    for mut g in child.glyphs {
        g.x = g.x + dx;
        g.y = g.y + dy;
        parent.glyphs.push(g);
    }
    for r in child.fill_rects {
        parent.fill_rects.push(FillRect {
            x: r.x + dx,
            y: r.y + dy,
            width: r.width,
            height: r.height,
        });
    }
}
