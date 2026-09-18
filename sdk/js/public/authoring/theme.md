# Theme and fonts

Appearance lives only in `styles/theme.json`. Each node names a `role` (and optional `variant`); it does not carry `color`, `font_size`, `font_family`, `padding`, `text_align`, or `box_decoration`. Putting those on a node is the most common compile failure.

Inline emphasis, links, and math use `modifiers` on the text node; the theme defines allowed `intent` values under `modifiers.styles` — see [Text](text.md).

Copy [`ex_on_dark.json`](../../skills/k2f/catalog/content/ex_on_dark.json) into `content/root.json` `children` to see `variant: "on_dark"` on existing roles (not cloned `*_dark_*` roles).

## What is in the file

| Section | Purpose |
| --- | --- |
| `font_aliases` | Friendly name → font stem under `assets/fonts/` |
| `palette` | Named colors; roles and primitives reference these keys or hex |
| `primitives` | Reusable surfaces, borders, corners, shadows, blur for `box_decoration` |
| `modifiers` | `precedence` plus `styles[type][intent]` patches for text modifiers |
| `roles` | Typography and decoration keyed by role name |

Start from [`starter/styles/theme.json`](../../skills/k2f/starter/styles/theme.json). For a fuller palette and table/card roles, see [`catalog/styles/theme.json`](../../skills/k2f/catalog/styles/theme.json).

Lengths in the theme (for example `font_size`, `padding_pt`, corner radii) are **millipt** (1 pt = 1000). Page size and margins stay in `manifest.json` `page_config`.

## Roles

Every node's `role` must exist under `theme.roles`. Role `default` must set `font_family`, `font_size`, `line_height_mult`, and `color`. Other roles inherit any omitted text field from `default`.

Optional on a role: `text_align`, `bold`, `italic`, `letter_spacing_pt`, `first_line_indent_pt`, `self_align`, `list_style`, `image_fit`, `box_decoration`, and `variants`. Container, `rule`, and `image` roles need not repeat typography when they do not paint text.

## Variants

A node's `variant`, if set, must exist on that role under `variants`. Variant keys are only `box_decoration`, `self_align`, `text_overrides`, `list_style`, and `image_fit`. Put `bold`, `color`, `text_align`, and `font_size` under `text_overrides`, not at the variant root.

Table numeric cells use `variant: "end"` on the cell node (theme `text_overrides`, not a node `text_align`) — see [Tables](tables.md). Image `cover` behavior is a role variant — see [Images](images.md).

## Palette and primitives

Define colors once in `palette`. Reference palette keys (or hex) from roles and from named entries under `primitives` (`surfaces`, `borders`, `corners`, `shadows`, …).

On a role, `box_decoration` values are **named primitive strings** (`background`, `border`, `corner_radius`, `shadow`, `blur`, …) except `padding_pt` (millipt number or per-edge object). Inline fill or border objects on a role fail compile (`UNKNOWN_PRIMITIVE`). Contract: [`styles.schema.json`](../../skills/k2f/schema/styles.schema.json) and [`visual_primitives.schema.json`](../../skills/k2f/schema/visual_primitives.schema.json).

## Modifier styles

Nest `modifiers.styles` as `type` → `intent` → patch (not flat keys). Example: `"emphasis": { "strong": { "bold": true } }`. Node usage: [`ex_modifiers.json`](../../skills/k2f/catalog/content/ex_modifiers.json).

## Fonts

K2F never uses system fonts. Ship faces under `assets/fonts/` and register them in `theme.font_aliases`. Missing glyphs fail closed (`FONT_MISSING_GLYPH`).

The starter theme ships Roboto. From the skill directory:

```bash
python scripts/init_package.py --workspace ./out/doc --title "…" --page a4 \
  --add-font /path/to/NotoSerif-Regular.ttf
```

`--add-font` keeps Roboto and adds a fallback; `--font` replaces Roboto (use a face that covers Latin). Point `font_family` on the roles you need (`h1`, `body`, …) at the new alias. Catalog assets include Noto Serif and Noto Sans Math; copy [`ex_math.json`](../../skills/k2f/catalog/content/ex_math.json) only after aliases and files match.

## Page background

Sheet fill is the **root node's** `box_decoration.background` (full page, including margins). The starter root uses role `document` with `background: "paper"`. There is no `page_config.background`.

## Allowed keys

Role and theme fields: [Allowed keys](../reference/keys.md#theme-role-stylesthemejson--roles). Exact contract: [`styles.schema.json`](../../skills/k2f/schema/styles.schema.json).

## Common mistakes

- Styling one node in `content/` instead of editing its role in the theme
- `font_family` not listed in `font_aliases` / missing under `assets/fonts/`
- Cloning dark roles (`th_dark_body`) instead of `variant: "on_dark"` — use [`ex_on_dark.json`](../../skills/k2f/catalog/content/ex_on_dark.json)
- Inline color or border objects on a role instead of `palette` + `primitives`
