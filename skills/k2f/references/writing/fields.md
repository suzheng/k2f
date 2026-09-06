# Allowed JSON keys

Skim this page, then open only the matching file under [`schema/`](../../schema/). Do not copy schema files into an author directory.

## Node (`content/*.json`)

Required: `id`, `role`, `content`. Optional: `variant`, `layout`, `modifiers`, `break_inside`, `break_before`, `keep_with_next`, `column_span`, `preserve_whitespace`, and list fields `list_id`, `depth`, `marker_type`.

**Never on a node:** `color`, `font_size`, `font_family`, `padding` / `padding_pt`, `self_align`, `text_align`, `box_decoration`, `x`, `y`, or other theme/style fields. Those live on the **role** in `styles/theme.json`.

`content.type` is `text` | `math` | `image` | `container` | `table` (this skill does not author `code_block` or `table_reference` — see [package.md](package.md)). Layout `type` is `stack` | `grid` | `overlay` | `columns`. Details: [`nodes.schema.json`](../../schema/nodes.schema.json).

## Theme role (`styles/theme.json` → `roles`)

Required on every role: `font_family`, `font_size`, `line_height_mult`, `color`. Optional: `letter_spacing_pt`, `first_line_indent_pt`, `text_align`, `bold`, `italic`, `self_align`, `box_decoration`, `list_style`, `variants`.

`box_decoration` values are **named primitive strings** (`background`, `border`, `corner_radius`, `shadow`, `blur`) except `padding_pt` (millipt or `{top,right,bottom,left}`). Inline fill/border objects on a role → `UNKNOWN_PRIMITIVE`. Define names under `primitives`. Details: [`styles.schema.json`](../../schema/styles.schema.json) + [`visual_primitives.schema.json`](../../schema/visual_primitives.schema.json).

## Manifest

`title`, `canvas_mode: "paged"`, `page_config` (`width`, `height`, `margin`), `engine_version`. Optional: `author`, `created_at`, `generated_by`, `running_blocks`. Details: [`manifest.schema.json`](../../schema/manifest.schema.json).
