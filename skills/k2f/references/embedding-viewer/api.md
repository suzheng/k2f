# Viewer mount API (quick reference)

Use `@openk2f/k2f/viewer` (`mountK2fViewer`, `<k2f-viewer>`). Types also ship on the `@openk2f/k2f` package as `ViewerMountOptions` / `ViewerHandle`.

## Official scale

`Viewer.official_scale()` → `2`. Raster pages are painted at 2×; UI zoom is CSS scale of that raster.

## `<k2f-viewer>` attributes

| Attribute | Effect |
|-----------|--------|
| `src` | URL of a `.K2F` package (fetched on connect / `src` change). **Only observed attribute.** |
| `editable` | Allow **Edit** toolbar + surgical popover. Opens in view mode; user clicks Edit first. Read at mount. |
| `no-banner` | Hide integrity banner. Read at mount. |

Changing `editable` / `no-banner` after connect does **not** remount. Use `element.open(bytes)` or remount to apply new options.

## `ViewerMountOptions`

| Option | Type | Default | Meaning |
|--------|------|---------|---------|
| `banner` | `boolean` | `true` | Show integrity banner |
| `editable` | `boolean` | `false` | Allow Edit toolbar + popover (forces sdk WASM); default open is view-only |
| `runtime` | `"sdk" \| "viewer"` | inferred | Explicit WASM flavor; see [bundlers.md](bundlers.md) |

**View vs edit:** With `editable`, the toolbar shows **Edit** / **Done**. View mode: no popover on click (links still open). Edit mode: click node → text-first popover → Save and relock.

## `ViewerHandle`

| Method / field | Purpose |
|----------------|---------|
| `destroy()` | Tear down listeners and free WASM objects |
| `goPage(n)` | Scroll page `n` into view |
| `exportPdf()` | PDF bytes from the published lock (no re-layout) |
| `selectId(id)` | Jump to a node (popover only in edit mode) |
| `open(bytes)` | Replace the open package |
| `editing` | `true` when the Edit toggle is active |
| `viewer` / `editor` | Underlying WASM objects (`editor` is `null` when not editable) |

## Events

Custom events bubble from the host (`bubbles` + `composed`):

| Event | Detail |
|-------|--------|
| `k2f-open` | `{ banner, statusCode, pageCount, handle }` |
| `k2f-page-change` | `{ page }` |
| `k2f-select` | Selection JSON for the clicked/selected node |

## Integrity banners

Derived from package integrity (must stay visible unless `no-banner`):

| Banner | Meaning |
|--------|---------|
| `SIGNED` | Valid hash chain + valid signature |
| `UNSIGNED` | Valid hashes, no signature |
| `SIGNED_BUT_BROKEN` | Signature present but hash/crypto failed — **not** signed |
| `BROKEN_INTEGRITY` | Content / appearance / engine / font mismatch — paints **old** lock |
| `UNLOCKED` | No `document.K2F.lock` |

`signed_by` on the file is an unbound claim. Trusted display names come only from app-side issuer maps, not from the package alone.

## Selection and copy

- Transparent spans from `viewer.text_layer(page)` over the PNG.
- Copy: `text/plain` plus `application/x-k2f-nodes+json` when the selection intersects K2F text spans.
- Default `text/plain` is Markdown via `viewer.selection_markdown(rangesJson)` (semantic tree). Toolbar **Copy: Markdown / Plain** (persisted as `localStorage.k2f.copyFormat`); `mountK2fViewer(..., { copyFormat })` overrides the initial value.
- With `editable`, toolbar **Edit** → click node → text-first on-page popover → Save and relock (not a separate DOM typography path). View mode is default; no popover until Edit is active.
