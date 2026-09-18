# Catalog

Packable **golden examples** for authoring: one `ex_*.json` file per construct. They live under [`skills/k2f/catalog/`](../../skills/k2f/catalog/) in the repo (with the [starter](../../skills/k2f/starter/) package and agent scripts). Copy **child nodes** into your package — do not ship the catalog folder as a deliverable.

## How to use

1. Create or open a package ([Getting started](../guide/getting-started.md)).
2. Edit `content/root.json`. Keep `id: "root"` and **document flow** (omit `layout` or use a vertical `stack` only on root).
3. Paste catalog node(s) into `children`. Do **not** replace `root.json` with a grid, overlay, columns, or horizontal-stack fragment — compile fails (`root is not document flow`).
4. Match [Allowed keys](keys.md) and [`schema/`](../../skills/k2f/schema/); relock with `k2f pack`.

Topic walkthroughs: [Text](../authoring/text.md) · [Images](../authoring/images.md) · [Tables](../authoring/tables.md) · [Layout](../authoring/layout.md) · [Theme and fonts](../authoring/theme.md).

The catalog tree’s own [`content/root.json`](../../skills/k2f/catalog/content/root.json) shows `{ "include": "content/…" }` stubs for a multi-file document. When you paste a single example, only that child node matters.

## Theme and fonts

The **current package** `styles/theme.json` must define every role, modifier type, and font the copied nodes use.

| Label | Meaning |
| --- | --- |
| **starter** | Works with [starter/styles/theme.json](../../skills/k2f/starter/styles/theme.json) as shipped. |
| **catalog** | Also needs roles, variants, or primitives from [catalog/styles/theme.json](../../skills/k2f/catalog/styles/theme.json) (for example `ex_glass.json`). |

Extra setup:

- [`ex_math.json`](../../skills/k2f/catalog/content/ex_math.json) — add `NotoSansMath-Regular.ttf` from [`catalog/assets/fonts/`](../../skills/k2f/catalog/assets/fonts/), register `font_aliases`, set the math role’s `font_family`.
- Serif headings or body — `--add-font catalog/assets/fonts/NotoSerif-Regular.ttf`, alias the stem, retarget roles ([Theme and fonts](../authoring/theme.md)). Starter stays Roboto-only.

## Fixed-height page shells

These examples pin the page **content box** with `layout.height`. They ship `240000` millipt as a **demo** only — set `height` to your real content box (page height minus margins) before export.

| Page setup | Content-box height (millipt) |
| --- | --- |
| A4, default 56pt margins | `730000` |
| A4, margin `0` | `842000` |
| Letter, default margins | `680000` |
| Letter, margin `0` | `792000` |
| Widescreen, margin `0` | `540000` |
| Square (`--page square`) | `600000` |
| Card (`--page card`) | `144000` |

Page dimensions and margins: [package.md](../../skills/k2f/references/writing/package.md) (`595000×842000` for A4, not ISO `595280×841890`).

- Inset full-page shells — role `page_shell` ([`ex_poster_shell.json`](../../skills/k2f/catalog/content/ex_poster_shell.json), [`ex_filled_page.json`](../../skills/k2f/catalog/content/ex_filled_page.json)); starter uses `padding_pt: 36000` on `page_shell` (shrink on a card).
- Trim-edge band — [`ex_banner_header.json`](../../skills/k2f/catalog/content/ex_banner_header.json) (`page_shell` `flush` + nested `page_shell`; do not pad `section`/`body` to fake bleed).

Applies to: `ex_poster_shell`, `ex_poster_growers`, `ex_filled_page`, `ex_banner_header`, `ex_cover`.

## Pick a starting example

