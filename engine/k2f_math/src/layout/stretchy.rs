use super::{layout_node, max_pt, merge, shape_text, MathBox};
use crate::atom::AtomClass;
use crate::constants;
use crate::error::MathError;
use crate::parse::{Delim, MathNode};
use crate::style::MathStyle;
use k2f_core::{FillRect, Pt};
use k2f_text::Font;

pub(super) fn layout_left_right(
    left: Delim,
    inner: &MathNode,
    right: Delim,
    font: &Font,
    base_size: Pt,
    style: MathStyle,
) -> Result<MathBox, MathError> {
    let inner_box = layout_node(inner, font, base_size, style)?;
    wrap_delims(left, inner_box, right, font, style.scale(base_size))
}

pub(crate) fn wrap_delims(
    left: Delim,
    inner: MathBox,
    right: Delim,
    font: &Font,
    size: Pt,
) -> Result<MathBox, MathError> {
    let left_box = stretch_delim(
        left,
        inner.ascent,
        inner.descent,
        font,
        size,
        AtomClass::Open,
    )?;
    let right_box = stretch_delim(
        right,
        inner.ascent,
        inner.descent,
        font,
        size,
        AtomClass::Close,
    )?;
    let left_a = left_box.ascent;
    let inner_a = inner.ascent;
    let right_a = right_box.ascent;
    let left_d = left_box.descent;
    let inner_d = inner.descent;
    let right_d = right_box.descent;
    let left_w = left_box.width;
    let inner_w = inner.width;
    let right_w = right_box.width;
    let ascent = max_pt(max_pt(left_a, inner_a), right_a);
    let descent = max_pt(max_pt(left_d, inner_d), right_d);
    let mut out = MathBox::empty(AtomClass::Inner);
    out.ascent = ascent;
    out.descent = descent;
    let mut x = Pt::ZERO;
    merge(&mut out, left_box, x, ascent - left_a);
    x = x + left_w;
    merge(&mut out, inner, x, ascent - inner_a);
    x = x + inner_w;
    merge(&mut out, right_box, x, ascent - right_a);
    out.width = x + right_w;
    Ok(out)
}

fn stretch_delim(
    delim: Delim,
    inner_ascent: Pt,
    inner_descent: Pt,
    font: &Font,
    size: Pt,
    class: AtomClass,
) -> Result<MathBox, MathError> {
    match delim {
        Delim::Null => {
            let mut b = MathBox::empty(class);
            b.ascent = inner_ascent;
            b.descent = inner_descent;
            Ok(b)
        }
        Delim::Char('|') => stretch_bar(false, inner_ascent, inner_descent, font, size, class),
        Delim::Char('\u{2016}') => {
            stretch_bar(true, inner_ascent, inner_descent, font, size, class)
        }
        Delim::Char(ch) => stretch_glyph(ch, inner_ascent, inner_descent, font, size, class),
    }
}

fn stretch_bar(
    double: bool,
    inner_ascent: Pt,
    inner_descent: Pt,
    font: &Font,
    size: Pt,
    class: AtomClass,
) -> Result<MathBox, MathError> {
    let target_h = inner_ascent + inner_descent;
    let shaped = shape_text(if double { "\u{2016}" } else { "|" }, font, size)?;
    let Some(ink) = font.glyph_ink(if double { '\u{2016}' } else { '|' }, size) else {
        return fit_height(shaped, inner_ascent, inner_descent, class);
    };
    let bar_w = font
        .glyph_ink('|', size)
        .map(|i| i.width())
        .unwrap_or_else(|| constants::rule_thickness(size));
    let mut out = MathBox::empty(class);
    out.width = shaped.width;
    out.ascent = inner_ascent;
    out.descent = inner_descent;
    if double {
        out.fill_rects.push(FillRect {
            x: ink.x_min,
            y: Pt::ZERO,
            width: bar_w,
            height: target_h,
        });
        let right = max_pt(ink.x_min, ink.x_max - bar_w);
        out.fill_rects.push(FillRect {
            x: right,
            y: Pt::ZERO,
            width: bar_w,
            height: target_h,
        });
    } else {
        out.fill_rects.push(FillRect {
            x: ink.x_min,
            y: Pt::ZERO,
            width: max_pt(bar_w, constants::rule_thickness(size)),
            height: target_h,
        });
    }
    Ok(out)
}

