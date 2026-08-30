use crate::resolved_style::EdgeInsets;
use crate::{Size, SizeConstraint};
use k2f_core::{FixedSizeHint, LayoutHint, Pt};

/// Extract the fixed-size hint (if any) from a container layout hint.
///
/// Only some layout variants carry an explicit fixed size.
pub fn fixed_size_hint(layout: &Option<LayoutHint>) -> FixedSizeHint {
    match layout {
        Some(LayoutHint::Stack { size, .. }) => *size,
        Some(LayoutHint::Overlay { size }) => *size,
        Some(LayoutHint::Grid { size, .. }) => *size,
        _ => FixedSizeHint::default(),
    }
}

/// Compute the maximum inner size available to children, applying padding and
/// fixed-size hints deterministically.
///
/// - Padding is always subtracted from the parent constraint's max.
/// - If a fixed width/height is present, children are constrained to the
///   corresponding inner size (fixed outer minus padding), but never exceed the
///   parent-provided max (deterministic).
pub fn inner_max_for_children(
    constraint: SizeConstraint,
    padding: EdgeInsets,
    fixed: FixedSizeHint,
) -> Size {
    let mut inner_max = Size::new(
        subtract_if_bounded(constraint.max.width, padding.horizontal()),
        subtract_if_bounded(constraint.max.height, padding.vertical()),
    );

    if let Some(w) = fixed.width {
        let fixed_inner_w = subtract_if_bounded(w, padding.horizontal());
        inner_max.width = min_pt(inner_max.width, fixed_inner_w);
    }
    if let Some(h) = fixed.height {
        let fixed_inner_h = subtract_if_bounded(h, padding.vertical());
        inner_max.height = min_pt(inner_max.height, fixed_inner_h);
    }

    inner_max
}

/// Cap an already-negotiated inner size using fixed-size hints.
///
/// This is used during arrangement to ensure any re-measurement uses the same
/// effective bounds as the measurement pass for fixed-size containers.
pub fn cap_inner_size_by_fixed(inner: Size, padding: EdgeInsets, fixed: FixedSizeHint) -> Size {
    let mut capped = inner;
    if let Some(w) = fixed.width {
        let fixed_inner_w = subtract_if_bounded(w, padding.horizontal());
        capped.width = min_pt(capped.width, fixed_inner_w);
    }
    if let Some(h) = fixed.height {
        let fixed_inner_h = subtract_if_bounded(h, padding.vertical());
        capped.height = min_pt(capped.height, fixed_inner_h);
    }
    capped
}

/// Apply the fixed-size hint as a minimum outer size.
///
/// If a fixed width/height is present, the measured size becomes:
/// `max(measured, fixed)` for that axis, then clamped to the parent constraint.
pub fn apply_fixed_min_outer(
    measured_outer: Size,
    fixed: FixedSizeHint,
    constraint: SizeConstraint,
) -> Size {
    let mut out = measured_outer;
    let mut changed = false;

    if let Some(w) = fixed.width {
        out.width = Pt(out.width.0.max(w.0));
        changed = true;
    }
    if let Some(h) = fixed.height {
        out.height = Pt(out.height.0.max(h.0));
        changed = true;
    }

    if changed {
        constraint.constrain(out)
    } else {
        out
    }
}

fn min_pt(a: Pt, b: Pt) -> Pt {
    if a.0 <= b.0 {
        a
    } else {
        b
    }
}

pub fn subtract_if_bounded(max: Pt, amount: Pt) -> Pt {
    if max.0 == i128::MAX {
        return max;
    }
    let v = max.0 - amount.0;
    Pt(v.max(0))
}
