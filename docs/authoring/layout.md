# Layout

`layout` applies only to **containers** (`content.type: "container"`). It positions that node's `children`. Leaves (text, image, table, …) do not carry `layout`.

Omit `layout` or use a vertical `stack` for ordinary top-to-bottom flow. Sibling paragraphs are separate text nodes; space between them is the parent `layout.gap` (see [Text](text.md)).

Copy [`ex_stack.json`](../../skills/k2f/catalog/content/ex_stack.json) and [`ex_grid.json`](../../skills/k2f/catalog/content/ex_grid.json) into `content/root.json` `children`. Keep `id: "root"` — do not replace `root.json` with a catalog fragment.

## Root rule

`content/root.json` must omit `layout` or use a **vertical** `stack` only. A `grid`, horizontal `stack`, `overlay`, or `columns` layout belongs on a **nested** child (for example `root` → `main.section` → grid).

## Stack

`layout.type: "stack"`, or omit `type` (stack is the default).

| Field | Meaning |
| --- | --- |
| `direction` | `vertical` (default) or `horizontal` |
| `gap` | Space between children, millipt (1000 = 1 pt) |
| `align_items` | Cross-axis: `start`, `center`, `end`, `stretch` |
| `justify_content` | Main-axis: `start`, `center`, `end` — not CSS `space-between` |
| `width` / `height` | Optional fixed outer size, millipt |

Title on the left and logo, meta, or amount on the right is a **two-column grid**, not a flex row: [`ex_split_bar.json`](../../skills/k2f/catalog/content/ex_split_bar.json). A block pinned to the trailing edge with left-aligned lines uses [`ex_end_block.json`](../../skills/k2f/catalog/content/ex_end_block.json).

## Grid

`layout.type: "grid"` with required `columns` and optional `rows`.

Tracks are always objects — never bare integers:

| Track | Meaning |
| --- | --- |
| `{ "fr": N }` | Share leftover space along that axis |
| `{ "pt": N }` | Fixed size, millipt |
| `{ "auto": true }` | Size from the largest child in that track |

| Goal | Approach |
| --- | --- |
| N equal columns | N `{ "fr": 1 }` column tracks |
| Auto-wrapped rows | Omit `rows` — the engine adds `{ "auto": true }` rows (`ceil(children ÷ columns)`) |
| Fixed row template | Set `rows` explicitly — the list does **not** grow when you add children |
| `fr` **rows** | The grid needs a finite outer **height** (`layout.height` or a fixed-height parent) |
| `fr` **columns** | The grid needs a finite outer **width** (or a bounded parent) |

Optional: `gap`, `row_gap`, `column_gap`, `cell_align` (default stretch), `width`, `height`. Working example: [`ex_grid.json`](../../skills/k2f/catalog/content/ex_grid.json).

## Page size and spacing

All layout numbers are **millipt** (1 pt = 1000). Page width, height, and margins are in `manifest.json` `page_config`, not on nodes. Inset inside a band is a **role** in [Theme and fonts](theme.md) (`box_decoration.padding_pt`), not `margin` or `padding` on a node.

## See also (catalog)

| Need | File |
| --- | --- |
| Magazine image + copy | [`ex_media_row.json`](../../skills/k2f/catalog/content/ex_media_row.json) |
| Page-height shell (slide, poster, filled page) | [`ex_poster_shell.json`](../../skills/k2f/catalog/content/ex_poster_shell.json), [`ex_filled_page.json`](../../skills/k2f/catalog/content/ex_filled_page.json) |
| Background under content | [`ex_overlay.json`](../../skills/k2f/catalog/content/ex_overlay.json) |
| Flowing multi-column article | [`ex_columns.json`](../../skills/k2f/catalog/content/ex_columns.json) |
| All constructs | [Catalog](../reference/catalog.md) |

## Allowed keys

[Allowed keys](../reference/keys.md) and [`nodes.schema.json`](../../skills/k2f/schema/nodes.schema.json) → `layout`.

## Common mistakes

- Replacing `content/root.json` with a grid or horizontal-stack fragment (`root is not document flow`)
- `justify_content: "space-between"` (not in the schema)
- CSS-style fields on nodes (`margin`, `padding`, `x`, `y`)
- `{ "fr": 1 }` rows without a finite outer height
- Integer track sizes instead of `{ "pt": N }`
- Empty spacer containers — use parent `gap` or theme `padding_pt` on a dedicated role
