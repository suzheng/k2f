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

/// Shrink auto track sizes so `pt + auto + gaps` fits `available`.
///
/// `pt` tracks are never resized. Overflowing autos shrink in proportion.
/// Unbounded `available` leaves autos unchanged.
fn fit_auto_sizes(tracks: &[GridTrack], auto_sizes: &[Pt], gap: Pt, available: Pt) -> Vec<Pt> {
    let n = tracks.len();
    let mut autos: Vec<Pt> = (0..n)
        .map(|i| {
            if tracks[i].is_auto() {
                auto_sizes.get(i).copied().unwrap_or(Pt::ZERO)
            } else {
                Pt::ZERO
            }
        })
        .collect();
    if available.0 == i128::MAX || n == 0 {
        return autos;
    }
    let pt_sum: i128 = tracks
        .iter()
        .map(|t| match t {
            GridTrack::Pt { pt } => *pt as i128,
            _ => 0,
        })
        .sum();
    let gap_total = gap.0 * (n.saturating_sub(1) as i128);
    let room = (available.0 - pt_sum - gap_total).max(0);
    let auto_sum: i128 = autos.iter().map(|p| p.0).sum();
    if auto_sum <= room {
        return autos;
    }
    if auto_sum == 0 {
        return autos;
    }
    // Proportional shrink so two overflowing autos share the axis instead of
    // zeroing the trailing track.
    let mut used = 0_i128;
    let mut last_auto = None;
    for i in 0..n {
        if !tracks[i].is_auto() {
            continue;
        }
        last_auto = Some(i);
        let share = autos[i].0 * room / auto_sum;
        autos[i] = Pt(share);
        used += share;
    }
    let rem = room - used;
    if let Some(i) = last_auto {
        autos[i].0 += rem;
    }
    autos
}

/// Like [`resolve_tracks`], but `auto` tracks use `auto_sizes[i]` as their content size.
///
/// - `available` includes gaps.
/// - Auto tracks hug content, then shrink if `pt + auto + gaps` would overflow `available`
///   (they must not spill into sibling tracks).
/// - `Fr` tracks share leftover after `pt`, fitted `auto`, and gaps.
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

    for t in tracks {
        if let GridTrack::Auto { auto } = t {
            if !*auto {
                return Err("grid auto track must be {\"auto\": true}".to_string());
            }
        }
    }

    let fitted_auto = fit_auto_sizes(tracks, auto_sizes, gap, available);

    let mut fixed_sum = Pt::ZERO;
    let mut fr_sum: i128 = 0;
    for (i, t) in tracks.iter().enumerate() {
        match t {
            GridTrack::Pt { pt } => fixed_sum += Pt(*pt as i128),
            GridTrack::Fr { fr } => fr_sum += *fr as i128,
            GridTrack::Auto { .. } => {
                fixed_sum += fitted_auto.get(i).copied().unwrap_or(Pt::ZERO);
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
                out.push(fitted_auto.get(i).copied().unwrap_or(Pt::ZERO));
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

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::GridTrack;

    #[test]
    fn overflowing_auto_tracks_shrink_to_available() {
        let tracks = vec![
            GridTrack::Auto { auto: true },
            GridTrack::Auto { auto: true },
        ];
        let out = resolve_tracks_with_intrinsics(
            &tracks,
            Pt(1000),
            Pt(100_000),
            &[Pt(80_000), Pt(80_000)],
        )
        .unwrap();
        assert_eq!(out, vec![Pt(49_500), Pt(49_500)]);
        assert_eq!(sum_with_gaps(&out, Pt(1000)), Pt(100_000));
    }

    #[test]
    fn auto_that_fits_is_unchanged_and_leftover_goes_to_fr() {
        let tracks = vec![GridTrack::Fr { fr: 1 }, GridTrack::Auto { auto: true }];
        let out =
            resolve_tracks_with_intrinsics(&tracks, Pt::ZERO, Pt(100_000), &[Pt(0), Pt(30_000)])
                .unwrap();
        assert_eq!(out, vec![Pt(70_000), Pt(30_000)]);
    }

    #[test]
    fn unbounded_available_does_not_shrink_auto() {
        let tracks = vec![
            GridTrack::Auto { auto: true },
            GridTrack::Auto { auto: true },
        ];
        let out = resolve_tracks_with_intrinsics(
            &tracks,
            Pt::ZERO,
            Pt(i128::MAX),
            &[Pt(80_000), Pt(80_000)],
        )
        .unwrap();
        assert_eq!(out, vec![Pt(80_000), Pt(80_000)]);
    }
}
