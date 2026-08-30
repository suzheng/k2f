use k2f_core::{GridTrack, Pt};

/// Resolve per-axis grid gaps. Omitted `row_gap` / `column_gap` fall back to `gap`.
pub fn grid_axis_gaps(gap: i64, row_gap: Option<i64>, column_gap: Option<i64>) -> (Pt, Pt) {
    (
        Pt(row_gap.unwrap_or(gap) as i128),
        Pt(column_gap.unwrap_or(gap) as i128),
    )
}

/// Resolve grid track sizes for a single axis (columns or rows).
///
/// - `available` is the total available size for the whole axis, **including gaps**.
/// - `gap` is the gap inserted between tracks (there are `tracks.len() - 1` gaps).
/// - `Pt` tracks are always honored as-is.
/// - `Fr` tracks share the remaining space after fixed tracks and gaps.
///
/// Remainder distribution is deterministic: any leftover units are assigned to
/// earlier `Fr` tracks in order.
pub fn resolve_tracks(tracks: &[GridTrack], gap: Pt, available: Pt) -> Result<Vec<Pt>, String> {
    if tracks.is_empty() {
        return Ok(vec![]);
    }

    let mut fixed_sum = Pt::ZERO;
    let mut fr_sum: i128 = 0;
    for t in tracks {
        match t {
            GridTrack::Pt { pt } => fixed_sum += Pt(*pt as i128),
            GridTrack::Fr { fr } => fr_sum += *fr as i128,
        }
    }

    if fr_sum > 0 && available.0 == i128::MAX {
        return Err(
            "Cannot resolve fr tracks with infinite available size; give the grid a fixed height or use stack/table"
                .to_string(),
        );
    }

    let gap_total = if gap != Pt::ZERO {
        Pt(gap.0 * (tracks.len().saturating_sub(1) as i128))
    } else {
        Pt::ZERO
    };

    // `available_for_tracks` excludes gaps (since gaps are not part of the tracks themselves).
    // If available is finite, we subtract gaps; if it's infinite, fr_sum must be 0 (handled above).
    let mut available_for_tracks = if available.0 == i128::MAX {
        available
    } else {
        available - gap_total
    };
    if available_for_tracks.0 < 0 {
        available_for_tracks = Pt::ZERO;
    }

    let mut remaining = available_for_tracks - fixed_sum;
    if remaining.0 < 0 {
        remaining = Pt::ZERO;
    }

    let mut out: Vec<Pt> = Vec::with_capacity(tracks.len());
    let mut remainder = remaining.0;
    for t in tracks {
        match t {
            GridTrack::Pt { pt } => out.push(Pt(*pt as i128)),
            GridTrack::Fr { fr } => {
                if fr_sum == 0 {
                    out.push(Pt::ZERO);
                } else {
                    let share = remaining.0 * (*fr as i128) / fr_sum;
                    out.push(Pt(share));
                    remainder -= share;
                }
            }
        }
    }

    // Deterministically distribute any remainder to earlier fr tracks.
    if remainder > 0 {
        for (i, t) in tracks.iter().enumerate() {
            if remainder == 0 {
                break;
            }
            if matches!(t, GridTrack::Fr { .. }) {
                out[i].0 += 1;
                remainder -= 1;
            }
        }
    }

    Ok(out)
}

pub fn sum_with_gaps(items: &[Pt], gap: Pt) -> Pt {
    let mut sum = Pt::ZERO;
    for (i, x) in items.iter().enumerate() {
        sum += *x;
        if gap != Pt::ZERO && i + 1 < items.len() {
            sum += gap;
        }
    }
    sum
}

/// Sum the prefix distance for `count` tracks, including gaps.
///
/// This is used to compute the offset to the start of the `count`-th track
/// (i.e. number of preceding tracks).
pub fn sum_prefix(items: &[Pt], count: usize, gap: Pt) -> Pt {
    let mut sum = Pt::ZERO;
    for i in 0..count {
        sum += items[i];
        if gap != Pt::ZERO {
            sum += gap;
        }
    }
    sum
}
