# Web viewer integration

Embed a `.K2F` document in a website the same way you would embed a PDF viewer: unpack, verify, paint the published lock. The browser is **not** the layout engine. Opening a locked file paints `document.K2F.lock` only — it does not recompile.

Official scale for raster pages is **2×** (`Viewer.official_scale()` → `2`).

## Install / build

```bash
bash scripts/build-sdk-js.sh
# Optional (this repo / site preview-only): bash scripts/build-viewer-only-js.sh
```

npm consumers: `npm i @openk2f/k2f`. Lazy-load `@openk2f/k2f/viewer` in app bundles. Copy `k2f_wasm_bg.wasm` to a public URL if your bundler cannot resolve it next to the JS module.

| Binary | Build | Init | When |
|--------|-------|------|------|
| Full sdk (`sdk/js/wasm/`) | `build-sdk-js.sh` | `initWasm` | Edit, compile, Editor; **npm default** |
| Viewer-only (`@openk2f/k2f/wasm-viewer/` in npm) | `build-viewer-only-js.sh` | `initViewerWasm` | Paint/preview only; WASM at `node_modules/@openk2f/k2f/wasm-viewer/k2f_viewer_bg.wasm` |

Serve over **HTTP** (WASM will not load from `file://`).

## Quick start

### Web component

```html
<k2f-viewer src="/files/report.K2F" editable></k2f-viewer>
<script type="module">
  import "@openk2f/k2f/viewer";
</script>
```

Attributes:

| Attribute | Effect |
|-----------|--------|
| `src` | URL of a `.K2F` package (fetched on connect / change). Only observed attribute. |
| `editable` | Allow **Edit** toolbar + surgical popover (`replace_text` + save). Opens in **view mode**; user clicks Edit first. Read at mount. |
| `no-banner` | Hide the integrity banner. Read at mount. |

### Mount API

```javascript
import { mountK2fViewer, initWasm } from "@openk2f/k2f/viewer";

await initWasm(); // or initWasm("/k2f_wasm_bg.wasm")
const bytes = await fetch("/invoice.K2F").then((r) => r.arrayBuffer());
const handle = await mountK2fViewer(
  document.getElementById("k2f-root"),
  new Uint8Array(bytes),
  { banner: true, editable: false },
);
// handle.destroy() on unmount
```

### Options (`ViewerMountOptions`)

| Option | Type | Default | Meaning |
|--------|------|---------|---------|
| `banner` | `boolean` | `true` | Show integrity banner |
| `editable` | `boolean` | `false` | Allow Edit toolbar + popover (forces sdk WASM); default open is view-only |
| `runtime` | `"sdk" \| "viewer"` | inferred | Explicit WASM; `editable` or `"sdk"` → full sdk |

### View vs edit mode

Like a PDF preview app, the viewer opens in **view mode**: pan, zoom, text selection, copy, links. Clicks do **not** open the edit popover.

When `editable: true`, the toolbar shows **Edit**. Click **Edit** to enter edit mode, click a node for the on-page popover, then **Save and relock**. **Done** returns to view mode (popover closes; WASM stays loaded).

When `editable: false`, there is no Edit button and no popover.

### Handle (`ViewerHandle`)

| Method | Purpose |
|--------|---------|
| `destroy()` | Tear down listeners and WASM viewers |
| `goPage(n)` | Jump page `n` into view |
| `export(format?)` | Export K2F / PDF / Markdown / PNG / JPG (`{ bytes, filename, mime }`) |
| `selectId(id)` | Jump to a node (emits `k2f-select`; popover only in edit mode) |
| `open(bytes)` | Replace the open package |
| `editing` | `boolean` — `true` when the Edit toggle is active |
| `viewer` / `editor` | Underlying WASM objects (`editor` is null when not editable) |

## Events

Custom events bubble from the host element (bubbles + composed):

| Event | Detail |
|-------|--------|
| `k2f-open` | `{ banner, statusCode, pageCount, handle }` |
| `k2f-page-change` | `{ page }` |
| `k2f-select` | Selection JSON for the clicked/selected node |

## Bundlers (Vite / Next.js)

1. Copy WASM into `public/` (skill helper or site script):

```bash
node skills/k2f/scripts/copy-wasm.mjs --dest ./public
# Sibling k2f-site: node scripts/copy-wasm.mjs (from that checkout)
```

2. Call `await initWasm("/k2f_wasm_bg.wasm")` before mount (or `initViewerWasm("/k2f_viewer_bg.wasm")` for preview-only in this repo).
3. Mount only on the client — do not SSR the canvas; do not pass `Uint8Array` as an RSC prop.
4. Page width comes from `page_config`, not the viewport. On open, the viewer **fit-to-width** when the widest page is wider than the scroll stage (snapped to toolbar zoom steps, capped at 100%). Use **+ / −** to zoom in after open.
5. Toolbar **Fullscreen** toggles the `<k2f-viewer>` host into browser fullscreen (hidden when the Fullscreen API is unavailable).
6. Import from `@openk2f/k2f/viewer` only (avoids duplicating `initWasm` via bare `@openk2f/k2f`).

The adoption portal for maintainers lives in the sibling [`k2f-site`](https://github.com/suzheng/k2f-site) checkout and consumes this package via `file:../k2f/sdk/js`. npm consumers do not need that repo.

See [examples/web-embed/README.md](../../examples/web-embed/README.md) for a runnable demo.

## Rules

1. **Never** use `fillText` or CSS flow for document body text.
2. **Opening ≠ compiling.** Locked packages paint the lock only.
3. Integrity banners (`SIGNED`, `UNSIGNED`, `SIGNED_BUT_BROKEN`, `BROKEN_INTEGRITY`, `UNLOCKED`) must stay visible to the user.
4. Prefer the npm `@openk2f/k2f` / `@openk2f/k2f/viewer` entries; do not reimplement paint.

## Related

- Agent skill: [embedding-viewer workflow](../../skills/k2f/references/embedding-viewer.md)
- Package README: [sdk/js/README.md](../../sdk/js/README.md)
- TypeScript surface: [sdk/js/types.d.ts](../../sdk/js/types.d.ts)
- Format integrity: [k2f-v0.1.md](../spec/k2f-v0.1.md)
