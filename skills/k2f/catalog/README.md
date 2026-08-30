# K2F skill catalog

Golden, packable examples for unpacked authoring. Copy nodes into your author package; do **not** ship this directory as a deliverable. Allowed keys: [`../schema/`](../schema/).

| Construct | File | Demonstrates |
|-----------|------|--------------|
| Document include pattern | `content/root.json` | `{ "include": "content/…" }` stubs |
| Heading + `keep_with_next` | `content/ex_heading.json` | `h1`, body, vertical stack |
| Vertical / horizontal stack | `content/ex_stack.json` | `layout.type: stack`, `gap`, `align_items`, `justify_content` |
| Fixed-height grid + `fr` | `content/ex_grid.json` | `columns`/`rows` tracks, `height`, `cell_align`, `row_gap`, `column_gap` |
| Poster/slide page shell | `content/ex_poster_shell.json` | Page-height grid: `pt` header/footer + `{fr:1}` body; nested 2-col grid in the grower. Set `height` to page height − margins. |
| Layered overlay | `content/ex_overlay.json` | `layout.type: overlay`, `break_inside: avoid` |
| Flowing columns | `content/ex_columns.json` | `column_span: "all"`, multi-column flow |
| Inline table | `content/ex_table.json` | `content.type: table`, inline `data.rows` |
| Bullet + numbered lists | `content/ex_list.json` | `list_id`, `depth`, `marker_type` bullet/number |
| Text modifiers | `content/ex_modifiers.json` | emphasis, link, math, underline, strikethrough, sub/sup, syntax_highlight |
| Display math | `content/ex_math.json` | `role: math`, `content.type: math` |
| Inline code (text) | `content/ex_code.json` | `role: code` + `content.type: text` (not `code_block`) |
| Embedded image | `content/ex_image.json` | PNG path, millipt size, role `self_align` in theme |
| Page header/footer | `manifest.json` | `running_blocks`, `{{page_current}}` placeholders |

Verify the whole catalog:

```bash
python scripts/pack_verify.py catalog -o /tmp/catalog.K2F
```

Run from the `skills/k2f/` directory with `k2f` on PATH.
