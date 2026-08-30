use crate::LayoutContext;
use k2f_core::Pt;

use super::metrics::measure_text_run_width;

pub(crate) fn fit_prefix_boundary(
    text: &str,
    style: &crate::style::Style,
    max_width: Pt,
    ctx: &LayoutContext,
) -> Result<usize, String> {
    if text.is_empty() {
        return Ok(0);
    }

    // Collect char boundaries (byte offsets) deterministically.
    let mut bounds: Vec<usize> = Vec::new();
    for (idx, _) in text.char_indices() {
        bounds.push(idx);
    }
    bounds.push(text.len());
    bounds.sort_unstable();
    bounds.dedup();

    // Binary search for the largest prefix that fits.
    let mut best: usize = 0;
    let mut lo: usize = 1;
    let mut hi: usize = bounds.len().saturating_sub(1);
    while lo <= hi && hi > 0 {
        let mid = (lo + hi) / 2;
        let cut = bounds[mid];
        let w = measure_text_run_width(&text[..cut], style, ctx)?;
        if w <= max_width {
            best = cut;
            lo = mid + 1;
        } else if mid == 0 {
            break;
        } else {
            hi = mid - 1;
        }
    }

    // Ensure we always make progress (at least one char), even if nothing fits.
    if best == 0 {
        Ok(bounds.get(1).copied().unwrap_or(text.len()))
    } else {
        Ok(best)
    }
}
