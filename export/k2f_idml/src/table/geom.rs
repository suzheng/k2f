use crate::coord::millipt_to_pt;
use k2f_core::{GeometryNode, GridTrack};

pub(super) fn col_widths_from_row(row: &[GeometryNode], table: &GeometryNode) -> Vec<f64> {
    let n = row.len();
    (0..n)
        .map(|i| {
            let millipt = if i + 1 < n {
                row[i + 1].x.0 - row[i].x.0
            } else {
                (table.x.0 + table.width.0) - row[i].x.0
            };
            millipt_to_pt(millipt.max(0))
        })
        .collect()
}

pub(super) fn row_heights_pt(rows: &[&[GeometryNode]], table: &GeometryNode) -> Vec<f64> {
    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            let y = row.first().map(|c| c.y.0).unwrap_or(table.y.0);
            let next_y = rows
                .get(i + 1)
                .and_then(|r| r.first())
                .map(|c| c.y.0)
                .unwrap_or(table.y.0 + table.height.0);
            let from_gap = next_y - y;
            let from_cell = row.iter().map(|c| c.height.0).max().unwrap_or(0);
            millipt_to_pt(from_gap.max(from_cell).max(0))
        })
        .collect()
}

/// Weight-split table width. Used only when cell geometry children are missing.
pub(super) fn col_widths_from_tracks(tracks: &[GridTrack], table_w: f64) -> Vec<f64> {
    let mut fixed = 0.0;
    let mut fr_sum = 0.0;
    let mut kinds = Vec::with_capacity(tracks.len());
    for t in tracks {
        match t {
            GridTrack::Pt { pt } => {
                let w = millipt_to_pt(*pt as i128);
                kinds.push((false, w));
                fixed += w;
            }
            GridTrack::Fr { fr } => {
                kinds.push((true, *fr as f64));
                fr_sum += *fr as f64;
            }
            GridTrack::Auto { .. } => {
                kinds.push((true, 1.0));
                fr_sum += 1.0;
            }
        }
    }
    let rest = (table_w - fixed).max(0.0);
    kinds
        .into_iter()
        .map(|(is_fr, v)| {
            if is_fr {
                if fr_sum > 0.0 {
                    rest * v / fr_sum
                } else {
                    0.0
                }
            } else {
                v
            }
        })
        .collect()
}