| Deliverable | Start with | Notes |
| --- | --- | --- |
| Invoice, CV, letter, memo (no trim-edge bar) | [`ex_filled_page.json`](../../skills/k2f/catalog/content/ex_filled_page.json) | `page_shell`; `{fr:1}` table in grower; fix `height` |
| Letterhead, CV, or form with full-bleed top/side band | [`ex_banner_header.json`](../../skills/k2f/catalog/content/ex_banner_header.json) | `--margin 0`, `flush` outer shell; fix `height` |
| Title + logo or amount on one line | [`ex_split_bar.json`](../../skills/k2f/catalog/content/ex_split_bar.json) | 2-col grid, not flex `space-between` |
| To/From/Date, bill-to blocks | [`ex_split_bar.json`](../../skills/k2f/catalog/content/ex_split_bar.json) | Reverse column weights `{auto:true}` + `{fr:1}` |
| Letter sender / right-flush address | [`ex_end_block.json`](../../skills/k2f/catalog/content/ex_end_block.json) | Horizontal stack `justify_content: end` |
| One-page poster, slide, or social square | [`ex_poster_shell.json`](../../skills/k2f/catalog/content/ex_poster_shell.json) | ± [`ex_poster_growers`](../../skills/k2f/catalog/content/ex_poster_growers.json), [`ex_card_bands`](../../skills/k2f/catalog/content/ex_card_bands.json), [`ex_overlay`](../../skills/k2f/catalog/content/ex_overlay.json), [`ex_glass`](../../skills/k2f/catalog/content/ex_glass.json) |
| Thesis or book cover | [`ex_cover.json`](../../skills/k2f/catalog/content/ex_cover.json) | Same shell pattern; fix `height` |
| Long report (flowing pages) | [`ex_heading`](../../skills/k2f/catalog/content/ex_heading.json) + stack + [`ex_list`](../../skills/k2f/catalog/content/ex_list.json) + [`ex_table`](../../skills/k2f/catalog/content/ex_table.json) | Optional [`ex_on_dark`](../../skills/k2f/catalog/content/ex_on_dark.json) bands |
| Two-column **article** (flowing) | [`ex_columns.json`](../../skills/k2f/catalog/content/ex_columns.json) | One columns container; figures use `column_span: "all"` |
| Magazine figure + copy | [`ex_media_row.json`](../../skills/k2f/catalog/content/ex_media_row.json) | `{pt:N}` + `{fr:1}` grid |
| KPI or metric strip | [`ex_card_bands.json`](../../skills/k2f/catalog/content/ex_card_bands.json) | Or vertical stack `justify_content: center` when sparse |
| Fillable blanks or checkboxes | [`ex_form.json`](../../skills/k2f/catalog/content/ex_form.json) | `form_field` — never `____` or `□` in body text |
| Running header/footer (title + page #) | [`catalog/manifest.json`](../../skills/k2f/catalog/manifest.json) | `running_blocks`; placeholders `page_current` / `page_total` only |
| Duplex card | Two `page_shell` children | `--page card`; back page `break_before: page`; `--expect-pages 2` |

## Examples by topic

Paths are under `skills/k2f/catalog/`.

### Layout and shells

| Example | What it shows |
| --- | --- |
| [`ex_stack.json`](../../skills/k2f/catalog/content/ex_stack.json) | Vertical / horizontal `stack`, `gap`, `align_items`, `justify_content` — **starter** |
| [`ex_grid.json`](../../skills/k2f/catalog/content/ex_grid.json) | `columns` / `rows`, `fr` tracks, `cell_align`, gaps — **starter** |
| [`ex_split_bar.json`](../../skills/k2f/catalog/content/ex_split_bar.json) | Title left, image or amount right (`body`/`h2` `variant: end`) — **starter** |
| [`ex_media_row.json`](../../skills/k2f/catalog/content/ex_media_row.json) | Fixed-width image column + flowing copy — **starter** |
| [`ex_end_block.json`](../../skills/k2f/catalog/content/ex_end_block.json) | Trailing-edge block with left-aligned lines — **starter** |
| [`ex_poster_shell.json`](../../skills/k2f/catalog/content/ex_poster_shell.json) | Page-height grid, `page_shell`, header/footer `{auto}` + body `{fr:1}` — **starter** |
| [`ex_banner_header.json`](../../skills/k2f/catalog/content/ex_banner_header.json) | Bleed band + inset body — **starter** |
| [`ex_filled_page.json`](../../skills/k2f/catalog/content/ex_filled_page.json) | Filled sheet with table grower + totals footer — **starter** |
| [`ex_poster_growers.json`](../../skills/k2f/catalog/content/ex_poster_growers.json) | Stacked `{fr:1}` rows for leftover space — **starter** |
| [`ex_card_bands.json`](../../skills/k2f/catalog/content/ex_card_bands.json) | Equal-height card interiors — **starter** |
| [`ex_cover.json`](../../skills/k2f/catalog/content/ex_cover.json) | Logo + title groups + year footer — **starter** |
| [`ex_overlay.json`](../../skills/k2f/catalog/content/ex_overlay.json) | `layout.type: overlay`, `break_inside: avoid` — **starter** |
| [`ex_glass.json`](../../skills/k2f/catalog/content/ex_glass.json) | `card` variant `glass` (blur + translucent surface) — **catalog** theme |
| [`ex_columns.json`](../../skills/k2f/catalog/content/ex_columns.json) | Flowing multi-column article — **starter** |

### Tables

| Example | What it shows |
| --- | --- |
| [`ex_table.json`](../../skills/k2f/catalog/content/ex_table.json) | Inline `content.type: table`, numeric cells `variant: end` — **starter** |
| [`ex_table_dense.json`](../../skills/k2f/catalog/content/ex_table_dense.json) | Weighted columns + `compact` cells — **starter** |
| [`ex_table_edges.json`](../../skills/k2f/catalog/content/ex_table_edges.json) | Ruled table + cell edge variants — **starter** |
| [`ex_table_colspan.json`](../../skills/k2f/catalog/content/ex_table_colspan.json) | Same-row `colspan` (no rowspan) — **starter** |
| [`ex_table_composite.json`](../../skills/k2f/catalog/content/ex_table_composite.json) | Title + list or icon row inside a cell — **starter** |

### Text, lists, and code

| Example | What it shows |
| --- | --- |
| [`ex_heading.json`](../../skills/k2f/catalog/content/ex_heading.json) | `h1` + `keep_with_next` — **starter** |
| [`ex_list.json`](../../skills/k2f/catalog/content/ex_list.json) | `list_id`, `depth`, bullet/number markers — **starter** |
| [`ex_modifiers.json`](../../skills/k2f/catalog/content/ex_modifiers.json) | emphasis, link, math, underline, strikethrough, sub/sup, syntax — **starter** |
| [`ex_code.json`](../../skills/k2f/catalog/content/ex_code.json) | `role: code`, `content.type: text` (not `code_block`) — **starter** |
| [`ex_math.json`](../../skills/k2f/catalog/content/ex_math.json) | Display math `content.type: math` — Noto Sans Math |
| [`ex_math_numbered.json`](../../skills/k2f/catalog/content/ex_math_numbered.json) | Numbered display math via grid (not `\tag`) — same fonts as `ex_math` |
| [`ex_form.json`](../../skills/k2f/catalog/content/ex_form.json) | `form_field` kinds (text, multiline, checkbox, signature) — **starter** |

### Images and chrome

| Example | What it shows |
| --- | --- |
| [`ex_image.json`](../../skills/k2f/catalog/content/ex_image.json) | Embedded PNG, millipt size — **starter** (copy asset to `assets/images/`) |
| [`ex_on_dark.json`](../../skills/k2f/catalog/content/ex_on_dark.json) | `variant: on_dark` on existing roles — **starter** |
| [`ex_badge.json`](../../skills/k2f/catalog/content/ex_badge.json) | Pill label; copy the **stack wrapper** into grid cells — **starter** |
| [`ex_rule.json`](../../skills/k2f/catalog/content/ex_rule.json) | Horizontal rule in a vertical stack/grid — **starter** |

### Document structure

| File | What it shows |
| --- | --- |
| [`content/root.json`](../../skills/k2f/catalog/content/root.json) | `{ "include": "content/…" }` pattern for split content |
| [`manifest.json`](../../skills/k2f/catalog/manifest.json) | `running_blocks` grid for header/footer |

## Verify

From `skills/k2f/` with `k2f` on PATH (`pip install k2f` or `K2F_CLI`):

```bash
python scripts/pack_verify.py catalog -o /tmp/catalog.K2F
```

The script packs the catalog folder as-is; it does not search your document workspace.

## See also

- [Allowed keys](keys.md) — field summary and catalog recipes
- [Format spec](../spec/k2f-v0.3.md)
- Per-row theme notes for agents: [catalog README on GitHub](https://github.com/suzheng/k2f/blob/main/skills/k2f/catalog/README.md)
