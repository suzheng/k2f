# Allowed JSON keys

Skim this page, then open only the matching file under [`schema/`](../../schema/). Do not copy schema files into an author directory.

## Node (`content/*.json`)

Required: `id`, `role`, `content`. Optional: `variant`, `layout`, `modifiers`, `break_inside`, `break_before`, `keep_with_next`, `column_span`, `colspan`, `preserve_whitespace`. When `role` is `list_item`: required `list_id`; optional `depth` (default 0) and `marker_type` (default bullet). `colspan` is table-cell occupancy only.

**Never on a node:** `color`, `font_size`, `font_family`, `padding` / `padding_pt`, `self_align`, `text_align`, `box_decoration`, `x`, `y`, or other theme/style fields. Those live on the **role** in `styles/theme.json`.

`content.type` is `text` | `math` | `image` | `container` | `table` | `form_field` (this skill does not author `code_block` or `table_reference` — see [package.md](package.md)). Layout `type` is `stack` | `grid` | `overlay` | `columns` (omit `type` → stack). Package `content/root.json` must omit `layout` or use **vertical** `stack` — nest grid/overlay/columns/horizontal stack under a child. Container `children` may be omitted (empty). Grid `rows` is optional (including one-row grids): omit → `{auto:true}` rows to fit children; `fr`/`pt` rows must be written. Details: [`nodes.schema.json`](../../schema/nodes.schema.json).

**Image** (`content.type: "image"`): `value.src` (not `path`), `width`, `height` (millipt). Minimal: `{ "id": "pic", "role": "image", "content": { "type": "image", "value": { "src": "assets/images/x.png", "width": 24000, "height": 24000 } } }` — copy [`ex_image.json`](../../catalog/content/ex_image.json).

**Form field** (`content.type: "form_field"`, `role: "form_field"`): a reserved box. Empty `value` still occupies the slot; filling must not reflow following nodes. `kind` is `text` | `multiline` | `checkbox`. Omit `width` in a vertical stack or `{fr:1}` grid cell (parent is bounded); a horizontal stack child must set `width`. `layout` and `modifiers` are forbidden. Set `break_inside: "avoid"`. Placeholder is metadata — it is not painted. Do **not** fake a blank with `____`, `□`/`☐`, or an image. Copy [`ex_form.json`](../../catalog/content/ex_form.json). Minimal:

```json
{
  "id": "app.name",
  "role": "form_field",
  "variant": "underline",
  "break_inside": "avoid",
  "content": {
    "type": "form_field",
    "value": { "kind": "text", "value": "", "placeholder": "Full legal name" }
  }
}
```

**Modifiers** (on a text node): required `range`, `type`, **`intent`**. `range` is UTF-8 **bytes** (`\n` = 1 byte; CJK/emoji are multi-byte). Always `python scripts/modifier_range.py --text "<exact value>" --find "…"`. `intent` must exist under `theme.modifiers.styles[type]` (`default`, `strong`, URL, …).

