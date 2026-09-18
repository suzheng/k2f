# Allowed keys

Quick lookup for **what may appear** in `content/*.json`, `styles/theme.json`, and `manifest.json`. This page summarizes the author contract; the matching JSON Schema under [`schema/`](../../skills/k2f/schema/) is normative when they differ.

**Workflow:** skim the section you are editing → open the schema file (`nodes`, `styles` + `visual_primitives`, or `manifest`) → copy shapes from the [Catalog](catalog.md) instead of inventing layout or CSS-like fields.

Topic walkthroughs: [Text](../authoring/text.md) · [Images](../authoring/images.md) · [Tables](../authoring/tables.md) · [Layout](../authoring/layout.md) · [Theme and fonts](../authoring/theme.md).

Agents also use the compact mirror in [`fields.md`](../../skills/k2f/references/writing/fields.md) inside the skill folder.

## Node (`content/*.json`)

### Fields

| | Keys |
| --- | --- |
| **Required** | `id`, `role`, `content` |
| **Optional** | `variant`, `layout`, `modifiers`, `break_inside`, `break_before`, `keep_with_next`, `column_span`, `colspan`, `preserve_whitespace` |
| **`list_item` only** | Required `list_id`; optional `depth` (default `0`), `marker_type` (default bullet) |

`colspan` is table-cell occupancy only. Container `children` may be omitted (empty).

### Never on a node

Do not put theme or paint on a node: `color`, `font_size`, `font_family`, `padding` / `padding_pt`, `self_align`, `text_align`, `box_decoration`, `x`, `y`, or similar. Those belong on the **role** in `styles/theme.json`.

### `content.type`

| Type | Notes |
| --- | --- |
| `text` | Default body copy, headings, code as text (`role: "code"`) |
| `math` | Display or inline LaTeX in `value` |
| `image` | `value.src` (not `path`), `width`, `height` in millipt — [Images](../authoring/images.md), [`ex_image.json`](../../skills/k2f/catalog/content/ex_image.json) |
| `container` | `children` + optional `layout` — [Layout](../authoring/layout.md) |
| `table` | Inline table — [Tables](../authoring/tables.md) |
| `form_field` | Fillable blank or checkbox — see below |

This authoring path does not use `code_block` or `table_reference` (schema-legal elsewhere). See [package.md](../../skills/k2f/references/writing/package.md).

**Root rule:** `content/root.json` must omit `layout` or use a **vertical** `stack` only. Put `grid`, `overlay`, `columns`, or a horizontal stack on a **nested** child.

**Layout `type`:** `stack` \| `grid` \| `overlay` \| `columns` (omit `type` → stack). Grid `rows` is optional; omit → auto rows. `fr` / `pt` row tracks must be written explicitly. Contract: [`nodes.schema.json`](../../skills/k2f/schema/nodes.schema.json).

### Form field (`content.type: "form_field"`)

Reserved box: empty `value` still occupies space; filling must not reflow following nodes.

| Rule | Detail |
| --- | --- |
| `role` | `form_field` |
| `kind` | `text` \| `multiline` \| `checkbox` |
| `width` | Omit in a vertical stack or `{fr:1}` cell; **required** as a horizontal-stack child |
| Forbidden on node | `layout`, `modifiers` |
| Break | Set `break_inside: "avoid"` |
| Placeholder | Metadata only — not painted |
| Do not fake blanks | No `____`, `□`/`☐`, or spacer images in body text |

Copy [`ex_form.json`](../../skills/k2f/catalog/content/ex_form.json). Minimal shape:

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

### Modifiers (text nodes)

Required: `range`, `type`, **`intent`**. `range` is UTF-8 **bytes** (`\n` = 1 byte; CJK and emoji are multi-byte). Always run `python scripts/modifier_range.py --text "<exact value>" --find "…"` from the skill directory. `intent` must exist under `theme.modifiers.styles[type]`. Example nodes: [`ex_modifiers.json`](../../skills/k2f/catalog/content/ex_modifiers.json).

## Catalog recipes

Copy catalog JSON into your package’s `children`. Do not invent CSS-like keys. Full index: [Catalog](catalog.md).

### Layout and page structure

