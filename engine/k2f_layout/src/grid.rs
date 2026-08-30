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
/// Auto tracks are not allowed here — call [`resolve_tracks_with_intrinsics`] after measuring.
pub fn resolve_tracks(tracks: &[GridTrack], gap: Pt, available: Pt) -> Result<Vec<Pt>, String> {
    if tracks.iter().any(GridTrack::is_auto) {
        return Err(
            "Cannot resolve auto tracks without content measure; give the grid a finite size and auto+fr rows"
                .to_string(),
        );
    }
    resolve_tracks_with_intrinsics(tracks, gap, available, &[])
}

/// Like [`resolve_tracks`], but `auto` tracks use `auto_sizes[i]` as their fixed size.
///
/// - `available` includes gaps.
/// - `Fr` tracks share leftover after `pt`, `auto`, and gaps.
/// - Remainder millipt go to earlier `Fr` tracks.
pub fn resolve_tracks_with_intrinsics(
    tracks: &[GridTrack],
    gap: Pt,
    available: Pt,
    auto_sizes: &[Pt],
) -> Result<Vec<Pt>, String> {
    if tracks.is_empty() {
        return Ok(vec![]);
    }

    let mut fixed_sum = Pt::ZERO;
    let mut fr_sum: i128 = 0;
    for (i, t) in tracks.iter().enumerate() {
        match t {
            GridTrack::Pt { pt } => fixed_sum += Pt(*pt as i128),
            GridTrack::Fr { fr } => fr_sum += *fr as i128,
            GridTrack::Auto { auto } => {
                if !*auto {
                    return Err("grid auto track must be {\"auto\": true}".to_string());
                }
                fixed_sum += auto_sizes.get(i).copied().unwrap_or(Pt::ZERO);
            }
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
    for (i, t) in tracks.iter().enumerate() {
        match t {
            GridTrack::Pt { pt } => out.push(Pt(*pt as i128)),
            GridTrack::Auto { .. } => {
                out.push(auto_sizes.get(i).copied().unwrap_or(Pt::ZERO));
            }
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