**Recipes (copy catalog, do not invent CSS):** divider → [`ex_rule.json`](../../catalog/content/ex_rule.json) (`role: "rule"` in a **vertical** stack at default `align_items` stretch — not inside a hugging `align_items: start` title stack; sibling the rule or nest the title group separately); title+logo / space-between → [`ex_split_bar.json`](../../catalog/content/ex_split_bar.json) (2-col `{fr:1}`+`{auto:true}`, right cell may be an image — not overlay); magazine image+copy → [`ex_media_row.json`](../../catalog/content/ex_media_row.json) (`{pt:N}`+`{fr:1}`); trailing-edge block (letter sender / right-flush cell) → [`ex_end_block.json`](../../catalog/content/ex_end_block.json); cover (logo + nested title groups + year) → [`ex_cover.json`](../../catalog/content/ex_cover.json); filled page (invoice/CV/letter/poster/slide/cover/one-page infographic) → [`ex_filled_page.json`](../../catalog/content/ex_filled_page.json) / [`ex_poster_shell.json`](../../catalog/content/ex_poster_shell.json) (role `page_shell`, set `height` to the content box); bleed header + inset body → [`ex_banner_header.json`](../../catalog/content/ex_banner_header.json) (`--margin 0`, `page_shell` `flush` + nested `page_shell`; no negative margin); grower leftover → [`ex_poster_growers.json`](../../catalog/content/ex_poster_growers.json) (`{fr:1}` on figure/dense cards, not a short quote); numbered display math → [`ex_math_numbered.json`](../../catalog/content/ex_math_numbered.json) (not `\\tag`); page background image → [`ex_overlay.json`](../../catalog/content/ex_overlay.json) (image child first), not `page_config`; glass card → [`ex_glass.json`](../../catalog/content/ex_glass.json) (`variant: "glass"`, catalog theme); dark band on a light doc → [`ex_on_dark.json`](../../catalog/content/ex_on_dark.json) (`variant: "on_dark"`, same roles); fillable blanks → [`ex_form.json`](../../catalog/content/ex_form.json) (`form_field`, never `____` in body text). Badge/pill: copy [`ex_badge.json`](../../catalog/content/ex_badge.json) stack wrapper (grid ignores `self_align` on a direct child). Running header/footer split → [`catalog/manifest.json`](../../catalog/manifest.json). Table cell vertical middle: `variant: "center"`; numeric right-align: `variant: "end"` (`text_overrides` in theme). Dense metrics: [`ex_table_dense.json`](../../catalog/content/ex_table_dense.json) (`compact` + weighted `fr`). Three-line table: table `variant: "ruled"` + header `bottom`. No colspan — extra columns + cell `variant: "hbar"` / `"bottom"` ([`ex_table_edges.json`](../../catalog/content/ex_table_edges.json)). Composite cell (title+list / icon row) → [`ex_table_composite.json`](../../catalog/content/ex_table_composite.json). Table `row_gap`/`column_gap` optional (omit → `gap`). No `font_variant` / small-caps (uppercase + role `letter_spacing_pt`). Line box = role `line_height_mult`; gap between nodes = parent `gap` (no node margin). `text_align: center` is the **content box** (page minus margins), not the sheet. Overlay children shrink to content unless that layer sets `width`/`height`.

## Theme role (`styles/theme.json` → `roles`)

`default` must set `font_family`, `font_size`, `line_height_mult`, `color`. Other roles inherit any omitted text field from `default` (container/`rule`/`image` roles need not repeat them). Optional: `letter_spacing_pt`, `first_line_indent_pt`, `text_align`, `bold`, `italic`, `self_align`, `box_decoration`, `list_style` (`marker_box_width_pt`, `marker_gap_pt`, `depth_indent_pt`, `marker_align`, `bullet_glyph`, `number_suffix`; omit = starter defaults), `image_fit` (`contain`|`cover`; omit = contain), `variants`.

Variant keys are only `box_decoration`, `self_align`, `text_overrides`, `list_style`, `image_fit`. Put `bold` / `color` / `text_align` / `font_size` under `text_overrides`, not at the variant root.

`box_decoration` values are **named primitive strings** (`background`, `border`, `corner_radius`, `shadow`, `blur`) except `padding_pt` (millipt, `{top,right,bottom,left}`, or `[top,right,bottom,left]` like page `margin`). Applies to **any** node using the role, including text (badges, table cells, `warning`, `code`). Inline fill/border objects on a role → `UNKNOWN_PRIMITIVE`. Define names under `primitives`. Details: [`styles.schema.json`](../../schema/styles.schema.json) + [`visual_primitives.schema.json`](../../schema/visual_primitives.schema.json).

**Theme modifiers** (`modifiers.styles`): nest `type` → `intent` → patch — not flat keys. Example: `"emphasis": { "strong": { "bold": true } }`. Copy [`starter/styles/theme.json`](../../starter/styles/theme.json); node usage in [`ex_modifiers.json`](../../catalog/content/ex_modifiers.json).

## Manifest

`title`, `canvas_mode: "paged"`, `page_config` (`width`, `height`, `margin`), `engine_version`. No `page_config.background` — sheet fill is the **root** role’s `box_decoration.background` (full page, including margins). Optional: `author`, `created_at`, `generated_by`, `running_blocks` (text or Grid node; placeholders `{{page_current}}`/`{{page_total}}` only — copy [`catalog/manifest.json`](../../catalog/manifest.json)). Details: [`manifest.schema.json`](../../schema/manifest.schema.json).
