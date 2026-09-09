# K2F skill catalog

Golden, packable examples for unpacked authoring. Copy nodes into your author package; do **not** ship this directory as a deliverable. Allowed keys: [fields.md](../references/writing/fields.md), then [`../schema/`](../schema/).

The **current package** theme must already define every role, modifier type, and font a copied node uses. Starter covers the rows marked “starter” below. `ex_math.json` still needs NotoSansMath from `assets/fonts/` (not in starter).

`ex_poster_shell.json`, `ex_poster_growers.json`, `ex_filled_page.json`, and `ex_cover.json` ship `layout.height: 240000` as a **demo**. For a real page, set `height` to the **content box** (page height − margins). A4 / margin 0 → `842000`. Widescreen / margin 0 → `960000×540000` with height `540000`. Use the table in [package.md](../references/writing/package.md) (`595000×842000`), not ISO `595280×841890`. Full-bleed shells copy `ex_poster_shell.json` / `ex_filled_page.json` as role `page_shell` (starter inset `padding_pt: 36000`); do not pad `section`/`body`.

| Construct | File | Demonstrates | Theme when copying into starter |
|-----------|------|--------------|---------------------------------|
| Document include pattern | `content/root.json` | `{ "include": "content/…" }` stubs | — |
| Heading + `keep_with_next` | `content/ex_heading.json` | `h1`, body, vertical stack | starter (`h1`, `body`, `section`) |
| Vertical / horizontal stack | `content/ex_stack.json` | `layout.type: stack`, `gap`, `align_items`, `justify_content` | starter |
| Header bar (title left, logo right) | `content/ex_split_bar.json` | 2-col grid `{fr:1}` + `{auto:true}` — right cell is an image; not Flexbox `space-between`, not overlay | starter (`h2`, `image`) |
| Magazine image + copy | `content/ex_media_row.json` | `{pt:N}` image column + `{fr:1}` copy (rows omitted) | starter (`image`, `h2`, `body`, `quote`) |
| Trailing-edge block | `content/ex_end_block.json` | Horizontal stack `justify_content: end` wrapping a content-width vertical stack — letter sender / right-flush cell; lines stay `text_align` start | starter |
| Fixed-height grid + `fr` | `content/ex_grid.json` | `columns`/`rows` tracks, `height`, `cell_align`, `row_gap`, `column_gap` | starter |
| Poster/slide page shell | `content/ex_poster_shell.json` | Page-height grid: `{auto:true}` header/footer + `{fr:1}` body; nested 2-col grid in the grower. Role `page_shell` = safe inset. **Change `height`** (demo is `240000`). | starter (`page_shell`, `h1`, `card`, `body`) |
| Filled page (invoice/CV/letter) | `content/ex_filled_page.json` | Same shell as poster; `{fr:1}` grower holds a **table**, totals in the `{auto:true}` footer. Not poster-only. **Change `height`**. | starter (`page_shell`, `h1`, `table`, `table_header_cell`, `table_row_cell`, `body`) |
| Grower stacked `{fr:1}` rows | `content/ex_poster_growers.json` | Leftover on **figure** + dense cards, not a short quote. `{fr:1}` = box height, not type. Role `page_shell`. **Change `height`**. | starter (`page_shell`, `h1`, `h2`, `card`, `body`, `image`) |
| Cover (logo + titles) | `content/ex_cover.json` | Same shell as poster; header nests stacks with different `gap` (logo / titles / author); `{fr:1}` grower; footer year. **Change `height`**. | starter (`section`, `h1`, `h2`, `body`, `image`) |
| Layered overlay | `content/ex_overlay.json` | `layout.type: overlay`, `break_inside: avoid`; children shrink to content unless they set `width`/`height` | starter (`card` + `raised` variant) |
| Glass card | `content/ex_glass.json` | overlay + `card` `variant: "glass"` (`box_decoration.blur` + translucent surface). Copy catalog theme `glass_light` / `blurs.background` — not in starter | catalog |
| Flowing columns | `content/ex_columns.json` | `column_span: "all"` inside **one** unpadded columns node (long article: do not split containers) | starter |
| Inline table | `content/ex_table.json` | `content.type: table`, inline `data.rows`; cell `variant: "end"` for numeric right-align (`text_overrides`); optional `row_gap`/`column_gap` | starter (`table`, `table_header_cell`, `table_row_cell`) |
| Dense metric table | `content/ex_table_dense.json` | Weighted `{fr}` columns + cell `variant: "compact"` inside a `card` | starter (same + `compact`) |
| Table cell edges | `content/ex_table_edges.json` | Table `variant: "ruled"` (top+bottom) + header `bottom`; extra columns, not colspan | starter (same + `subtle_bottom` / `subtle_hbar`) |
| Dark band / light interior | `content/ex_on_dark.json` | Same `h2`/`body`/`card` roles + `variant: "on_dark"` — not `th_dark_*` role clones | starter (`on_dark`) |
| Badge / pill label | `content/ex_badge.json` | Stack wrapping a `badge` text node (`self_align: start`). Copy the **wrapper** into a grid cell — grid stretch ignores `self_align` on a direct child | starter (`badge`, `section`) |
| Horizontal rule | `content/ex_rule.json` | `role: "rule"` + small `layout.height` + empty children; insert as a **vertical** stack/grid child (default `align_items` is `stretch`) | starter (`rule`) |
| Bullet + numbered lists | `content/ex_list.json` | `list_id`, `depth`, `marker_type` bullet/number | starter (`list_item`) |
| Text modifiers | `content/ex_modifiers.json` | emphasis, link, math, underline, strikethrough, sub/sup, syntax_highlight; multi-line `\n` + `modifier_range.py` | starter modifier styles |
| Display math | `content/ex_math.json` | `role: math`, `content.type: math` | add `NotoSansMath-Regular.ttf` + `font_aliases`; set math `font_family` |
| Numbered display math | `content/ex_math_numbered.json` | 2-col `{fr:1}` + `{auto:true}` — not `\\tag` | same as `ex_math.json` |
| Inline code (text) | `content/ex_code.json` | `role: code` + `content.type: text` (not `code_block`) | starter (`code`) |
| Embedded image | `content/ex_image.json` | PNG path, millipt size, role `self_align` in theme | starter (`image`); copy the PNG under `assets/images/` |
| Page header/footer | `manifest.json` | `running_blocks` Grid `{fr:1}`+`{auto:true}` (title left, `{{page_current}}` right). Placeholders: `page_current` / `page_total` only | starter (`running_header`, `running_footer`, variant `end`) |

By deliverable (same files, not extra packages): invoice / CV / letter → `ex_filled_page` (role `page_shell`; fix `height` to the content box); long report → heading + stack + list + table ± `ex_on_dark`; flowing two-column **article** → `ex_columns` (one container; figures `column_span: all`); magazine figure+copy → `ex_media_row`; one-page poster or slide → `ex_poster_shell` (role `page_shell`; fix `height` to the content box) ± `ex_poster_growers` ± `ex_overlay` (nested **grid** for side-by-side in the grower); thesis cover → `ex_cover` (fix `height`); title+logo bar → `ex_split_bar`; letter sender / right-flush cell → `ex_end_block`; divider → `ex_rule` (vertical stretch parent); form-like table rules → `ex_table_edges`; dense metrics → `ex_table_dense`; badge/pill → `ex_badge` (copy the stack wrapper); running header/footer split → `catalog/manifest.json`.

Verify the whole catalog:

```bash
python scripts/pack_verify.py catalog -o /tmp/catalog.K2F
```

Run from the `skills/k2f/` directory with `k2f` on PATH (`pip install k2f` or `K2F_CLI`). The script does not search a source checkout.
