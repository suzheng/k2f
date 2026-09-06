# Allowed JSON keys

Skim this page, then open only the matching file under [`schema/`](../../schema/). Do not copy schema files into an author directory.

## Node (`content/*.json`)

Required: `id`, `role`, `content`. Optional: `variant`, `layout`, `modifiers`, `break_inside`, `break_before`, `keep_with_next`, `column_span`, `preserve_whitespace`, and list fields `list_id`, `depth`, `marker_type`.

**Never on a node:** `color`, `font_size`, `font_family`, `padding` / `padding_pt`, `self_align`, `text_align`, `box_decoration`, `x`, `y`, or other theme/style fields. Those live on the **role** in `styles/theme.json`.

`content.type` is `text` | `math` | `image` | `container` | `table` (this skill does not author `code_block` or `table_reference` — see [package.md](package.md)). Layout `type` is `stack` | `grid` | `overlay` | `columns`. Details: [`nodes.schema.json`](../../schema/nodes.schema.json).

**Recipes (copy catalog, do not invent CSS):** divider → [`ex_rule.json`](../../catalog/content/ex_rule.json) (`role: "rule"`, insert as a stack/grid **child**); title+logo / space-between → [`ex_split_bar.json`](../../catalog/content/ex_split_bar.json) (2-col `{fr:1}`+`{auto:true}`, right cell may be an image — not overlay); trailing-edge block (letter sender / right-flush cell) → [`ex_end_block.json`](../../catalog/content/ex_end_block.json); cover (logo + nested title groups + year) → [`ex_cover.json`](../../catalog/content/ex_cover.json); cover/slide vertical fill / footer pinned to page bottom → [`ex_poster_shell.json`](../../catalog/content/ex_poster_shell.json); numbered display math → [`ex_math_numbered.json`](../../catalog/content/ex_math_numbered.json) (not `\\tag`); page background image → [`ex_overlay.json`](../../catalog/content/ex_overlay.json) (image child first), not `page_config`. Table cell vertical middle: cell `variant: "center"` (theme `self_align`). No colspan — extra columns + cell `variant: "hbar"` / `"bottom"` ([`ex_table_edges.json`](../../catalog/content/ex_table_edges.json)). No `font_variant` / small-caps (uppercase + role `letter_spacing_pt`). Line box = role `line_height_mult`; gap between nodes = parent `gap` (no node margin). `text_align: center` is the **content box** (page minus margins), not the sheet. Overlay children shrink to content unless that layer sets `width`/`height`.

## Theme role (`styles/theme.json` → `roles`)

Required on every role: `font_family`, `font_size`, `line_height_mult`, `color`. Optional: `letter_spacing_pt`, `first_line_indent_pt`, `text_align`, `bold`, `italic`, `self_align`, `box_decoration`, `list_style`, `variants`.

`box_decoration` values are **named primitive strings** (`background`, `border`, `corner_radius`, `shadow`, `blur`) except `padding_pt` (millipt or `{top,right,bottom,left}`). Inline fill/border objects on a role → `UNKNOWN_PRIMITIVE`. Define names under `primitives`. Details: [`styles.schema.json`](../../schema/styles.schema.json) + [`visual_primitives.schema.json`](../../schema/visual_primitives.schema.json).

## Manifest

`title`, `canvas_mode: "paged"`, `page_config` (`width`, `height`, `margin`), `engine_version`. Optional: `author`, `created_at`, `generated_by`, `running_blocks`. Details: [`manifest.schema.json`](../../schema/manifest.schema.json).