fn stretch_glyph(
    ch: char,
    inner_ascent: Pt,
    inner_descent: Pt,
    font: &Font,
    size: Pt,
    class: AtomClass,
) -> Result<MathBox, MathError> {
    let target_h = inner_ascent + inner_descent;
    let natural_h = constants::ascent(size) + constants::descent(size);
    if let Some(pieces) = pieces(ch) {
        if target_h.0 > natural_h.0 {
            return assemble(pieces, inner_ascent, inner_descent, font, size, class);
        }
    }
    let use_size = if target_h.0 > natural_h.0 && natural_h.0 > 0 {
        Pt(size.0 * target_h.0 / natural_h.0)
    } else {
        size
    };
    let shaped = shape_text(&ch.to_string(), font, use_size)?;
    fit_height(shaped, inner_ascent, inner_descent, class)
}

struct Pieces {
    top: char,
    ext: char,
    bot: char,
    mid: Option<char>,
}

fn pieces(ch: char) -> Option<Pieces> {
    Some(match ch {
        '(' => Pieces {
            top: '\u{239B}',
            ext: '\u{239C}',
            bot: '\u{239D}',
            mid: None,
        },
        ')' => Pieces {
            top: '\u{239E}',
            ext: '\u{239F}',
            bot: '\u{23A0}',
            mid: None,
        },
        '[' => Pieces {
            top: '\u{23A1}',
            ext: '\u{23A2}',
            bot: '\u{23A3}',
            mid: None,
        },
        ']' => Pieces {
            top: '\u{23A4}',
            ext: '\u{23A5}',
            bot: '\u{23A6}',
            mid: None,
        },
        '{' => Pieces {
            top: '\u{23A7}',
            ext: '\u{23AA}',
            bot: '\u{23A9}',
            mid: Some('\u{23A8}'),
        },
        '}' => Pieces {
            top: '\u{23AB}',
            ext: '\u{23AA}',
            bot: '\u{23AD}',
            mid: Some('\u{23AC}'),
        },
        _ => return None,
    })
}

fn assemble(
    p: Pieces,
    inner_ascent: Pt,
    inner_descent: Pt,
    font: &Font,
    size: Pt,
    class: AtomClass,
) -> Result<MathBox, MathError> {
    let target_h = inner_ascent + inner_descent;
    let top = shape_text(&p.top.to_string(), font, size)?;
    let bot = shape_text(&p.bot.to_string(), font, size)?;
    let ext = shape_text(&p.ext.to_string(), font, size)?;
    let mut out = MathBox::empty(class);
    out.ascent = inner_ascent;
    out.descent = inner_descent;
    if let Some(mid_ch) = p.mid {
        let mid = shape_text(&mid_ch.to_string(), font, size)?;
        let top_h = top.height();
        let mid_h = mid.height();
        let mid_y = max_pt(Pt::ZERO, (target_h - mid_h) / 2);
        let bot_y = max_pt(Pt::ZERO, target_h - bot.height());
        place(&mut out, top, Pt::ZERO);
        tile(&mut out, &ext, top_h, mid_y);
        place(&mut out, mid, mid_y);
        tile(&mut out, &ext, mid_y + mid_h, bot_y);
        place(&mut out, bot, bot_y);
    } else {
        let bot_y = max_pt(Pt::ZERO, target_h - bot.height());
        let top_h = top.height();
        place(&mut out, top, Pt::ZERO);
        tile(&mut out, &ext, top_h, bot_y);
        place(&mut out, bot, bot_y);
    }
    Ok(out)
}

fn place(out: &mut MathBox, child: MathBox, y: Pt) {
    out.width = max_pt(out.width, child.width);
    merge(out, child, Pt::ZERO, y);
}

fn tile(out: &mut MathBox, ext: &MathBox, y0: Pt, y1: Pt) {
    if y1.0 <= y0.0 || ext.height().0 <= 0 {
        return;
    }
    let eh = ext.height();
    let mut y = y0;
    while y.0 + eh.0 <= y1.0 {
        place(out, ext.clone(), y);
        y = y + eh;
    }
    if y.0 < y1.0 {
        let y_last = y1 - eh;
        place(out, ext.clone(), max_pt(y0, y_last));
    }
}

fn fit_height(
    mut b: MathBox,
    target_ascent: Pt,
    target_descent: Pt,
    class: AtomClass,
) -> Result<MathBox, MathError> {
    let target_h = target_ascent + target_descent;
    let h = b.height();
    let dy = (target_h - h) / 2;
    for g in &mut b.glyphs {
        g.y = g.y + dy;
    }
    for r in &mut b.fill_rects {
        r.y = r.y + dy;
    }
    b.ascent = target_ascent;
    b.descent = target_descent;
    b.class = class;
    Ok(b)
}