| Goal | Example | Notes |
| --- | --- | --- |
| Horizontal divider | [`ex_rule.json`](../../skills/k2f/catalog/content/ex_rule.json) | `role: "rule"` in a **vertical** stack at default `align_items` stretch — not inside a hugging `align_items: start` title stack; sibling the rule or nest the title group separately |
| Title + logo / space-between row | [`ex_split_bar.json`](../../skills/k2f/catalog/content/ex_split_bar.json) | 2-col `{fr:1}` + `{auto:true}`; right cell may be an image — not overlay |
| Magazine image + copy | [`ex_media_row.json`](../../skills/k2f/catalog/content/ex_media_row.json) | `{pt:N}` + `{fr:1}` |
| Trailing-edge block (sender / right-flush) | [`ex_end_block.json`](../../skills/k2f/catalog/content/ex_end_block.json) | |
| Cover (logo + nested title groups + year) | [`ex_cover.json`](../../skills/k2f/catalog/content/ex_cover.json) | |
| Filled page (invoice, CV, poster, slide, one-pager) | [`ex_filled_page.json`](../../skills/k2f/catalog/content/ex_filled_page.json), [`ex_poster_shell.json`](../../skills/k2f/catalog/content/ex_poster_shell.json) | Role `page_shell`; set `height` to the content box |
| Bleed header + inset body | [`ex_banner_header.json`](../../skills/k2f/catalog/content/ex_banner_header.json) | `--margin 0`; `page_shell` `flush` + nested `page_shell` — no negative margin |
| Grower for leftover space | [`ex_poster_growers.json`](../../skills/k2f/catalog/content/ex_poster_growers.json) | `{fr:1}` on figure/dense cards, not a short quote |
| Background image under content | [`ex_overlay.json`](../../skills/k2f/catalog/content/ex_overlay.json) | Image child first — not `page_config` |
| Glass card | [`ex_glass.json`](../../skills/k2f/catalog/content/ex_glass.json) | `variant: "glass"` (catalog theme) |
| Dark band on a light document | [`ex_on_dark.json`](../../skills/k2f/catalog/content/ex_on_dark.json) | `variant: "on_dark"` on existing roles |
| Fillable blanks | [`ex_form.json`](../../skills/k2f/catalog/content/ex_form.json) | `form_field` — never underscores in body text |
| Badge / pill | [`ex_badge.json`](../../skills/k2f/catalog/content/ex_badge.json) | Stack wrapper (grid ignores `self_align` on a direct child) |
| Running header / footer split | [`catalog/manifest.json`](../../skills/k2f/catalog/manifest.json) | Placeholders `{{page_current}}` / `{{page_total}}` only |
| Numbered display math | [`ex_math_numbered.json`](../../skills/k2f/catalog/content/ex_math_numbered.json) | Not `\tag` |

### Tables

| Goal | Approach |
| --- | --- |
| Cell vertical center | `variant: "center"` on the cell |
| Numeric right-align | `variant: "end"` (`text_overrides` in theme, not node `text_align`) |
| Dense metrics grid | [`ex_table_dense.json`](../../skills/k2f/catalog/content/ex_table_dense.json) — `compact` + weighted `fr` |
| Three-line (ruled) table | Table `variant: "ruled"` + header row `bottom` |
| No colspan — edge rules | Extra columns + cell `variant: "hbar"` / `"bottom"` — [`ex_table_edges.json`](../../skills/k2f/catalog/content/ex_table_edges.json) |
| Composite cell (title + list / icon row) | [`ex_table_composite.json`](../../skills/k2f/catalog/content/ex_table_composite.json) |
| Row/column spacing | `row_gap` / `column_gap` optional (omit → parent `gap`) |

### Typography and spacing (nodes vs theme)

| Topic | Rule |
| --- | --- |
| Small caps | No `font_variant`; use uppercase + role `letter_spacing_pt` |
| Line height vs gap | Line box = role `line_height_mult`; space between siblings = parent `gap` (no node margin) |
| Centered text scope | `text_align: center` on a role is the **content box** (page minus margins), not the physical sheet |
| Overlay sizing | Overlay children shrink to content unless that layer sets `width` / `height` |

## Theme role (`styles/theme.json` → `roles`)

| | |
| --- | --- |
| **`default` required** | `font_family`, `font_size`, `line_height_mult`, `color` |
| **Inheritance** | Other roles inherit any omitted text field from `default` (container / `rule` / `image` roles need not repeat typography) |
| **Optional on role** | `letter_spacing_pt`, `first_line_indent_pt`, `text_align`, `bold`, `italic`, `self_align`, `box_decoration`, `list_style`, `image_fit`, `variants` |

**Variants:** keys are only `box_decoration`, `self_align`, `text_overrides`, `list_style`, `image_fit`. Put `bold`, `color`, `text_align`, and `font_size` under `text_overrides`, not at the variant root.

**`box_decoration`:** values are **named primitive strings** (`background`, `border`, `corner_radius`, `shadow`, `blur`) except `padding_pt` (millipt number, per-edge object, or `[top,right,bottom,left]` like page `margin`). Applies to any node using the role (badges, table cells, warnings, code). Inline fill/border objects on a role fail compile (`UNKNOWN_PRIMITIVE`). Define names under `primitives`.

**`list_style`:** `marker_box_width_pt`, `marker_gap_pt`, `depth_indent_pt`, `marker_align`, `bullet_glyph`, `number_suffix` (omit → starter defaults).

**`image_fit`:** `contain` \| `cover` (omit → `contain`).

**Theme modifiers:** nest `modifiers.styles` as `type` → `intent` → patch — not flat keys. Example: `"emphasis": { "strong": { "bold": true } }`. Starter: [`starter/styles/theme.json`](../../skills/k2f/starter/styles/theme.json).

Contracts: [`styles.schema.json`](../../skills/k2f/schema/styles.schema.json), [`visual_primitives.schema.json`](../../skills/k2f/schema/visual_primitives.schema.json). Walkthrough: [Theme and fonts](../authoring/theme.md).

## Manifest (`manifest.json`)

| | |
| --- | --- |
| **Typical** | `title`, `canvas_mode: "paged"`, `page_config` (`width`, `height`, `margin`), `engine_version` |
| **Optional** | `author`, `created_at`, `generated_by`, `running_blocks` (text or grid node) |
| **No sheet background in manifest** | Page fill is the **root** role’s `box_decoration.background` (full page, including margins) |

Running blocks and page sizes: [`manifest.schema.json`](../../skills/k2f/schema/manifest.schema.json), [package.md](../../skills/k2f/references/writing/package.md).

## See also

- [Catalog](catalog.md) — all `ex_*.json` files and theme labels
- [Format spec](../spec/k2f-v0.3.md)
- [`schema/`](../../skills/k2f/schema/) — do not copy schema files into an author directory (`UNEXPECTED_PATH`)
