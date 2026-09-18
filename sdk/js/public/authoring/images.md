# Images

An image is a node with `content.type: "image"`. The engine does not inspect the binary for size — you declare `width` and `height` in millipt (1 pt = 1000).

Copy [`ex_image.json`](../../skills/k2f/catalog/content/ex_image.json) into `content/root.json` `children`, and copy the PNG into `assets/images/`.

## Shape

- `content.type` is `"image"`
- `content.value.src` is a package path under `assets/images/` (not `path`, not a URL)
- `content.value.width` / `height` are millipt
- Role is `image` in the starter theme (`self_align` and `image_fit` live on the role, not the node)
- Place the file next to the declared `src` before `k2f pack`

## Theme

Starter defines `image` with `self_align: start` and a `cover` variant (`image_fit: cover`). See [Theme and fonts](theme.md).

## See also

Magazine image + copy: [`ex_media_row.json`](../../skills/k2f/catalog/content/ex_media_row.json) (a [grid](layout.md), not this page).

## Common mistakes

- Forgetting to copy the bitmap into `assets/images/`
- Using CSS `px` or raw pixels instead of millipt
- Putting `width` / `height` on `layout` instead of `content.value`
