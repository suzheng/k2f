# Theme and fonts

Style lives only in `styles/theme.json`. Putting `color`, `font_size`, `font_family`, `padding`, or `text_align` on a content node is the most common compile failure.

Read [fields.md](../../skills/k2f/references/writing/fields.md) (theme section), then [`styles.schema.json`](../../skills/k2f/schema/styles.schema.json) and [`visual_primitives.schema.json`](../../skills/k2f/schema/visual_primitives.schema.json). Starter file: [`starter/styles/theme.json`](../../skills/k2f/starter/styles/theme.json).

## Roles

Every node's `role` must exist under `theme.roles`. `default` sets `font_family`, `font_size`, `line_height_mult`, and `color`. Other roles inherit omitted text fields from `default`.

A `variant` on a node must exist on that role. Variant keys are only `box_decoration`, `self_align`, `text_overrides`, `list_style`, and `image_fit`. Put `bold` / `color` / `text_align` / `font_size` under `text_overrides`, not at the variant root.

`box_decoration` values are **named primitives** (`background`, `border`, `corner_radius`, …) except `padding_pt`. Inline color objects on a role → `UNKNOWN_PRIMITIVE`.

## Fonts

K2F never uses system fonts. Embed the face under `assets/fonts/` and alias it in `theme.font_aliases`. Missing glyphs fail closed (`FONT_MISSING_GLYPH`).

Starter ships Roboto. Adding a face:

```bash
python scripts/init_package.py --workspace ./out/doc --title "…" --page a4 \
  --add-font /path/to/NotoSerif-Regular.ttf
```

Then point the roles you want (for example `h1` / `body`) at that alias. Catalog includes Noto Serif and Noto Sans Math; `ex_math.json` still needs the math font plus `font_aliases`.

## Page background

Sheet fill is the **root** role's `box_decoration.background` (the whole page, including margins). There is no `page_config.background`.

## Common mistakes

- Styling a single node instead of its role
- `font_family` that is not in `font_aliases` / `assets/fonts/`
- Cloning dark roles (`th_dark_body`) instead of `variant: "on_dark"` — see [`ex_on_dark.json`](../../skills/k2f/catalog/content/ex_on_dark.json)
