use super::stretchy::wrap_delims;
use super::{layout_node, max_pt, merge, MathBox};
use crate::atom::AtomClass;
use crate::constants;
use crate::error::MathError;
use crate::parse::{Delim, EnvKind, MathNode};
use crate::style::MathStyle;
use k2f_core::Pt;
use k2f_text::Font;

pub(super) fn layout_env(
    kind: EnvKind,
    rows: &[Vec<MathNode>],
    font: &Font,
    base_size: Pt,
    style: MathStyle,
) -> Result<MathBox, MathError> {
    let table = layout_table(kind, rows, font, base_size, style)?;
    let size = style.scale(base_size);
    match kind {
        EnvKind::Matrix | EnvKind::Align => Ok(table),
        EnvKind::PMatrix => wrap_delims(Delim::Char('('), table, Delim::Char(')'), font, size),
        EnvKind::BMatrix => wrap_delims(Delim::Char('['), table, Delim::Char(']'), font, size),
        EnvKind::Cases => wrap_delims(Delim::Char('{'), table, Delim::Null, font, size),
    }
}

#[derive(Clone, Copy)]
enum ColAlign {
    Left,
    Center,
    Right,
}

fn col_align(kind: EnvKind, col: usize) -> ColAlign {
    match kind {
        EnvKind::Align if col % 2 == 0 => ColAlign::Right,
        EnvKind::Align => ColAlign::Left,
        EnvKind::Cases => ColAlign::Left,
        EnvKind::Matrix | EnvKind::PMatrix | EnvKind::BMatrix => ColAlign::Center,
    }
}

fn gap_after(kind: EnvKind, col: usize, ncols: usize, size: Pt) -> Pt {
    if col + 1 >= ncols {
        return Pt::ZERO;
    }
    match kind {
        EnvKind::Align if col % 2 == 0 => Pt::ZERO,
        EnvKind::Align => constants::align_group_sep(size),
        _ => constants::array_col_sep(size),
    }
}

fn layout_table(
    kind: EnvKind,
    rows: &[Vec<MathNode>],
    font: &Font,
    base_size: Pt,
    style: MathStyle,
) -> Result<MathBox, MathError> {
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if ncols == 0 {
        return Ok(MathBox::empty(AtomClass::Inner));
    }
    let mut cells: Vec<Vec<MathBox>> = Vec::with_capacity(rows.len());
    for row in rows {
        let mut laid = Vec::with_capacity(ncols);
        for j in 0..ncols {
            match row.get(j) {
                Some(n) => laid.push(layout_node(n, font, base_size, style)?),
                None => laid.push(MathBox::empty(AtomClass::Ord)),
            }
        }
        cells.push(laid);
    }

    let mut col_w = vec![Pt::ZERO; ncols];
    for row in &cells {
        for (j, cell) in row.iter().enumerate() {
            col_w[j] = max_pt(col_w[j], cell.width);
        }
    }
    let mut row_ascent = Vec::with_capacity(cells.len());
    let mut row_descent = Vec::with_capacity(cells.len());
    for row in &cells {
        let mut a = Pt::ZERO;
        let mut d = Pt::ZERO;
        for cell in row {
            a = max_pt(a, cell.ascent);
            d = max_pt(d, cell.descent);
        }
        row_ascent.push(a);
        row_descent.push(d);
    }

    let size = style.scale(base_size);
    let mut xs = vec![Pt::ZERO; ncols];
    let mut x = Pt::ZERO;
    for j in 0..ncols {
        xs[j] = x;
        x = x + col_w[j] + gap_after(kind, j, ncols, size);
    }
    let width = x;

    let row_sep = constants::array_row_sep(size);
    let mut y = Pt::ZERO;
    let mut out = MathBox::empty(AtomClass::Inner);
    for (i, row) in cells.into_iter().enumerate() {
        for (j, cell) in row.into_iter().enumerate() {
            let cell_w = cell.width;
            let cell_a = cell.ascent;
            let dx = match col_align(kind, j) {
                ColAlign::Left => xs[j],
                ColAlign::Center => xs[j] + (col_w[j] - cell_w) / 2,
                ColAlign::Right => xs[j] + (col_w[j] - cell_w),
            };
            let dy = y + (row_ascent[i] - cell_a);
            merge(&mut out, cell, dx, dy);
        }
        y = y + row_ascent[i] + row_descent[i];
        if i + 1 < row_ascent.len() {
            y = y + row_sep;
        }
    }

    let axis = constants::axis_height(size);
    let mut ascent = y / 2 + axis;
    let mut descent = y - ascent;
    if descent.0 < 0 {
        ascent = y;
        descent = Pt::ZERO;
    }
    out.width = width;
    out.ascent = ascent;
    out.descent = descent;
    Ok(out)
}
