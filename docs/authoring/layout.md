# Layout

Containers arrange children with `layout`. Omit `layout` (or use a vertical `stack`) for ordinary document flow. `content/root.json` itself must stay vertical flow — nest grid or a horizontal stack **under a child**, never as the root.

Copy [`ex_stack.json`](../../skills/k2f/catalog/content/ex_stack.json) and [`ex_grid.json`](../../skills/k2f/catalog/content/ex_grid.json) into `content/root.json` `children`.

## Stack

`layout.type: "stack"` with `direction` `vertical` or `horizontal`.

- `gap` — space between children (millipt). This is paragraph spacing when the children are text nodes
- `align_items` / `justify_content` — `start`, `center`, `end`, `stretch` (not CSS `space-between`)

A title-left / amount-right bar is a **2-column grid**, not Flexbox: [`ex_split_bar.json`](../../skills/k2f/catalog/content/ex_split_bar.json).

## Grid

`layout.type: "grid"` with `columns` (and optional `rows`) as `{ "fr": N }`, `{ "pt": N }`, or `{ "auto": true }`.

- Omit `rows` to wrap into implicit `{ "auto": true }` rows
- `N` equal columns: `N` `{ "fr": 1 }` tracks
- `height` is required when a row uses `fr` (the fraction needs a known height)

## Units

All lengths are millipt (1 pt = 1000). Page size and margins live in `manifest.json` `page_config`, not on nodes.

## Later

Overlay, flowing columns, and page-height shells (`ex_poster_shell.json`, `ex_filled_page.json`) are in the [catalog](../../skills/k2f/catalog/README.md). Skip them until stack and grid are enough.

## Common mistakes

- Replacing `content/root.json` with a grid fragment (`root is not document flow`)
- Inventing CSS fields (`margin`, `padding` on a node, `x` / `y`)
- Using stack `justify_content: "space-between"` (not in the schema)
