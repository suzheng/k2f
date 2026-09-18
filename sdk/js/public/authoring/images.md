# Images

An image is a node with `content.type: "image"` and `role: "image"`. The engine does not read width or height from the file — you declare them in `content.value` as millipt (1 pt = 1000).

Copy [`ex_image.json`](../../skills/k2f/catalog/content/ex_image.json) into `content/root.json` `children` (keep `id: "root"` on the document; the catalog file is a small section, not a full root). Put the bytes at `value.src` under `assets/images/` before `k2f pack`.

## Asset files

| Rule | Detail |
|------|--------|
| Path | Package-relative under `assets/images/` only (not the `assets/` root) |
| `src` | Field name is `src`, not `path`; not an `http://` URL |
| Formats | PNG, JPEG, WebP, SVG — **not GIF** |
| SVG text | Rasterized without system fonts. `<text>`, `<tspan>`, `<textPath>`, and `<foreignObject>` fail at pack — use `<path>` or a sibling K2F text node |

## Node shape

- `content.value.src`, `width`, and `height` are required
- `width` / `height` are the declared paint size (millipt), not CSS pixels
- `role` is usually `image`; starter theme already defines that role

## Fit and alignment

In a vertical stack, default `align_items` is `stretch`: the image **box** spans the content width; the bitmap is painted inside it.

- **Contain** (default): letterbox, centered — role `image_fit` omitted or `contain`
- **Cover**: center-crop into the box — node `variant: "cover"` on role `image` in the starter theme
- **Left / right**: set `self_align` to `start` or `end` on the **role** in `styles/theme.json`, not on the node

Do not put `image_fit` or `self_align` on the image node. See [Theme and fonts](theme.md).

## See also (catalog)

| Need | File |
|------|------|
| Fixed-width figure + flowing copy | [`ex_media_row.json`](../../skills/k2f/catalog/content/ex_media_row.json) ([grid](layout.md)) |
| Title + logo / trailing image | [`ex_split_bar.json`](../../skills/k2f/catalog/content/ex_split_bar.json) |
| Full-bleed background image | [`ex_overlay.json`](../../skills/k2f/catalog/content/ex_overlay.json) ([layout](layout.md)) |

## Allowed keys

[Allowed keys](../reference/keys.md) (image row) and [`nodes.schema.json`](../../skills/k2f/schema/nodes.schema.json).

## Common mistakes

- Omitting the file under `assets/images/` before pack
- Using CSS `px` or raw pixel numbers instead of millipt
- Putting declared `width` / `height` on `layout` instead of `content.value`
- Putting `image_fit` or `self_align` on the node instead of the theme role
- GIF assets or external URLs in `src`
- SVG that still contains live text tags
