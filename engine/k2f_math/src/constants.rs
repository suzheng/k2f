//! Layout constants, all derived from the current font size by integer math.
//! No float ever touches a width or a position.

use k2f_core::Pt;

/// `size * per_mille / 1000`, i.e. a fraction of the em.
fn em(size: Pt, per_mille: i128) -> Pt {
    Pt(size.0 * per_mille / 1000)
}

/// `count` math units (1mu = 1/18 em).
pub(crate) fn mu(size: Pt, count: i128) -> Pt {
    Pt(size.0 * count / 18)
}

/// Approximate typographic extents. v1 deliberately does not read the OpenType
/// MATH table or per-glyph bounding boxes; a flat 800/200 split keeps rows,
/// scripts and fractions stable across fonts.
pub(crate) fn ascent(size: Pt) -> Pt {
    em(size, 800)
}

pub(crate) fn descent(size: Pt) -> Pt {
    em(size, 200)
}

/// Fraction bar and radical vinculum thickness, never thinner than one millipt.
pub(crate) fn rule_thickness(size: Pt) -> Pt {
    Pt(std::cmp::max(1, size.0 * 40 / 1000))
}

/// Vertical clearance between a fraction bar and the numerator/denominator.
pub(crate) fn frac_gap(size: Pt) -> Pt {
    em(size, 150)
}

/// Horizontal padding on each side of a fraction bar.
pub(crate) fn frac_pad(size: Pt) -> Pt {
    em(size, 100)
}

/// Height of the math axis above the baseline; fraction bars centre on it.
pub(crate) fn axis_height(size: Pt) -> Pt {
    em(size, 250)
}

pub(crate) fn sup_shift(size: Pt) -> Pt {
    em(size, 450)
}

pub(crate) fn sub_shift(size: Pt) -> Pt {
    em(size, 250)
}

/// Minimum clearance between a superscript and a subscript on the same base.
pub(crate) fn script_gap(size: Pt) -> Pt {
    em(size, 100)
}

/// Clearance between the radical vinculum and the radicand.
pub(crate) fn sqrt_gap(size: Pt) -> Pt {
    em(size, 80)
}

/// Overhang of the vinculum past the radicand.
pub(crate) fn sqrt_pad(size: Pt) -> Pt {
    em(size, 60)
}

/// Clearance between a big operator and a limit placed above or below it.
pub(crate) fn limit_gap(size: Pt) -> Pt {
    em(size, 150)
}

/// Display-style big operators are drawn at 1.5x the surrounding size.
pub(crate) fn big_op_size(size: Pt) -> Pt {
    Pt(size.0 * 15 / 10)
}

/// Horizontal gap between matrix/cases columns.
pub(crate) fn array_col_sep(size: Pt) -> Pt {
    em(size, 400)
}

/// Vertical gap between array rows.
pub(crate) fn array_row_sep(size: Pt) -> Pt {
    em(size, 150)
}

/// Gap between `align` equation groups (`a&=b`  `|`  `c&=d`).
pub(crate) fn align_group_sep(size: Pt) -> Pt {
    em(size, 1000)
}
