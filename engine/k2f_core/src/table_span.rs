use crate::SemanticNode;
use std::collections::HashSet;

/// Occupancy of an authored table cell (`colspan`, default 1).
pub fn table_cell_colspan(cell: &SemanticNode) -> usize {
    (cell.colspan as usize).max(1)
}

/// Visual start column and occupancy of one authored cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableCellSlot {
    pub start_col: usize,
    pub span: usize,
}

/// Row-major slots: each cell starts where the previous occupancy ended.
pub fn table_row_slots(row: &[SemanticNode]) -> Vec<TableCellSlot> {
    let mut start = 0usize;
    let mut out = Vec::with_capacity(row.len());
    for cell in row {
        let span = table_cell_colspan(cell);
        out.push(TableCellSlot {
            start_col: start,
            span,
        });
        start += span;
    }
    out
}

/// Sum of cell occupancy on a row (default 1 per cell).
pub fn table_row_cover(row: &[SemanticNode]) -> usize {
    row.iter().map(table_cell_colspan).sum()
}

/// Semantic rows whose every authored cell id is present on this page fragment.
pub fn table_rows_on_page<'a>(
    rows: &'a [Vec<SemanticNode>],
    present_ids: &HashSet<&str>,
) -> Vec<&'a [SemanticNode]> {
    rows.iter()
        .filter(|row| !row.is_empty() && row.iter().all(|c| present_ids.contains(c.id.as_str())))
        .map(|r| r.as_slice())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeContent;

    fn cell(id: &str, span: u32) -> SemanticNode {
        SemanticNode {
            id: id.into(),
            role: "body".into(),
            content: NodeContent::Text("x".into()),
            colspan: span,
            ..Default::default()
        }
    }

    #[test]
    fn slots_walk_start_columns() {
        let row = vec![cell("a", 2), cell("b", 1)];
        assert_eq!(table_row_cover(&row), 3);
        assert_eq!(
            table_row_slots(&row),
            vec![
                TableCellSlot {
                    start_col: 0,
                    span: 2
                },
                TableCellSlot {
                    start_col: 2,
                    span: 1
                },
            ]
        );
    }

    #[test]
    fn omitted_colspan_covers_one() {
        let row = vec![cell("a", 1), cell("b", 1)];
        assert_eq!(table_row_cover(&row), 2);
    }

    #[test]
    fn rows_on_page_require_every_cell() {
        let rows = vec![
            vec![cell("h0", 1), cell("h1", 1)],
            vec![cell("a", 2), cell("b", 1)],
            vec![cell("c0", 1), cell("c1", 1), cell("c2", 1)],
        ];
        let present: HashSet<&str> = ["h0", "h1", "a", "b"].into_iter().collect();
        let on_page = table_rows_on_page(&rows, &present);
        assert_eq!(on_page.len(), 2);
        assert_eq!(on_page[0][0].id, "h0");
        assert_eq!(on_page[1][0].id, "a");
    }
}
