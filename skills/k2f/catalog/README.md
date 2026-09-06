# K2F skill catalog

Golden, packable examples for unpacked authoring. Copy nodes into your author package; do **not** ship this directory as a deliverable. Allowed keys: [fields.md](../references/writing/fields.md), then [`../schema/`](../schema/).

The **current package** theme must already define every role, modifier type, and font a copied node uses. Starter covers the rows marked “starter” below. `ex_math.json` still needs NotoSansMath from `assets/fonts/` (not in starter).

`ex_poster_shell.json` and `ex_cover.json` ship `layout.height: 240000` as a **demo**. For a real page, set `height` to page height − margins (A4 / margin 0 → `842000`; widescreen / margin 0 → `960000×540000` with height `540000`).

| Construct | File | Demonstrates | Theme when copying into starter |
|-----------|------|--------------|---------------------------------|
| Document include pattern | `content/root.json` | `{ "include": "content/…" }` stubs | — |
| Heading + `keep_with_next` | `content/ex_heading.json` | `h1`, body, vertical stack | starter (`h1`, `body`, `section`) |
| Vertical / horizontal stack | `content/ex_stack.json` | `layout.type: stack`, `gap`, `align_items`, `justify_content` | starter |
| Header bar (title left, logo right) | `content/ex_split_bar.json` | 2-col grid `{fr:1}` + `{auto:true}` — right cell is an image; not Flexbox `space-between`, not overlay | starter (`h2`, `image`) |
| Trailing-edge block | `content/ex_end_block.json` | Horizontal stack `justify_content: end` wrapping a content-width vertical stack — letter sender / right-flush cell; lines stay `text_align` start | starter |
| Fixed-height grid + `fr` | `content/ex_grid.json` | `columns`/`rows` tracks, `height`, `cell_align`, `row_gap`, `column_gap` | starter |
| Poster/slide page shell | `content/ex_poster_shell.json` | Page-height grid: `{auto:true}` header/footer + `{fr:1}` body; nested 2-col grid in the grower. **Change `height`** (demo is `240000`). | starter (`section`, `h1`, `card`, `body`) |
| Cover (logo + titles) | `content/ex_cover.json` | Same shell as poster; header nests stacks with different `gap` (logo / titles / author); `{fr:1}` grower; footer year. **Change `height`**. | starter (`section`, `h1`, `h2`, `body`, `image`) |
| Layered overlay | `content/ex_overlay.json` | `layout.type: overlay`, `break_inside: avoid`; children shrink to content unless they set `width`/`height` | starter (`card` + `raised` variant) |
| Flowing columns | `content/ex_columns.json` | `column_span: "all"`, multi-column flow | starter |
| Inline table | `content/ex_table.json` | `content.type: table`, inline `data.rows`; cell `variant: "center"` for vertical middle | starter (`table`, `table_header_cell`, `table_row_cell`) |
| Table cell edges | `content/ex_table_edges.json` | Named border `edges` via cell `variant: "hbar"` / `"bottom"` — extra columns, not colspan | starter (same + `subtle_bottom` / `subtle_hbar`) |
| Horizontal rule | `content/ex_rule.json` | `role: "rule"` + small `layout.height` + empty children; insert as a stack/grid **child** | starter (`rule`) |
| Bullet + numbered lists | `content/ex_list.json` | `list_id`, `depth`, `marker_type` bullet/number | starter (`list_item`) |
| Text modifiers | `content/ex_modifiers.json` | emphasis, link, math, underline, strikethrough, sub/sup, syntax_highlight | starter modifier styles |
| Display math | `content/ex_math.json` | `role: math`, `content.type: math` | add `NotoSansMath-Regular.ttf` + `font_aliases`; set math `font_family` |
| Numbered display math | `content/ex_math_numbered.json` | 2-col `{fr:1}` + `{auto:true}` — not `\\tag` | same as `ex_math.json` |
| Inline code (text) | `content/ex_code.json` | `role: code` + `content.type: text` (not `code_block`) | starter (`code`) |
| Embedded image | `content/ex_image.json` | PNG path, millipt size, role `self_align` in theme | starter (`image`); copy the PNG under `assets/images/` |
| Page header/footer | `manifest.json` | `running_blocks`, `{{page_current}}` placeholders | starter (`running_header`, `running_footer`) |

By deliverable (same files, not extra packages): long report → heading + stack + list + table; one-page poster or slide → `ex_poster_shell` (fix `height`) ± `ex_overlay`; thesis cover → `ex_cover` (fix `height`); two-column body → nested grid inside the grower, or `ex_columns`; title+logo bar → `ex_split_bar`; letter sender / right-flush cell → `ex_end_block`; divider → `ex_rule` (as a child); form-like table rules → `ex_table_edges`.

Verify the whole catalog:

```bash
python scripts/pack_verify.py catalog -o /tmp/catalog.K2F
```

Run from the `skills/k2f/` directory with `k2f` on PATH (`pip install k2f` or `K2F_CLI`). The script does not search a source checkout.
