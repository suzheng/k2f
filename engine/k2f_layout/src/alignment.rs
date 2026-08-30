use k2f_core::{Align, JustifyContent, Pt};

/// Compute a deterministic offset + (possibly stretched) size for a child inside an axis.
///
/// - For `stretch`, the child size becomes `available` and offset is 0.
/// - For other align modes, the child keeps its `natural` size (clamped to `available`)
///   and is offset within the available space.
pub(crate) fn align_offset_and_size(align: Align, available: Pt, natural: Pt) -> (Pt, Pt) {
    let natural = if natural.0 > available.0 {
        available
    } else {
        natural
    };
    let free = (available.0 - natural.0).max(0);

    match align {
        Align::Stretch => (Pt::ZERO, available),
        Align::Start => (Pt::ZERO, natural),
        Align::Center => (Pt(free / 2), natural),
        Align::End => (Pt(free), natural),
    }
}

/// Compute the deterministic leading offset for main-axis distribution.
pub(crate) fn justify_offset(justify: JustifyContent, available: Pt, used: Pt) -> Pt {
    let free = (available.0 - used.0).max(0);
    match justify {
        JustifyContent::Start => Pt::ZERO,
        JustifyContent::Center => Pt(free / 2),
        JustifyContent::End => Pt(free),
    }
}
