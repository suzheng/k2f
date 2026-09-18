# Tables

A table is one node with `role: "table"` and `content.type: "table"`. Rows live in `content.value.data` as **inline** cell trees — each cell is a full node (usually `table_header_cell` or `table_row_cell`), not a string.

Copy [`ex_table.json`](../../skills/k2f/catalog/content/ex_table.json) into `content/root.json` `children` (do not replace `root.json` with the fragment).

## Table fields

Inside `content.value`:

- `column_widths` — one track per column: `{ "fr": N }` or `{ "pt": N }` (millipt). Same track language as [grid](layout.md); tables do not use `{ "auto": true }`.
- `header_rows` — how many **leading rows** repeat at page breaks (an integer count, not a row of cells).
- `gap` — default spacing between rows and columns (millipt). Optional `row_gap` / `column_gap` override one axis; omit or `null` falls back to `gap`.
- `data` — `{ "type": "inline", "rows": [ [ cell, … ], … ] }`. Each row must cover all columns — use cell `colspan` when one cell spans multiple tracks (see catalog below).

Starter already defines roles `table`, `table_header_cell`, and `table_row_cell`.

## Cells and edits

- Cell copy lives on **text** nodes inside the cell. Later edits target those **cell** ids, not the table root id.
- A cell can hold nested structure (title + list, icon + label) — copy [`ex_table_composite.json`](../../skills/k2f/catalog/content/ex_table_composite.json).
- Look and alignment come from the cell **role** and **variant**, not fields on the node. Right-align numbers: `variant: "end"`. Vertical center in a tall cell: `variant: "center"`. Ruled tables: table `variant: "ruled"` plus cell edge variants in [`ex_table_edges.json`](../../skills/k2f/catalog/content/ex_table_edges.json). See [Theme and fonts](theme.md).

## What tables do not do

- No `rowspan`. Put a full-width title or date in a **sibling** node above the table, not a merged cell.
- `column_span: "all"` is for a table node inside a flowing columns block ([`ex_columns.json`](../../skills/k2f/catalog/content/ex_columns.json)) — not the same as cell `colspan`.
- This guide covers inline `data.rows` only. `data.type: "asset"` and `table_reference` are format features; the [K2F Skill](/skills/k2f) authoring workflow uses inline tables.

## Catalog examples

Index: [Catalog](../reference/catalog.md).

| Need | File |
|------|------|
| Table inside a filled invoice-style page | [`ex_filled_page.json`](../../skills/k2f/catalog/content/ex_filled_page.json) |
| Dense metrics | [`ex_table_dense.json`](../../skills/k2f/catalog/content/ex_table_dense.json) |
| Ruled edges | [`ex_table_edges.json`](../../skills/k2f/catalog/content/ex_table_edges.json) |
| Same-row colspan | [`ex_table_colspan.json`](../../skills/k2f/catalog/content/ex_table_colspan.json) |
| Title + list in a cell | [`ex_table_composite.json`](../../skills/k2f/catalog/content/ex_table_composite.json) |

## Allowed keys

Node fields: [Allowed keys](../reference/keys.md) (table recipes). Table shape: [`nodes.schema.json`](../../skills/k2f/schema/nodes.schema.json) (`content.type: "table"`).

## Common mistakes

- Drawing a table with stacked text and `_` / `|` characters
- Patching the table root id when you mean to change cell copy
- Setting `text_align` on the cell node instead of a theme variant such as `end` or `center`
- Using `column_span` (columns layout) where you meant cell `colspan`
- Putting a bare `image` or badge in a cell when it stretches — nest a stack ([`ex_table_composite.json`](../../skills/k2f/catalog/content/ex_table_composite.json), [`ex_badge.json`](../../skills/k2f/catalog/content/ex_badge.json))
- Faking a bullet list with `\n` in one text cell — use `list_item` nodes in a composite cell
