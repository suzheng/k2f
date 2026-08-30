use crate::{Size, SizeConstraint};
use k2f_core::Pt;

/// Deterministic sizing for leaf nodes (no external resource loading).
pub fn measure_image(width: Pt, height: Pt, constraint: SizeConstraint) -> Result<Size, String> {
    // Semantic validation should have enforced positivity; keep a hard check here as well.
    if width.0 <= 0 || height.0 <= 0 {
        return Err(format!(
            "Image size must be positive (width={}, height={})",
            width, height
        ));
    }

    let natural = Size::new(width, height);
    let max = constraint.max;

    // If unconstrained, return the intrinsic size (still constrained to min/max for completeness).
    if is_unbounded(max.width) && is_unbounded(max.height) {
        return Ok(constraint.constrain(natural));
    }

    // If it already fits the max bounds, keep it.
    if (is_unbounded(max.width) || width <= max.width)
        && (is_unbounded(max.height) || height <= max.height)
    {
        return Ok(constraint.constrain(natural));
    }

    // Scale down proportionally to fit within max bounds, preserving aspect ratio.
    let (scale_num, scale_den) = choose_fit_scale(width, height, max.width, max.height)?;
    let scaled_w = checked_mul_div(width.0, scale_num, scale_den)?;
    let scaled_h = checked_mul_div(height.0, scale_num, scale_den)?;

    Ok(constraint.constrain(Size::new(Pt(scaled_w), Pt(scaled_h))))
}

pub fn measure_table_reference(
    width: Pt,
    height: Pt,
    constraint: SizeConstraint,
) -> Result<Size, String> {
    if width.0 <= 0 || height.0 <= 0 {
        return Err(format!(
            "TableReference size must be positive (width={}, height={})",
            width, height
        ));
    }
    Ok(constraint.constrain(Size::new(width, height)))
}

fn is_unbounded(v: Pt) -> bool {
    v.0 == i128::MAX
}

/// Choose the scale factor (as a rational num/den) that fits (width,height) into (max_w,max_h).
/// Returned ratio is always >= 0 and typically <= 1 for downscaling.
fn choose_fit_scale(width: Pt, height: Pt, max_w: Pt, max_h: Pt) -> Result<(i128, i128), String> {
    let w = width.0;
    let h = height.0;
    if w <= 0 || h <= 0 {
        return Err("choose_fit_scale: width/height must be positive".to_string());
    }

    // If one axis is unbounded, the other axis determines the scale.
    if is_unbounded(max_w) && is_unbounded(max_h) {
        return Ok((1, 1));
    }
    if is_unbounded(max_w) {
        // Only height constrains.
        let num = max_h.0.max(0);
        return Ok((num, h));
    }
    if is_unbounded(max_h) {
        // Only width constrains.
        let num = max_w.0.max(0);
        return Ok((num, w));
    }

    // Both bounded: pick the smaller ratio between max_w/w and max_h/h.
    // Compare max_w/w <= max_h/h  <=>  max_w*h <= max_h*w
    let left = (max_w.0).checked_mul(h).ok_or_else(|| {
        "choose_fit_scale: overflow comparing width/height constraints".to_string()
    })?;
    let right = (max_h.0).checked_mul(w).ok_or_else(|| {
        "choose_fit_scale: overflow comparing width/height constraints".to_string()
    })?;

    if left <= right {
        Ok((max_w.0.max(0), w))
    } else {
        Ok((max_h.0.max(0), h))
    }
}

fn checked_mul_div(value: i128, num: i128, den: i128) -> Result<i128, String> {
    if den == 0 {
        return Err("checked_mul_div: division by zero".to_string());
    }
    if num <= 0 {
        // If a max constraint is 0 (or negative), we deterministically collapse to 0.
        return Ok(0);
    }
    let prod = value
        .checked_mul(num)
        .ok_or_else(|| "checked_mul_div: overflow during scaling".to_string())?;
    Ok(prod / den)
}
