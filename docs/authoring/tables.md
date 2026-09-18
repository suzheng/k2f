# Tables

A table is `content.type: "table"` with inline `data.rows`. Each cell is a full node (usually `table_header_cell` or `table_row_cell`).

Copy [`ex_table.json`](../../skills/k2f/catalog/content/ex_table.json) into `content/root.json` `children`.

## Shape

- `column_widths` — `{ "fr": 1 }` tracks (same language as [grid](layout.md))
- `header_rows` — how many leading rows are header
- `data.type: "inline"` and `data.rows` — array of rows, each an array of nodes
- Numeric right-align: cell `variant: "end"` (theme `text_overrides`, not a node `text_align`)

Starter must already define roles `table`, `table_header_cell`, and `table_row_cell`.

## See also (catalog, not this page)

| Need | File |
|------|------|
| Dense metrics | [`ex_table_dense.json`](../../skills/k2f/catalog/content/ex_table_dense.json) |
| Ruled edges | [`ex_table_edges.json`](../../skills/k2f/catalog/content/ex_table_edges.json) |
| Same-row colspan | [`ex_table_colspan.json`](../../skills/k2f/catalog/content/ex_table_colspan.json) |
| Title + list in a cell | [`ex_table_composite.json`](../../skills/k2f/catalog/content/ex_table_composite.json) |

No `rowspan`. Full-width titles are **siblings above** the table, not merged cells.

## Allowed keys

[fields.md](../../skills/k2f/references/writing/fields.md) and [`nodes.schema.json`](../../skills/k2f/schema/nodes.schema.json).

## Common mistakes

- Drawing a table with stacked text and `_` / `|` characters
- Setting `text_align` on the cell node instead of a theme variant
- Using `column_span` (columns layout) where you meant cell `colspan`
