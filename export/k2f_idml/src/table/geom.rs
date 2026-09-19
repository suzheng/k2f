use crate::coord::millipt_to_pt;
use k2f_core::{GeometryNode, GridTrack};

/// Native InDesign cells are one rectangle per row×column. Invoice-style grids
/// keep a 4pt `gap` and still harvest. A pill or vertically centered card in a
/// taller row does not share that band, so Cell fill would stretch.
pub(super) fn row_cells_share_band(rows: &[Vec<&GeometryNode>]) -> bool {
    if rows.is_empty() {
        return false;
    }
    for row in rows {
        if row.is_empty() {
            return false;
        }
        let y = row[0].y.0;
        let h = row[0].height.0;
        if h <= 0 {
            return false;
        }
        for cell in row {
            if cell.y.0 != y || cell.height.0 != h {
                return false;
            }
        }
    }
    true
}

/// True when each row's boxes abut horizontally (no column gap).
pub(super) fn rows_are_horizontally_flush(rows: &[Vec<&GeometryNode>]) -> bool {
    for row in rows {
        let mut x: Option<i128> = None;
        for cell in row {
            if cell.width.0 <= 0 {
                return false;
            }
            if let Some(expect) = x {
                if cell.x.0 != expect {
                    return false;
                }
            }
            x = Some(cell.x.0 + cell.width.0);
        }
    }
    true
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::{row_cells_share_band, rows_are_horizontally_flush};
    use k2f_core::{GeometryNode, Pt};

    fn geo(id: &str, x: i128, y: i128, w: i128, h: i128) -> GeometryNode {
        GeometryNode {
            id: id.into(),
            x: Pt(x),
            y: Pt(y),
            width: Pt(w),
            height: Pt(h),
            glyphs: vec![],
            text_runs: vec![],
            fill_rects: vec![],
            children: vec![],
        }
    }

    #[test]
    fn flush_grid_shares_band() {
        let a = geo("a", 0, 0, 50_000, 40_000);
        let b = geo("b", 50_000, 0, 50_000, 40_000);
        let c = geo("c", 0, 40_000, 50_000, 40_000);
        let d = geo("d", 50_000, 40_000, 50_000, 40_000);
        let rows = vec![vec![&a, &b], vec![&c, &d]];
        assert!(row_cells_share_band(&rows));
        assert!(rows_are_horizontally_flush(&rows));
    }

    #[test]
    fn invoice_style_gap_still_shares_band() {
        let a = geo("a", 0, 0, 48_000, 40_000);
        let b = geo("b", 52_000, 0, 48_000, 40_000);
        let rows = vec![vec![&a, &b]];
        assert!(row_cells_share_band(&rows));
        assert!(!rows_are_horizontally_flush(&rows));
    }

    #[test]
    fn short_pill_in_row_breaks_band() {
        let a = geo("a", 0, 0, 50_000, 32_000);
        let pill = geo("p", 50_000, 0, 50_000, 13_000);
        assert!(!row_cells_share_band(&[vec![&a, &pill]]));
    }

    #[test]
    fn mixed_y_in_row_breaks_band() {
        let tall = geo("t", 85_000, 0, 150_000, 73_000);
        let day = geo("d", 0, 10_000, 85_000, 52_000);
        assert!(!row_cells_share_band(&[vec![&day, &tall]]));
    }
}
